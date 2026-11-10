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
    RepositorioChamadasSerpro, RepositorioEscritoriosConsumo, RepositorioPlanosEscritorio,
    ServicoMedicaoSerpro,
};
use crate::domain::serpro::produto::ProdutoSerpro;

/// Serviço de medição: registra chamadas, verifica limites de plano,
/// atualiza agregados e gerencia planos/assinaturas.
pub struct ServicoMedicaoSerproImpl {
    chamadas: Arc<dyn RepositorioChamadasSerpro>,
    agregados: Arc<dyn RepositorioAgregadosConsumo>,
    assinaturas: Arc<dyn RepositorioAssinaturasEscritorio>,
    bloqueios: Arc<dyn RepositorioBloqueiosSerpro>,
    notificador: Option<Arc<dyn NotificadorConsumo>>,
}

#[async_trait::async_trait]
pub trait NotificadorConsumo: Send + Sync {
    /// Cria notificação in-app quando o consumo cruza 80% ou 100% do limite.
    async fn notificar_limite(
        &self,
        escritorio_id: Uuid,
        usuario_id: Option<Uuid>,
        percentual: f64,
        limite: i64,
    ) -> Result<(), ErroMedicao>;
}

/// Verificação dos limites de clientes/usuários do plano nos casos de uso
/// de cadastro.
#[async_trait::async_trait]
pub trait LimitesPlano: Send + Sync {
    async fn pode_criar_cliente(&self, escritorio_id: Uuid) -> Result<bool, ErroMedicao>;
    async fn pode_criar_usuario(&self, escritorio_id: Uuid) -> Result<bool, ErroMedicao>;
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
            notificador: None,
        }
    }

    #[must_use]
    pub fn com_notificador(mut self, notificador: Arc<dyn NotificadorConsumo>) -> Self {
        self.notificador = Some(notificador);
        self
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
        crate::presentation::observability::registrar_chamada_serpro(
            comando.resultado != ResultadoChamada::Sucesso,
        );

        // 4. Atualiza o agregado do dia/mês atual.
        let _ = self.atualizar_agregados(comando.escritorio_id, agora).await;

        let chamadas_usadas = chamadas_mes + 1;
        let percentual = if assinatura.chamadas_mes_limite > 0 {
            (chamadas_usadas as f64 / assinatura.chamadas_mes_limite as f64) * 100.0
        } else {
            0.0
        };

        if let Some(notificador) = &self.notificador {
            let anterior = if assinatura.chamadas_mes_limite > 0 {
                (chamadas_mes as f64 / assinatura.chamadas_mes_limite as f64) * 100.0
            } else {
                0.0
            };
            for marco in [80.0_f64, 100.0_f64] {
                if anterior < marco && percentual >= marco {
                    let _ = notificador
                        .notificar_limite(
                            comando.escritorio_id,
                            comando.usuario_id,
                            marco,
                            assinatura.chamadas_mes_limite,
                        )
                        .await;
                }
            }
        }

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

            let agregado_mes = AgregadoConsumoSerpro {
                id: Uuid::now_v7(),
                escritorio_id,
                produto,
                ambiente,
                periodo: PeriodoAgregado::Mes,
                referencia_periodo: inicio_mes(referencia),
                total_chamadas: total,
                chamadas_sucesso: sucesso,
                chamadas_4xx: c4xx,
                chamadas_5xx: c5xx,
                chamadas_timeout: ctimeout,
                chamadas_indeterminado: cint,
                duracao_total_ms: duracao,
                atualizado_em: Utc::now(),
            };
            self.agregados.upsert(&agregado_mes).await?;
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
        clientes_limite: i64,
        usuarios_limite: i64,
        preco_mensal_cents: i64,
    ) -> Result<PlanoEscritorio, ErroMedicao> {
        if nome.trim().is_empty() {
            return Err(ErroMedicao::Interno("nome do plano é obrigatório".into()));
        }
        if chamadas_mes <= 0 || clientes_limite <= 0 || usuarios_limite <= 0 {
            return Err(ErroMedicao::Interno(
                "limites do plano devem ser positivos".into(),
            ));
        }
        let plano = PlanoEscritorio {
            id: Uuid::now_v7(),
            nome: nome.trim().to_owned(),
            descricao,
            chamadas_mes,
            clientes_limite,
            usuarios_limite,
            preco_mensal_cents,
            ativo: true,
            criado_em: Utc::now(),
            atualizado_em: Utc::now(),
        };
        self.planos.criar(&plano).await
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
                clientes_limite: existente.clientes_limite,
                usuarios_limite: existente.usuarios_limite,
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
                clientes_limite: _plano.clientes_limite,
                usuarios_limite: _plano.usuarios_limite,
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

    /// Backoffice: inclui planos inativos.
    pub async fn executar_todos(&self) -> Result<Vec<PlanoEscritorio>, ErroMedicao> {
        self.planos.listar_todos().await
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

        let total_mes: i64 = self
            .chamadas
            .listar_por_periodo(
                escritorio_id,
                None,
                None,
                None,
                inicio_mes(Utc::now()),
                inicio_mes(Utc::now()) + chrono::Duration::days(31),
            )
            .await?
            .len() as i64;

        let percentual = if assinatura.chamadas_mes_limite > 0 {
            (total_mes as f64 / assinatura.chamadas_mes_limite as f64) * 100.0
        } else {
            0.0
        };

        Ok(ConsumoEscritorioDto {
            escritorio_id,
            plano_id: assinatura.plano_id,
            chamadas_mes_limite: assinatura.chamadas_mes_limite,
            clientes_limite: assinatura.clientes_limite,
            usuarios_limite: assinatura.usuarios_limite,
            bloqueio_ao_exceder: assinatura.bloqueio_ao_exceder,
            chamadas_usadas_mes: total_mes,
            percentual_utilizacao: percentual,
            aviso_consumo: aviso_consumo(percentual),
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
    pub clientes_limite: i64,
    pub usuarios_limite: i64,
    pub bloqueio_ao_exceder: bool,
    pub chamadas_usadas_mes: i64,
    pub percentual_utilizacao: f64,
    pub aviso_consumo: Option<String>,
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

/// Primeiro instante do mï¿½s corrente em UTC.
pub fn inicio_mes(referencia: DateTime<Utc>) -> DateTime<Utc> {
    NaiveDate::from_ymd_opt(
        referencia.date_naive().year(),
        referencia.date_naive().month(),
        1,
    )
    .expect("data vï¿½lida")
    .and_hms_opt(0, 0, 0)
    .expect("hora vï¿½lida")
    .and_utc()
}

/// Aviso amigï¿½vel a partir de 80% / 100% do limite mensal.
pub fn aviso_consumo(percentual: f64) -> Option<String> {
    if percentual >= 100.0 {
        Some("Limite mensal de chamadas SERPRO atingido.".to_owned())
    } else if percentual >= 80.0 {
        Some("Consumo acima de 80% do limite mensal de chamadas SERPRO.".to_owned())
    } else {
        None
    }
}

/// Caso de uso: consumo agregado de todos os escritórios (backoffice).
pub struct ConsumoAdminUseCase {
    chamadas: Arc<dyn RepositorioChamadasSerpro>,
    assinaturas: Arc<dyn RepositorioAssinaturasEscritorio>,
    planos: Arc<dyn RepositorioPlanosEscritorio>,
    escritorios: Arc<dyn RepositorioEscritoriosConsumo>,
}

impl ConsumoAdminUseCase {
    pub fn novo(
        chamadas: Arc<dyn RepositorioChamadasSerpro>,
        assinaturas: Arc<dyn RepositorioAssinaturasEscritorio>,
        planos: Arc<dyn RepositorioPlanosEscritorio>,
        escritorios: Arc<dyn RepositorioEscritoriosConsumo>,
    ) -> Self {
        Self {
            chamadas,
            assinaturas,
            planos,
            escritorios,
        }
    }

    pub async fn executar(
        &self,
        inicio: DateTime<Utc>,
        fim: DateTime<Utc>,
        produto: Option<String>,
        ambiente: Option<String>,
        escritorio_id: Option<Uuid>,
    ) -> Result<ConsumoAdminDto, ErroMedicao> {
        let escritorios = self.escritorios.listar().await?;
        let planos = self.planos.listar_todos().await?;
        let mut linhas = Vec::new();
        let mut por_dia: std::collections::BTreeMap<String, (i64, i64, i64)> =
            std::collections::BTreeMap::new();
        let mut total_geral = 0i64;
        let mut erro_geral = 0i64;
        let mut duracao_geral = 0i64;

        for escritorio in escritorios {
            if let Some(filtro) = escritorio_id
                && filtro != escritorio.id
            {
                continue;
            }
            let chamadas = self
                .chamadas
                .listar_por_periodo(
                    escritorio.id,
                    produto.clone(),
                    ambiente.clone(),
                    None,
                    inicio,
                    fim,
                )
                .await?;
            let total = chamadas.len() as i64;
            let sucesso = chamadas
                .iter()
                .filter(|c| c.resultado == ResultadoChamada::Sucesso)
                .count() as i64;
            let erros = total - sucesso;
            let duracao: i64 = chamadas.iter().map(|c| c.duracao_ms as i64).sum();
            for chamada in &chamadas {
                let dia = chamada.criado_em.format("%Y-%m-%d").to_string();
                let entrada = por_dia.entry(dia).or_insert((0, 0, 0));
                entrada.0 += 1;
                if chamada.resultado != ResultadoChamada::Sucesso {
                    entrada.1 += 1;
                }
                entrada.2 += chamada.duracao_ms as i64;
            }
            total_geral += total;
            erro_geral += erros;
            duracao_geral += duracao;

            let assinatura = self
                .assinaturas
                .buscar_por_escritorio(escritorio.id)
                .await?;
            let plano_nome = assinatura.as_ref().and_then(|a| {
                planos
                    .iter()
                    .find(|p| p.id == a.plano_id)
                    .map(|p| p.nome.clone())
            });

            linhas.push(ConsumoEscritorioAdminDto {
                escritorio_id: escritorio.id,
                nome: escritorio.nome,
                cnpj: escritorio.cnpj,
                plano_nome,
                chamadas_limite: assinatura.as_ref().map(|a| a.chamadas_mes_limite),
                chamadas_usadas: total,
                chamadas_sucesso: sucesso,
                chamadas_erro: erros,
                taxa_erro: if total > 0 {
                    erros as f64 / total as f64
                } else {
                    0.0
                },
                duracao_media_ms: if total > 0 { duracao / total } else { 0 },
            });
        }

        linhas.sort_by_key(|a| std::cmp::Reverse(a.chamadas_usadas));
        let ranking = linhas
            .iter()
            .take(10)
            .map(|l| RankingConsumoDto {
                escritorio_id: l.escritorio_id,
                nome: l.nome.clone(),
                chamadas: l.chamadas_usadas,
            })
            .collect();
        let serie = por_dia
            .into_iter()
            .map(|(dia, (total, erros, duracao))| SerieConsumoDto {
                dia,
                chamadas: total,
                erros,
                duracao_media_ms: if total > 0 { duracao / total } else { 0 },
            })
            .collect();

        Ok(ConsumoAdminDto {
            inicio,
            fim,
            total_chamadas: total_geral,
            total_erros: erro_geral,
            taxa_erro: if total_geral > 0 {
                erro_geral as f64 / total_geral as f64
            } else {
                0.0
            },
            duracao_media_ms: if total_geral > 0 {
                duracao_geral / total_geral
            } else {
                0
            },
            escritorios: linhas,
            ranking,
            serie,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ConsumoAdminDto {
    pub inicio: DateTime<Utc>,
    pub fim: DateTime<Utc>,
    pub total_chamadas: i64,
    pub total_erros: i64,
    pub taxa_erro: f64,
    pub duracao_media_ms: i64,
    pub escritorios: Vec<ConsumoEscritorioAdminDto>,
    pub ranking: Vec<RankingConsumoDto>,
    pub serie: Vec<SerieConsumoDto>,
}

#[derive(Debug, Clone)]
pub struct ConsumoEscritorioAdminDto {
    pub escritorio_id: Uuid,
    pub nome: String,
    pub cnpj: String,
    pub plano_nome: Option<String>,
    pub chamadas_limite: Option<i64>,
    pub chamadas_usadas: i64,
    pub chamadas_sucesso: i64,
    pub chamadas_erro: i64,
    pub taxa_erro: f64,
    pub duracao_media_ms: i64,
}

#[derive(Debug, Clone)]
pub struct RankingConsumoDto {
    pub escritorio_id: Uuid,
    pub nome: String,
    pub chamadas: i64,
}

#[derive(Debug, Clone)]
pub struct SerieConsumoDto {
    pub dia: String,
    pub chamadas: i64,
    pub erros: i64,
    pub duracao_media_ms: i64,
}
