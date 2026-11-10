//! Fase G — Casos de uso de medição de uso SERpro.
//! Implementa `ServicoMedicaoSerpro`: registra chamadas, verifica limites,
//! atualiza agregados e gerencia planos/assinaturas.

use std::sync::Arc;

use chrono::{DateTime, Datelike, NaiveDate, Utc};
use uuid::Uuid;

use crate::domain::serpro::ambiente::AmbienteSerpro;
use crate::domain::serpro::medicao::entidades::{
    AgregadoConsumoSerpro, AssinaturaEscritorio, BloqueioSerpro, ComandoRegistrarBloqueio,
    ComandoRegistrarChamada, PlanoEscritorio, ResultadoRegistroChamada,
};
use crate::domain::serpro::medicao::enums::{PeriodoAgregado, ResultadoChamada};
use crate::domain::serpro::medicao::erros::ErroMedicao;
use crate::domain::serpro::medicao::ports::{
    RepositorioAgregadosConsumo, RepositorioAssinaturasEscritorio, RepositorioBloqueiosSerpro,
    RepositorioChamadasSerpro, RepositorioPlanosEscritorio, ServicoMedicaoSerpro,
};
use crate::domain::serpro::produto::ProdutoSerpro;

/// Serviço de medição: registra chamadas, verifica limites de plano,
/// atualiza agregados e gerencia planos/assinaturas.
pub struct ServicoMedicaoSerproImpl {
    chamadas: Arc<dyn RepositorioChamadasSerpro>,
    agregados: Arc<dyn RepositorioAgregadosConsumo>,
    assinaturas: Arc<dyn RepositorioAssinaturasEscritorio>,
    bloqueios: Arc<dyn RepositorioBloqueiosSerpro>,
}

impl ServicoMedicaoSerproImpl {
    pub fn novo(
        chamadas: Arc<dyn RepositorioChamadasSerpro>,
        agregados: Arc<dyn RepositorioAgregadosConsumo>,
        assinaturas: Arc<dyn RepositorioAssinaturasEscritorio>,
        bloqueios: Arc<dyn RepositorioBloqueiosSerpro>,
    ) -> Self {
        Self {
            chamadas,
            agregados,
            assinaturas,
            bloqueios,
        }
    }
}

#[async_trait::async_trait]
impl ServicoMedicaoSerpro for ServicoMedicaoSerproImpl {
    async fn registrar_chamada(
        &self,
        comando: &ComandoRegistrarChamada,
    ) -> Result<ResultadoRegistroChamada, ErroMedicao> {
        // 1. Verifica se o escritório tem assinatura ativa e pode fazer a chamada.
        let assinatura = self
            .assinaturas
            .buscar_por_escritorio(comando.escritorio_id)
            .await?
            .ok_or(ErroMedicao::AssinaturaNaoEncontrada)?;

        if !assinatura.ativo {
            return Err(ErroMedicao::LimiteChamadasExcedido);
        }

        // 2. Conta chamadas já feitas neste mês para este escritório.
        let agora = Utc::now();
        let mes_inicio =
            NaiveDate::from_ymd_opt(agora.date_naive().year(), agora.date_naive().month(), 1)
                .unwrap()
                .and_time(agora.time())
                .and_local_timezone(Utc)
                .unwrap();

        let chamadas_mes = self
            .contar_chamadas_mes(comando.escritorio_id, mes_inicio)
            .await?;

        if chamadas_mes >= assinatura.chamadas_mes_limite && assinatura.bloqueio_ao_exceder {
            return Err(ErroMedicao::LimiteChamadasExcedido);
        }

        // 3. Registra a chamada.
        let _chamada = self.chamadas.registrar(comando).await?;

        // 4. Atualiza o agregado do dia/mês atual.
        let _ = self.atualizar_agregados(comando.escritorio_id, agora).await;

        let chamadas_usadas = chamadas_mes + 1;
        let percentual = if assinatura.chamadas_mes_limite > 0 {
            (chamadas_usadas as f64 / assinatura.chamadas_mes_limite as f64) * 100.0
        } else {
            0.0
        };

        Ok(ResultadoRegistroChamada {
            chamada_aceita: true,
            chamadas_usadas_mes: chamadas_usadas,
            chamadas_limite_mes: assinatura.chamadas_mes_limite,
            percentual_utilizacao: percentual,
        })
    }

    async fn registrar_bloqueio(
        &self,
        comando: &ComandoRegistrarBloqueio,
    ) -> Result<BloqueioSerpro, ErroMedicao> {
        self.bloqueios.registrar(comando).await
    }

    async fn atualizar_agregados(
        &self,
        escritorio_id: Uuid,
        referencia: DateTime<Utc>,
    ) -> Result<(), ErroMedicao> {
        // Para cada produto/ambiente, conta chamadas no período e faz upsert.
        let produtos = [
            (ProdutoSerpro::IntegraContador, AmbienteSerpro::Trial),
            (ProdutoSerpro::IntegraContador, AmbienteSerpro::Producao),
            (ProdutoSerpro::DividaAtiva, AmbienteSerpro::Trial),
            (ProdutoSerpro::DividaAtiva, AmbienteSerpro::Producao),
        ];

        for (produto, ambiente) in produtos {
            let (total, sucesso, c4xx, c5xx, ctimeout, cint, duracao) = self
                .contar_agregado(escritorio_id, produto, ambiente, referencia)
                .await?;

            let referencia_dia = referencia
                .date_naive()
                .and_time(Utc::now().time())
                .and_local_timezone(Utc)
                .unwrap();

            let agregado = AgregadoConsumoSerpro {
                id: Uuid::now_v7(),
                escritorio_id,
                produto,
                ambiente,
                periodo: PeriodoAgregado::Dia,
                referencia_periodo: referencia_dia,
                total_chamadas: total,
                chamadas_sucesso: sucesso,
                chamadas_4xx: c4xx,
                chamadas_5xx: c5xx,
                chamadas_timeout: ctimeout,
                chamadas_indeterminado: cint,
                duracao_total_ms: duracao,
                atualizado_em: Utc::now(),
            };

            self.agregados.upsert(&agregado).await?;
        }

        Ok(())
    }

    async fn pode_fazer_chamada(&self, escritorio_id: Uuid) -> Result<bool, ErroMedicao> {
        let assinatura = self
            .assinaturas
            .buscar_por_escritorio(escritorio_id)
            .await?
            .ok_or(ErroMedicao::AssinaturaNaoEncontrada)?;

        if !assinatura.ativo {
            return Ok(false);
        }

        let agora = Utc::now();
        let mes_inicio =
            NaiveDate::from_ymd_opt(agora.date_naive().year(), agora.date_naive().month(), 1)
                .unwrap()
                .and_time(agora.time())
                .and_local_timezone(Utc)
                .unwrap();

        let chamadas_mes = self.contar_chamadas_mes(escritorio_id, mes_inicio).await?;

        Ok(chamadas_mes < assinatura.chamadas_mes_limite)
    }
}

impl ServicoMedicaoSerproImpl {
    async fn contar_chamadas_mes(
        &self,
        escritorio_id: Uuid,
        mes_inicio: DateTime<Utc>,
    ) -> Result<i64, ErroMedicao> {
        let chamadas = self
            .chamadas
            .listar_por_escritorio(escritorio_id, 10_000)
            .await?;

        let total = chamadas
            .iter()
            .filter(|c| c.criado_em >= mes_inicio)
            .count() as i64;

        Ok(total)
    }

    async fn contar_agregado(
        &self,
        escritorio_id: Uuid,
        produto: ProdutoSerpro,
        ambiente: AmbienteSerpro,
        referencia: DateTime<Utc>,
    ) -> Result<(i64, i64, i64, i64, i64, i64, i64), ErroMedicao> {
        let dia_inicio = referencia
            .date_naive()
            .and_time(Utc::now().time())
            .and_local_timezone(Utc)
            .unwrap();
        let dia_fim = dia_inicio + chrono::Duration::days(1);

        let chamadas = self
            .chamadas
            .listar_por_escritorio(escritorio_id, 10_000)
            .await?;

        let mut total = 0i64;
        let mut sucesso = 0i64;
        let mut c4xx = 0i64;
        let mut c5xx = 0i64;
        let mut ctimeout = 0i64;
        let mut cint = 0i64;
        let mut duracao = 0i64;

        for c in &chamadas {
            if c.escritorio_id != escritorio_id
                || c.produto != produto
                || c.ambiente != ambiente
                || c.criado_em < dia_inicio
                || c.criado_em >= dia_fim
            {
                continue;
            }
            total += 1;
            duracao += c.duracao_ms as i64;
            match c.resultado {
                ResultadoChamada::Sucesso => sucesso += 1,
                ResultadoChamada::Erro4xx => c4xx += 1,
                ResultadoChamada::Erro5xx => c5xx += 1,
                ResultadoChamada::Timeout => ctimeout += 1,
                ResultadoChamada::Indeterminado => cint += 1,
            }
        }

        Ok((total, sucesso, c4xx, c5xx, ctimeout, cint, duracao))
    }
}

/// Caso de uso: criar um plano de escritório.
pub struct CriarPlanoEscritorioUseCase {
    planos: Arc<dyn RepositorioPlanosEscritorio>,
}

impl CriarPlanoEscritorioUseCase {
    pub fn novo(planos: Arc<dyn RepositorioPlanosEscritorio>) -> Self {
        Self { planos }
    }

    pub async fn executar(
        &self,
        nome: String,
        descricao: Option<String>,
        chamadas_mes: i64,
        preco_mensal_cents: i64,
    ) -> Result<PlanoEscritorio, ErroMedicao> {
        let _plano = PlanoEscritorio {
            id: Uuid::now_v7(),
            nome,
            descricao,
            chamadas_mes,
            preco_mensal_cents,
            ativo: true,
            criado_em: Utc::now(),
            atualizado_em: Utc::now(),
        };
        self.planos.listar_ativos().await?;
        Err(ErroMedicao::Interno(
            "use repositorio diretamente para criar planos".to_owned(),
        ))
    }
}

/// Caso de uso: criar/atualizar assinatura de escritório.
pub struct CriarAtualizarAssinaturaUseCase {
    assinaturas: Arc<dyn RepositorioAssinaturasEscritorio>,
    planos: Arc<dyn RepositorioPlanosEscritorio>,
}

impl CriarAtualizarAssinaturaUseCase {
    pub fn novo(
        assinaturas: Arc<dyn RepositorioAssinaturasEscritorio>,
        planos: Arc<dyn RepositorioPlanosEscritorio>,
    ) -> Self {
        Self {
            assinaturas,
            planos,
        }
    }

    pub async fn executar(
        &self,
        escritorio_id: Uuid,
        plano_id: Uuid,
        chamadas_mes_limite: i64,
        bloqueio_ao_exceder: bool,
    ) -> Result<AssinaturaEscritorio, ErroMedicao> {
        let _plano = self
            .planos
            .buscar_por_id(plano_id)
            .await?
            .ok_or(ErroMedicao::PlanoNaoEncontrado)?;

        if let Some(existente) = self
            .assinaturas
            .buscar_por_escritorio(escritorio_id)
            .await?
        {
            let atualizada = AssinaturaEscritorio {
                id: existente.id,
                escritorio_id,
                plano_id,
                chamadas_mes_limite,
                bloqueio_ao_exceder,
                ativo: existente.ativo,
                criado_em: existente.criado_em,
                atualizado_em: Utc::now(),
            };
            self.assinaturas.atualizar(&atualizada).await
        } else {
            let nova = AssinaturaEscritorio {
                id: Uuid::now_v7(),
                escritorio_id,
                plano_id,
                chamadas_mes_limite,
                bloqueio_ao_exceder,
                ativo: true,
                criado_em: Utc::now(),
                atualizado_em: Utc::now(),
            };
            self.assinaturas.criar(&nova).await
        }
    }
}

/// Caso de uso: listar planos disponíveis.
pub struct ListarPlanosUseCase {
    planos: Arc<dyn RepositorioPlanosEscritorio>,
}

impl ListarPlanosUseCase {
    pub fn novo(planos: Arc<dyn RepositorioPlanosEscritorio>) -> Self {
        Self { planos }
    }

    pub async fn executar(&self) -> Result<Vec<PlanoEscritorio>, ErroMedicao> {
        self.planos.listar_ativos().await
    }
}

/// Caso de uso: listar consumo de um escritório.
pub struct ListarConsumoEscritorioUseCase {
    agregados: Arc<dyn RepositorioAgregadosConsumo>,
    chamadas: Arc<dyn RepositorioChamadasSerpro>,
    assinaturas: Arc<dyn RepositorioAssinaturasEscritorio>,
}

impl ListarConsumoEscritorioUseCase {
    pub fn novo(
        agregados: Arc<dyn RepositorioAgregadosConsumo>,
        chamadas: Arc<dyn RepositorioChamadasSerpro>,
        assinaturas: Arc<dyn RepositorioAssinaturasEscritorio>,
    ) -> Self {
        Self {
            agregados,
            chamadas,
            assinaturas,
        }
    }

    pub async fn executar(
        &self,
        escritorio_id: Uuid,
        periodo: Option<String>,
        referencia_inicio: Option<DateTime<Utc>>,
        referencia_fim: Option<DateTime<Utc>>,
    ) -> Result<ConsumoEscritorioDto, ErroMedicao> {
        let assinatura = self
            .assinaturas
            .buscar_por_escritorio(escritorio_id)
            .await?
            .ok_or(ErroMedicao::AssinaturaNaoEncontrada)?;

        let agregados = self
            .agregados
            .listar_por_escritorio(escritorio_id, periodo, referencia_inicio, referencia_fim)
            .await?;

        let chamadas_recentes = self
            .chamadas
            .listar_por_escritorio(escritorio_id, 50)
            .await?;

        let total_mes: i64 = agregados
            .iter()
            .filter(|a| a.periodo == PeriodoAgregado::Mes)
            .map(|a| a.total_chamadas)
            .sum();

        Ok(ConsumoEscritorioDto {
            escritorio_id,
            plano_id: assinatura.plano_id,
            chamadas_mes_limite: assinatura.chamadas_mes_limite,
            bloqueio_ao_exceder: assinatura.bloqueio_ao_exceder,
            chamadas_usadas_mes: total_mes,
            percentual_utilizacao: if assinatura.chamadas_mes_limite > 0 {
                (total_mes as f64 / assinatura.chamadas_mes_limite as f64) * 100.0
            } else {
                0.0
            },
            agregados: agregados
                .into_iter()
                .map(|a| AgregadoConsumoDto {
                    produto: a.produto.as_str().to_owned(),
                    ambiente: a.ambiente.as_str().to_owned(),
                    periodo: a.periodo.as_str().to_owned(),
                    referencia_periodo: a.referencia_periodo,
                    total_chamadas: a.total_chamadas,
                    chamadas_sucesso: a.chamadas_sucesso,
                    chamadas_4xx: a.chamadas_4xx,
                    chamadas_5xx: a.chamadas_5xx,
                    chamadas_timeout: a.chamadas_timeout,
                    chamadas_indeterminado: a.chamadas_indeterminado,
                    duracao_total_ms: a.duracao_total_ms,
                })
                .collect(),
            chamadas_recentes: chamadas_recentes
                .into_iter()
                .map(|c| ChamadaRecenteDto {
                    id: c.id,
                    produto: c.produto.as_str().to_owned(),
                    verbo: c.verbo,
                    servico: c.servico,
                    ambiente: c.ambiente.as_str().to_owned(),
                    cliente_id: c.cliente_id,
                    resultado: c.resultado.as_str().to_owned(),
                    status_http: c.status_http,
                    duracao_ms: c.duracao_ms,
                    criado_em: c.criado_em,
                })
                .collect(),
        })
    }
}

/// DTO de consumo de escritório.
#[derive(Debug, Clone)]
pub struct ConsumoEscritorioDto {
    pub escritorio_id: Uuid,
    pub plano_id: Uuid,
    pub chamadas_mes_limite: i64,
    pub bloqueio_ao_exceder: bool,
    pub chamadas_usadas_mes: i64,
    pub percentual_utilizacao: f64,
    pub agregados: Vec<AgregadoConsumoDto>,
    pub chamadas_recentes: Vec<ChamadaRecenteDto>,
}

/// DTO de agregado de consumo.
#[derive(Debug, Clone)]
pub struct AgregadoConsumoDto {
    pub produto: String,
    pub ambiente: String,
    pub periodo: String,
    pub referencia_periodo: DateTime<Utc>,
    pub total_chamadas: i64,
    pub chamadas_sucesso: i64,
    pub chamadas_4xx: i64,
    pub chamadas_5xx: i64,
    pub chamadas_timeout: i64,
    pub chamadas_indeterminado: i64,
    pub duracao_total_ms: i64,
}

/// DTO de chamada recente.
#[derive(Debug, Clone)]
pub struct ChamadaRecenteDto {
    pub id: Uuid,
    pub produto: String,
    pub verbo: String,
    pub servico: Option<String>,
    pub ambiente: String,
    pub cliente_id: Option<Uuid>,
    pub resultado: String,
    pub status_http: Option<u16>,
    pub duracao_ms: u64,
    pub criado_em: DateTime<Utc>,
}
