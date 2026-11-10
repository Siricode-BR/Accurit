//! Fase G — Implementação Diesel de `RepositorioChamadasSerpro`,
//! `RepositorioAgregadosConsumo`, `RepositorioPlanosEscritorio`,
//! `RepositorioAssinaturasEscritorio` e `RepositorioBloqueiosSerpro`.

use chrono::{DateTime, Utc};
use diesel::ExpressionMethods;
use diesel::QueryDsl;
use diesel::RunQueryDsl;
use diesel::prelude::*;
use uuid::Uuid;

use crate::domain::serpro::ambiente::AmbienteSerpro;
use crate::domain::serpro::medicao::entidades::{
    AgregadoConsumoSerpro, AssinaturaEscritorio, BloqueioSerpro, ChamadaSerpro,
    ComandoRegistrarBloqueio, ComandoRegistrarChamada, PlanoEscritorio,
};
use crate::domain::serpro::medicao::enums::{
    MotivoBloqueio, OrigemChamada, PeriodoAgregado, ResultadoChamada,
};
use crate::domain::serpro::medicao::erros::ErroMedicao;
use crate::domain::serpro::medicao::ports::{
    RepositorioAgregadosConsumo, RepositorioAssinaturasEscritorio, RepositorioBloqueiosSerpro,
    RepositorioChamadasSerpro, RepositorioPlanosEscritorio,
};
use crate::domain::serpro::produto::ProdutoSerpro;
use crate::infrastructure::persistence::pool::DbPool;

#[derive(Clone)]
pub struct RepositorioMedicaoDiesel {
    pool: DbPool,
}

impl RepositorioMedicaoDiesel {
    pub const fn novo(pool: DbPool) -> Self {
        Self { pool }
    }
}

// ===== Rows =====

#[derive(diesel::Queryable, diesel::Selectable)]
#[diesel(table_name = crate::infrastructure::persistence::schema::chamadas_serpro)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct ChamadaSerproRow {
    id: Uuid,
    escritorio_id: Uuid,
    produto: String,
    verbo: String,
    servico: Option<String>,
    ambiente: String,
    cliente_id: Option<Uuid>,
    usuario_id: Option<Uuid>,
    origem: String,
    resultado: String,
    status_http: Option<i32>,
    duracao_ms: i32,
    request_id: String,
    criado_em: DateTime<Utc>,
}

impl ChamadaSerproRow {
    fn para_entidade(self) -> ChamadaSerpro {
        ChamadaSerpro {
            id: self.id,
            escritorio_id: self.escritorio_id,
            produto: self
                .produto
                .parse()
                .unwrap_or(ProdutoSerpro::IntegraContador),
            verbo: self.verbo,
            servico: self.servico,
            ambiente: self.ambiente.parse().unwrap_or(AmbienteSerpro::Trial),
            cliente_id: self.cliente_id,
            usuario_id: self.usuario_id,
            origem: self.origem.parse().unwrap_or(OrigemChamada::Manual),
            resultado: self
                .resultado
                .parse()
                .unwrap_or(ResultadoChamada::Indeterminado),
            status_http: self.status_http.map(|v| v as u16),
            duracao_ms: self.duracao_ms as u64,
            request_id: self.request_id,
            criado_em: self.criado_em,
        }
    }
}

#[derive(diesel::Queryable, diesel::Selectable)]
#[diesel(table_name = crate::infrastructure::persistence::schema::agregados_consumo_serpro)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct AgregadoConsumoSerproRow {
    id: Uuid,
    escritorio_id: Uuid,
    produto: String,
    ambiente: String,
    periodo: String,
    referencia_periodo: DateTime<Utc>,
    total_chamadas: i32,
    chamadas_sucesso: i32,
    chamadas_4xx: i32,
    chamadas_5xx: i32,
    chamadas_timeout: i32,
    chamadas_indeterminado: i32,
    duracao_total_ms: i64,
    atualizado_em: DateTime<Utc>,
}

impl AgregadoConsumoSerproRow {
    fn para_entidade(self) -> AgregadoConsumoSerpro {
        AgregadoConsumoSerpro {
            id: self.id,
            escritorio_id: self.escritorio_id,
            produto: self
                .produto
                .parse()
                .unwrap_or(ProdutoSerpro::IntegraContador),
            ambiente: self.ambiente.parse().unwrap_or(AmbienteSerpro::Trial),
            periodo: self.periodo.parse().unwrap_or(PeriodoAgregado::Dia),
            referencia_periodo: self.referencia_periodo,
            total_chamadas: self.total_chamadas as i64,
            chamadas_sucesso: self.chamadas_sucesso as i64,
            chamadas_4xx: self.chamadas_4xx as i64,
            chamadas_5xx: self.chamadas_5xx as i64,
            chamadas_timeout: self.chamadas_timeout as i64,
            chamadas_indeterminado: self.chamadas_indeterminado as i64,
            duracao_total_ms: self.duracao_total_ms,
            atualizado_em: self.atualizado_em,
        }
    }
}

#[derive(diesel::Queryable, diesel::Selectable)]
#[diesel(table_name = crate::infrastructure::persistence::schema::planos_escritorio)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct PlanoEscritorioRow {
    id: Uuid,
    nome: String,
    descricao: Option<String>,
    chamadas_mes: i32,
    clientes_limite: i32,
    usuarios_limite: i32,
    preco_mensal_cents: i32,
    ativo: bool,
    criado_em: DateTime<Utc>,
    atualizado_em: DateTime<Utc>,
}

impl PlanoEscritorioRow {
    fn para_entidade(self) -> PlanoEscritorio {
        PlanoEscritorio {
            id: self.id,
            nome: self.nome,
            descricao: self.descricao,
            chamadas_mes: self.chamadas_mes as i64,
            clientes_limite: self.clientes_limite as i64,
            usuarios_limite: self.usuarios_limite as i64,
            preco_mensal_cents: self.preco_mensal_cents as i64,
            ativo: self.ativo,
            criado_em: self.criado_em,
            atualizado_em: self.atualizado_em,
        }
    }
}

#[derive(diesel::Queryable, diesel::Selectable)]
#[diesel(table_name = crate::infrastructure::persistence::schema::assinaturas_escritorio)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct AssinaturaEscritorioRow {
    id: Uuid,
    escritorio_id: Uuid,
    plano_id: Uuid,
    chamadas_mes_limite: i32,
    clientes_limite: i32,
    usuarios_limite: i32,
    bloqueio_ao_exceder: bool,
    ativo: bool,
    criado_em: DateTime<Utc>,
    atualizado_em: DateTime<Utc>,
}

impl AssinaturaEscritorioRow {
    fn para_entidade(self) -> AssinaturaEscritorio {
        AssinaturaEscritorio {
            id: self.id,
            escritorio_id: self.escritorio_id,
            plano_id: self.plano_id,
            chamadas_mes_limite: self.chamadas_mes_limite as i64,
            clientes_limite: self.clientes_limite as i64,
            usuarios_limite: self.usuarios_limite as i64,
            bloqueio_ao_exceder: self.bloqueio_ao_exceder,
            ativo: self.ativo,
            criado_em: self.criado_em,
            atualizado_em: self.atualizado_em,
        }
    }
}

#[derive(diesel::Queryable, diesel::Selectable)]
#[diesel(table_name = crate::infrastructure::persistence::schema::bloqueios_serpro)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct BloqueioSerproRow {
    id: Uuid,
    escritorio_id: Uuid,
    produto: String,
    motivo: String,
    contexto: Option<String>,
    criado_em: DateTime<Utc>,
}

impl BloqueioSerproRow {
    fn para_entidade(self) -> BloqueioSerpro {
        BloqueioSerpro {
            id: self.id,
            escritorio_id: self.escritorio_id,
            produto: self
                .produto
                .parse()
                .unwrap_or(ProdutoSerpro::IntegraContador),
            motivo: self
                .motivo
                .parse()
                .unwrap_or(MotivoBloqueio::ParametroInvalido),
            contexto: self.contexto,
            criado_em: self.criado_em,
        }
    }
}

// ===== Repositório =====

#[async_trait::async_trait]
impl RepositorioChamadasSerpro for RepositorioMedicaoDiesel {
    async fn registrar(
        &self,
        comando: &ComandoRegistrarChamada,
    ) -> Result<ChamadaSerpro, ErroMedicao> {
        let pool = self.pool.clone();
        let comando = comando.clone();
        tokio::task::spawn_blocking(move || -> Result<ChamadaSerpro, ErroMedicao> {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroMedicao::RepositorioIndisponivel)?;
            let id = Uuid::now_v7();
            let agora = Utc::now();
            let linha = diesel::insert_into(
                crate::infrastructure::persistence::schema::chamadas_serpro::table,
            )
            .values((
                crate::infrastructure::persistence::schema::chamadas_serpro::dsl::id.eq(id),
                crate::infrastructure::persistence::schema::chamadas_serpro::dsl::escritorio_id
                    .eq(comando.escritorio_id),
                crate::infrastructure::persistence::schema::chamadas_serpro::dsl::produto
                    .eq(comando.produto.as_str()),
                crate::infrastructure::persistence::schema::chamadas_serpro::dsl::verbo
                    .eq(comando.verbo.as_str()),
                crate::infrastructure::persistence::schema::chamadas_serpro::dsl::servico
                    .eq(comando.servico.as_deref()),
                crate::infrastructure::persistence::schema::chamadas_serpro::dsl::ambiente
                    .eq(comando.ambiente.as_str()),
                crate::infrastructure::persistence::schema::chamadas_serpro::dsl::cliente_id
                    .eq(comando.cliente_id),
                crate::infrastructure::persistence::schema::chamadas_serpro::dsl::usuario_id
                    .eq(comando.usuario_id),
                crate::infrastructure::persistence::schema::chamadas_serpro::dsl::origem
                    .eq(comando.origem.as_str()),
                crate::infrastructure::persistence::schema::chamadas_serpro::dsl::resultado
                    .eq(comando.resultado.as_str()),
                crate::infrastructure::persistence::schema::chamadas_serpro::dsl::status_http
                    .eq(comando.status_http.map(|v| v as i32)),
                crate::infrastructure::persistence::schema::chamadas_serpro::dsl::duracao_ms
                    .eq(comando.duracao_ms as i32),
                crate::infrastructure::persistence::schema::chamadas_serpro::dsl::request_id
                    .eq(comando.request_id.as_str()),
                crate::infrastructure::persistence::schema::chamadas_serpro::dsl::criado_em
                    .eq(agora),
            ))
            .get_result::<ChamadaSerproRow>(&mut conexao)
            .map_err(|_| ErroMedicao::RepositorioIndisponivel)?
            .para_entidade();
            Ok(linha)
        })
        .await
        .map_err(|_| ErroMedicao::RepositorioIndisponivel)?
    }

    async fn listar_por_escritorio(
        &self,
        escritorio_id: Uuid,
        limite: i64,
    ) -> Result<Vec<ChamadaSerpro>, ErroMedicao> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || -> Result<Vec<ChamadaSerpro>, ErroMedicao> {
            let mut conexao = pool.get().map_err(|_| ErroMedicao::RepositorioIndisponivel)?;
            let linhas: Vec<ChamadaSerproRow> = crate::infrastructure::persistence::schema::chamadas_serpro::table
                .filter(
                    crate::infrastructure::persistence::schema::chamadas_serpro::dsl::escritorio_id
                        .eq(escritorio_id),
                )
                .order(
                    crate::infrastructure::persistence::schema::chamadas_serpro::dsl::criado_em
                        .desc(),
                )
                .limit(limite)
                .load(&mut conexao)
                .map_err(|_| ErroMedicao::RepositorioIndisponivel)?;
            Ok(linhas.into_iter().map(|r| r.para_entidade()).collect())
        })
        .await
        .map_err(|_| ErroMedicao::RepositorioIndisponivel)?
    }

    async fn listar_por_periodo(
        &self,
        escritorio_id: Uuid,
        produto: Option<String>,
        ambiente: Option<String>,
        periodo: Option<String>,
        referencia_inicio: DateTime<Utc>,
        referencia_fim: DateTime<Utc>,
    ) -> Result<Vec<ChamadaSerpro>, ErroMedicao> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || -> Result<Vec<ChamadaSerpro>, ErroMedicao> {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroMedicao::RepositorioIndisponivel)?;
            let mut consulta = crate::infrastructure::persistence::schema::chamadas_serpro::table
                .filter(
                    crate::infrastructure::persistence::schema::chamadas_serpro::dsl::escritorio_id
                        .eq(escritorio_id),
                )
                .filter(
                    crate::infrastructure::persistence::schema::chamadas_serpro::dsl::criado_em
                        .ge(referencia_inicio),
                )
                .filter(
                    crate::infrastructure::persistence::schema::chamadas_serpro::dsl::criado_em
                        .lt(referencia_fim),
                )
                .into_boxed();
            if let Some(p) = produto {
                consulta = consulta.filter(
                    crate::infrastructure::persistence::schema::chamadas_serpro::dsl::produto.eq(p),
                );
            }
            if let Some(a) = ambiente {
                consulta = consulta.filter(
                    crate::infrastructure::persistence::schema::chamadas_serpro::dsl::ambiente
                        .eq(a),
                );
            }
            if let Some(p) = periodo {
                consulta = consulta.filter(
                    crate::infrastructure::persistence::schema::chamadas_serpro::dsl::origem.eq(p),
                );
            }
            let linhas: Vec<ChamadaSerproRow> = consulta
                .order(
                    crate::infrastructure::persistence::schema::chamadas_serpro::dsl::criado_em
                        .desc(),
                )
                .load(&mut conexao)
                .map_err(|_| ErroMedicao::RepositorioIndisponivel)?;
            Ok(linhas.into_iter().map(|r| r.para_entidade()).collect())
        })
        .await
        .map_err(|_| ErroMedicao::RepositorioIndisponivel)?
    }
}

#[async_trait::async_trait]
impl RepositorioAgregadosConsumo for RepositorioMedicaoDiesel {
    async fn upsert(
        &self,
        agregado: &AgregadoConsumoSerpro,
    ) -> Result<AgregadoConsumoSerpro, ErroMedicao> {
        let pool = self.pool.clone();
        let agregado = agregado.clone();
        tokio::task::spawn_blocking(move || -> Result<AgregadoConsumoSerpro, ErroMedicao> {
            let mut conexao = pool.get().map_err(|_| ErroMedicao::RepositorioIndisponivel)?;
            let existente: Option<Uuid> = crate::infrastructure::persistence::schema::agregados_consumo_serpro::table
                .filter(
                    crate::infrastructure::persistence::schema::agregados_consumo_serpro::dsl::escritorio_id
                        .eq(agregado.escritorio_id),
                )
                .filter(
                    crate::infrastructure::persistence::schema::agregados_consumo_serpro::dsl::produto
                        .eq(agregado.produto.as_str()),
                )
                .filter(
                    crate::infrastructure::persistence::schema::agregados_consumo_serpro::dsl::ambiente
                        .eq(agregado.ambiente.as_str()),
                )
                .filter(
                    crate::infrastructure::persistence::schema::agregados_consumo_serpro::dsl::periodo
                        .eq(agregado.periodo.as_str()),
                )
                .filter(
                    crate::infrastructure::persistence::schema::agregados_consumo_serpro::dsl::referencia_periodo
                        .eq(agregado.referencia_periodo),
                )
                .select(crate::infrastructure::persistence::schema::agregados_consumo_serpro::dsl::id)
                .first(&mut conexao)
                .optional()
                .map_err(|_| ErroMedicao::RepositorioIndisponivel)?;

            match existente {
                Some(id) => {
                    diesel::update(
                        crate::infrastructure::persistence::schema::agregados_consumo_serpro::table
                            .filter(
                                crate::infrastructure::persistence::schema::agregados_consumo_serpro::dsl::id
                                    .eq(id),
                            ),
                    )
                    .set((
                        crate::infrastructure::persistence::schema::agregados_consumo_serpro::dsl::total_chamadas
                            .eq(agregado.total_chamadas as i32),
                        crate::infrastructure::persistence::schema::agregados_consumo_serpro::dsl::chamadas_sucesso
                            .eq(agregado.chamadas_sucesso as i32),
                        crate::infrastructure::persistence::schema::agregados_consumo_serpro::dsl::chamadas_4xx
                            .eq(agregado.chamadas_4xx as i32),
                        crate::infrastructure::persistence::schema::agregados_consumo_serpro::dsl::chamadas_5xx
                            .eq(agregado.chamadas_5xx as i32),
                        crate::infrastructure::persistence::schema::agregados_consumo_serpro::dsl::chamadas_timeout
                            .eq(agregado.chamadas_timeout as i32),
                        crate::infrastructure::persistence::schema::agregados_consumo_serpro::dsl::chamadas_indeterminado
                            .eq(agregado.chamadas_indeterminado as i32),
                        crate::infrastructure::persistence::schema::agregados_consumo_serpro::dsl::duracao_total_ms
                            .eq(agregado.duracao_total_ms),
                        crate::infrastructure::persistence::schema::agregados_consumo_serpro::dsl::atualizado_em
                            .eq(Utc::now()),
                    ))
                    .execute(&mut conexao)
                    .map_err(|_| ErroMedicao::RepositorioIndisponivel)?;
                    Ok(agregado)
                }
                None => {
                    let linha = diesel::insert_into(
                        crate::infrastructure::persistence::schema::agregados_consumo_serpro::table,
                    )
                    .values((
                        crate::infrastructure::persistence::schema::agregados_consumo_serpro::dsl::id
                            .eq(Uuid::now_v7()),
                        crate::infrastructure::persistence::schema::agregados_consumo_serpro::dsl::escritorio_id
                            .eq(agregado.escritorio_id),
                        crate::infrastructure::persistence::schema::agregados_consumo_serpro::dsl::produto
                            .eq(agregado.produto.as_str()),
                        crate::infrastructure::persistence::schema::agregados_consumo_serpro::dsl::ambiente
                            .eq(agregado.ambiente.as_str()),
                        crate::infrastructure::persistence::schema::agregados_consumo_serpro::dsl::periodo
                            .eq(agregado.periodo.as_str()),
                        crate::infrastructure::persistence::schema::agregados_consumo_serpro::dsl::referencia_periodo
                            .eq(agregado.referencia_periodo),
                        crate::infrastructure::persistence::schema::agregados_consumo_serpro::dsl::total_chamadas
                            .eq(agregado.total_chamadas as i32),
                        crate::infrastructure::persistence::schema::agregados_consumo_serpro::dsl::chamadas_sucesso
                            .eq(agregado.chamadas_sucesso as i32),
                        crate::infrastructure::persistence::schema::agregados_consumo_serpro::dsl::chamadas_4xx
                            .eq(agregado.chamadas_4xx as i32),
                        crate::infrastructure::persistence::schema::agregados_consumo_serpro::dsl::chamadas_5xx
                            .eq(agregado.chamadas_5xx as i32),
                        crate::infrastructure::persistence::schema::agregados_consumo_serpro::dsl::chamadas_timeout
                            .eq(agregado.chamadas_timeout as i32),
                        crate::infrastructure::persistence::schema::agregados_consumo_serpro::dsl::chamadas_indeterminado
                            .eq(agregado.chamadas_indeterminado as i32),
                        crate::infrastructure::persistence::schema::agregados_consumo_serpro::dsl::duracao_total_ms
                            .eq(agregado.duracao_total_ms),
                        crate::infrastructure::persistence::schema::agregados_consumo_serpro::dsl::atualizado_em
                            .eq(agregado.atualizado_em),
                        crate::infrastructure::persistence::schema::agregados_consumo_serpro::dsl::atualizado_em
                            .eq(Utc::now()),
                    ))
                    .get_result::<AgregadoConsumoSerproRow>(&mut conexao)
                    .map_err(|_| ErroMedicao::RepositorioIndisponivel)?
                    .para_entidade();
                    Ok(linha)
                }
            }
        })
        .await
        .map_err(|_| ErroMedicao::RepositorioIndisponivel)?
    }

    async fn listar_por_escritorio(
        &self,
        escritorio_id: Uuid,
        periodo: Option<String>,
        referencia_inicio: Option<DateTime<Utc>>,
        referencia_fim: Option<DateTime<Utc>>,
    ) -> Result<Vec<AgregadoConsumoSerpro>, ErroMedicao> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || -> Result<Vec<AgregadoConsumoSerpro>, ErroMedicao> {
            let mut conexao = pool.get().map_err(|_| ErroMedicao::RepositorioIndisponivel)?;
            let mut consulta = crate::infrastructure::persistence::schema::agregados_consumo_serpro::table
                .filter(
                    crate::infrastructure::persistence::schema::agregados_consumo_serpro::dsl::escritorio_id
                        .eq(escritorio_id),
                )
                .into_boxed();
            if let Some(p) = &periodo {
                consulta = consulta.filter(
                    crate::infrastructure::persistence::schema::agregados_consumo_serpro::dsl::periodo
                        .eq(p),
                );
            }
            if let Some(ri) = referencia_inicio {
                consulta = consulta.filter(
                    crate::infrastructure::persistence::schema::agregados_consumo_serpro::dsl::referencia_periodo
                        .ge(ri),
                );
            }
            if let Some(rf) = referencia_fim {
                consulta = consulta.filter(
                    crate::infrastructure::persistence::schema::agregados_consumo_serpro::dsl::referencia_periodo
                        .lt(rf),
                );
            }
            let linhas: Vec<AgregadoConsumoSerproRow> = consulta
                .order(
                    crate::infrastructure::persistence::schema::agregados_consumo_serpro::dsl::referencia_periodo
                        .desc(),
                )
                .load(&mut conexao)
                .map_err(|_| ErroMedicao::RepositorioIndisponivel)?;
            Ok(linhas.into_iter().map(|r| r.para_entidade()).collect())
        })
        .await
        .map_err(|_| ErroMedicao::RepositorioIndisponivel)?
    }
}

#[async_trait::async_trait]
impl RepositorioPlanosEscritorio for RepositorioMedicaoDiesel {
    async fn criar(&self, plano: &PlanoEscritorio) -> Result<PlanoEscritorio, ErroMedicao> {
        let pool = self.pool.clone();
        let plano = plano.clone();
        tokio::task::spawn_blocking(move || -> Result<PlanoEscritorio, ErroMedicao> {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroMedicao::RepositorioIndisponivel)?;
            let linha = diesel::insert_into(
                crate::infrastructure::persistence::schema::planos_escritorio::table,
            )
            .values((
                crate::infrastructure::persistence::schema::planos_escritorio::dsl::id
                    .eq(Uuid::now_v7()),
                crate::infrastructure::persistence::schema::planos_escritorio::dsl::nome
                    .eq(plano.nome),
                crate::infrastructure::persistence::schema::planos_escritorio::dsl::descricao
                    .eq(plano.descricao),
                crate::infrastructure::persistence::schema::planos_escritorio::dsl::chamadas_mes
                    .eq(plano.chamadas_mes as i32),
                crate::infrastructure::persistence::schema::planos_escritorio::dsl::clientes_limite
                    .eq(plano.clientes_limite as i32),
                crate::infrastructure::persistence::schema::planos_escritorio::dsl::usuarios_limite
                    .eq(plano.usuarios_limite as i32),
                crate::infrastructure::persistence::schema::planos_escritorio::dsl::preco_mensal_cents
                    .eq(plano.preco_mensal_cents as i32),
                crate::infrastructure::persistence::schema::planos_escritorio::dsl::ativo
                    .eq(plano.ativo),
                crate::infrastructure::persistence::schema::planos_escritorio::dsl::criado_em
                    .eq(Utc::now()),
                crate::infrastructure::persistence::schema::planos_escritorio::dsl::atualizado_em
                    .eq(Utc::now()),
            ))
            .get_result::<PlanoEscritorioRow>(&mut conexao)
            .map_err(|_| ErroMedicao::RepositorioIndisponivel)?
            .para_entidade();
            Ok(linha)
        })
        .await
        .map_err(|_| ErroMedicao::RepositorioIndisponivel)?
    }

    async fn listar_ativos(&self) -> Result<Vec<PlanoEscritorio>, ErroMedicao> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || -> Result<Vec<PlanoEscritorio>, ErroMedicao> {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroMedicao::RepositorioIndisponivel)?;
            let linhas: Vec<PlanoEscritorioRow> =
                crate::infrastructure::persistence::schema::planos_escritorio::table
                    .filter(
                        crate::infrastructure::persistence::schema::planos_escritorio::dsl::ativo
                            .eq(true),
                    )
                    .load(&mut conexao)
                    .map_err(|_| ErroMedicao::RepositorioIndisponivel)?;
            Ok(linhas.into_iter().map(|r| r.para_entidade()).collect())
        })
        .await
        .map_err(|_| ErroMedicao::RepositorioIndisponivel)?
    }

    async fn listar_todos(&self) -> Result<Vec<PlanoEscritorio>, ErroMedicao> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || -> Result<Vec<PlanoEscritorio>, ErroMedicao> {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroMedicao::RepositorioIndisponivel)?;
            let linhas: Vec<PlanoEscritorioRow> =
                crate::infrastructure::persistence::schema::planos_escritorio::table
                    .order(
                        crate::infrastructure::persistence::schema::planos_escritorio::dsl::nome
                            .asc(),
                    )
                    .load(&mut conexao)
                    .map_err(|_| ErroMedicao::RepositorioIndisponivel)?;
            Ok(linhas.into_iter().map(|r| r.para_entidade()).collect())
        })
        .await
        .map_err(|_| ErroMedicao::RepositorioIndisponivel)?
    }

    async fn buscar_por_id(&self, id: Uuid) -> Result<Option<PlanoEscritorio>, ErroMedicao> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || -> Result<Option<PlanoEscritorio>, ErroMedicao> {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroMedicao::RepositorioIndisponivel)?;
            let linha: Option<PlanoEscritorioRow> =
                crate::infrastructure::persistence::schema::planos_escritorio::table
                    .filter(
                        crate::infrastructure::persistence::schema::planos_escritorio::dsl::id
                            .eq(id),
                    )
                    .first(&mut conexao)
                    .optional()
                    .map_err(|_| ErroMedicao::RepositorioIndisponivel)?;
            Ok(linha.map(|r| r.para_entidade()))
        })
        .await
        .map_err(|_| ErroMedicao::RepositorioIndisponivel)?
    }
}

#[async_trait::async_trait]
impl RepositorioAssinaturasEscritorio for RepositorioMedicaoDiesel {
    async fn buscar_por_escritorio(
        &self,
        escritorio_id: Uuid,
    ) -> Result<Option<AssinaturaEscritorio>, ErroMedicao> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || -> Result<Option<AssinaturaEscritorio>, ErroMedicao> {
            let mut conexao = pool.get().map_err(|_| ErroMedicao::RepositorioIndisponivel)?;
            let linha: Option<AssinaturaEscritorioRow> = crate::infrastructure::persistence::schema::assinaturas_escritorio::table
                .filter(crate::infrastructure::persistence::schema::assinaturas_escritorio::dsl::escritorio_id.eq(escritorio_id))
                .first(&mut conexao)
                .optional()
                .map_err(|_| ErroMedicao::RepositorioIndisponivel)?;
            Ok(linha.map(|r| r.para_entidade()))
        })
        .await
        .map_err(|_| ErroMedicao::RepositorioIndisponivel)?
    }

    async fn criar(
        &self,
        assinatura: &AssinaturaEscritorio,
    ) -> Result<AssinaturaEscritorio, ErroMedicao> {
        let pool = self.pool.clone();
        let assinatura = assinatura.clone();
        tokio::task::spawn_blocking(move || -> Result<AssinaturaEscritorio, ErroMedicao> {
            let mut conexao = pool.get().map_err(|_| ErroMedicao::RepositorioIndisponivel)?;
            let linha = diesel::insert_into(
                crate::infrastructure::persistence::schema::assinaturas_escritorio::table,
            )
            .values((
                crate::infrastructure::persistence::schema::assinaturas_escritorio::dsl::id
                    .eq(Uuid::now_v7()),
                crate::infrastructure::persistence::schema::assinaturas_escritorio::dsl::escritorio_id
                    .eq(assinatura.escritorio_id),
                crate::infrastructure::persistence::schema::assinaturas_escritorio::dsl::plano_id
                    .eq(assinatura.plano_id),
                crate::infrastructure::persistence::schema::assinaturas_escritorio::dsl::chamadas_mes_limite
                    .eq(assinatura.chamadas_mes_limite as i32),
                crate::infrastructure::persistence::schema::assinaturas_escritorio::dsl::bloqueio_ao_exceder
                    .eq(assinatura.bloqueio_ao_exceder),
                crate::infrastructure::persistence::schema::assinaturas_escritorio::dsl::ativo
                    .eq(assinatura.ativo),
                crate::infrastructure::persistence::schema::assinaturas_escritorio::dsl::criado_em
                    .eq(assinatura.criado_em),
                crate::infrastructure::persistence::schema::assinaturas_escritorio::dsl::atualizado_em
                    .eq(Utc::now()),
            ))
            .get_result::<AssinaturaEscritorioRow>(&mut conexao)
            .map_err(|_| ErroMedicao::RepositorioIndisponivel)?
            .para_entidade();
            Ok(linha)
        })
        .await
        .map_err(|_| ErroMedicao::RepositorioIndisponivel)?
    }

    async fn atualizar(
        &self,
        assinatura: &AssinaturaEscritorio,
    ) -> Result<AssinaturaEscritorio, ErroMedicao> {
        let pool = self.pool.clone();
        let assinatura = assinatura.clone();
        tokio::task::spawn_blocking(move || -> Result<AssinaturaEscritorio, ErroMedicao> {
            let mut conexao = pool.get().map_err(|_| ErroMedicao::RepositorioIndisponivel)?;
            let _linhas = diesel::update(
                crate::infrastructure::persistence::schema::assinaturas_escritorio::table
                    .filter(crate::infrastructure::persistence::schema::assinaturas_escritorio::dsl::id
                        .eq(assinatura.id)),
            )
            .set((
                crate::infrastructure::persistence::schema::assinaturas_escritorio::dsl::plano_id
                    .eq(assinatura.plano_id),
                crate::infrastructure::persistence::schema::assinaturas_escritorio::dsl::chamadas_mes_limite
                    .eq(assinatura.chamadas_mes_limite as i32),
                crate::infrastructure::persistence::schema::assinaturas_escritorio::dsl::clientes_limite
                    .eq(assinatura.clientes_limite as i32),
                crate::infrastructure::persistence::schema::assinaturas_escritorio::dsl::usuarios_limite
                    .eq(assinatura.usuarios_limite as i32),
                crate::infrastructure::persistence::schema::assinaturas_escritorio::dsl::bloqueio_ao_exceder
                    .eq(assinatura.bloqueio_ao_exceder),
                crate::infrastructure::persistence::schema::assinaturas_escritorio::dsl::ativo
                    .eq(assinatura.ativo),
                crate::infrastructure::persistence::schema::assinaturas_escritorio::dsl::atualizado_em
                    .eq(Utc::now()),
            ))
            .execute(&mut conexao)
            .map_err(|_| ErroMedicao::RepositorioIndisponivel)?;
            if _linhas != 1 {
                return Err(ErroMedicao::RepositorioIndisponivel);
            }
            let linha: Option<AssinaturaEscritorioRow> = crate::infrastructure::persistence::schema::assinaturas_escritorio::table
                .filter(crate::infrastructure::persistence::schema::assinaturas_escritorio::dsl::id.eq(assinatura.id))
                .first(&mut conexao)
                .optional()
                .map_err(|_| ErroMedicao::RepositorioIndisponivel)?;
            linha.map(|r| r.para_entidade()).ok_or(ErroMedicao::RepositorioIndisponivel)
        })
        .await
        .map_err(|_| ErroMedicao::RepositorioIndisponivel)?
    }
}

#[async_trait::async_trait]
impl RepositorioBloqueiosSerpro for RepositorioMedicaoDiesel {
    async fn registrar(
        &self,
        comando: &ComandoRegistrarBloqueio,
    ) -> Result<BloqueioSerpro, ErroMedicao> {
        let pool = self.pool.clone();
        let comando = comando.clone();
        tokio::task::spawn_blocking(move || -> Result<BloqueioSerpro, ErroMedicao> {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroMedicao::RepositorioIndisponivel)?;
            let linha = diesel::insert_into(
                crate::infrastructure::persistence::schema::bloqueios_serpro::table,
            )
            .values((
                crate::infrastructure::persistence::schema::bloqueios_serpro::dsl::id
                    .eq(Uuid::now_v7()),
                crate::infrastructure::persistence::schema::bloqueios_serpro::dsl::escritorio_id
                    .eq(comando.escritorio_id),
                crate::infrastructure::persistence::schema::bloqueios_serpro::dsl::produto
                    .eq(comando.produto.as_str()),
                crate::infrastructure::persistence::schema::bloqueios_serpro::dsl::motivo
                    .eq(comando.motivo.as_str()),
                crate::infrastructure::persistence::schema::bloqueios_serpro::dsl::contexto
                    .eq(comando.contexto.as_deref()),
                crate::infrastructure::persistence::schema::bloqueios_serpro::dsl::criado_em
                    .eq(Utc::now()),
            ))
            .get_result::<BloqueioSerproRow>(&mut conexao)
            .map_err(|_| ErroMedicao::RepositorioIndisponivel)?
            .para_entidade();
            Ok(linha)
        })
        .await
        .map_err(|_| ErroMedicao::RepositorioIndisponivel)?
    }

    async fn listar_por_escritorio(
        &self,
        escritorio_id: Uuid,
        limite: i64,
    ) -> Result<Vec<BloqueioSerpro>, ErroMedicao> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || -> Result<Vec<BloqueioSerpro>, ErroMedicao> {
            let mut conexao = pool.get().map_err(|_| ErroMedicao::RepositorioIndisponivel)?;
            let linhas: Vec<BloqueioSerproRow> = crate::infrastructure::persistence::schema::bloqueios_serpro::table
                .filter(
                    crate::infrastructure::persistence::schema::bloqueios_serpro::dsl::escritorio_id
                        .eq(escritorio_id),
                )
                .order(
                    crate::infrastructure::persistence::schema::bloqueios_serpro::dsl::criado_em
                        .desc(),
                )
                .limit(limite)
                .load(&mut conexao)
                .map_err(|_| ErroMedicao::RepositorioIndisponivel)?;
            Ok(linhas.into_iter().map(|r| r.para_entidade()).collect())
        })
        .await
        .map_err(|_| ErroMedicao::RepositorioIndisponivel)?
    }
}

#[async_trait::async_trait]
impl crate::domain::serpro::medicao::ports::RepositorioEscritoriosConsumo
    for RepositorioMedicaoDiesel
{
    async fn listar(
        &self,
    ) -> Result<Vec<crate::domain::serpro::medicao::ports::EscritorioConsumo>, ErroMedicao> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroMedicao::RepositorioIndisponivel)?;
            use crate::infrastructure::persistence::schema::escritorios::dsl as e;
            let linhas: Vec<(Uuid, String, String)> = e::escritorios
                .select((e::id, e::nome, e::cnpj))
                .order(e::nome.asc())
                .load(&mut conexao)
                .map_err(|_| ErroMedicao::RepositorioIndisponivel)?;
            Ok(linhas
                .into_iter()
                .map(
                    |(id, nome, cnpj)| crate::domain::serpro::medicao::ports::EscritorioConsumo {
                        id,
                        nome,
                        cnpj,
                    },
                )
                .collect())
        })
        .await
        .map_err(|_| ErroMedicao::RepositorioIndisponivel)?
    }
}

#[async_trait::async_trait]
impl crate::application::serpro::medicao::use_cases::NotificadorConsumo
    for RepositorioMedicaoDiesel
{
    async fn notificar_limite(
        &self,
        escritorio_id: Uuid,
        usuario_id: Option<Uuid>,
        percentual: f64,
        limite: i64,
    ) -> Result<(), ErroMedicao> {
        let Some(usuario_id) = usuario_id else {
            return Ok(());
        };
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroMedicao::RepositorioIndisponivel)?;
            let titulo = if percentual >= 100.0 {
                "Limite mensal de chamadas SERPRO atingido"
            } else {
                "Consumo de chamadas SERPRO acima de 80%"
            };
            let corpo = format!(
                "O escritório utilizou {percentual:.0}% do limite de {limite} chamadas SERPRO no mês."
            );
            diesel::sql_query(
                "INSERT INTO notificacoes \
                 (id, usuario_id, escritorio_id, canal, tipo, titulo, corpo, lida) \
                 VALUES ($1, $2, $3, 'in_app', 'limite_consumo_serpro', $4, $5, FALSE)",
            )
            .bind::<diesel::sql_types::Uuid, _>(Uuid::now_v7())
            .bind::<diesel::sql_types::Uuid, _>(usuario_id)
            .bind::<diesel::sql_types::Uuid, _>(escritorio_id)
            .bind::<diesel::sql_types::Text, _>(titulo)
            .bind::<diesel::sql_types::Text, _>(corpo)
            .execute(&mut conexao)
            .map_err(|_| ErroMedicao::RepositorioIndisponivel)?;
            Ok(())
        })
        .await
        .map_err(|_| ErroMedicao::RepositorioIndisponivel)?
    }
}

#[async_trait::async_trait]
impl crate::application::serpro::medicao::use_cases::LimitesPlano for RepositorioMedicaoDiesel {
    async fn pode_criar_cliente(&self, escritorio_id: Uuid) -> Result<bool, ErroMedicao> {
        let Some(assinatura) =
            RepositorioAssinaturasEscritorio::buscar_por_escritorio(self, escritorio_id).await?
        else {
            return Ok(true);
        };
        let pool = self.pool.clone();
        let total = tokio::task::spawn_blocking(move || {
            use crate::infrastructure::persistence::schema::clientes::dsl as c;
            let mut conexao = pool
                .get()
                .map_err(|_| ErroMedicao::RepositorioIndisponivel)?;
            c::clientes
                .filter(c::escritorio_id.eq(escritorio_id))
                .count()
                .get_result::<i64>(&mut conexao)
                .map_err(|_| ErroMedicao::RepositorioIndisponivel)
        })
        .await
        .map_err(|_| ErroMedicao::RepositorioIndisponivel)??;
        Ok(total < assinatura.clientes_limite)
    }

    async fn pode_criar_usuario(&self, escritorio_id: Uuid) -> Result<bool, ErroMedicao> {
        let Some(assinatura) =
            RepositorioAssinaturasEscritorio::buscar_por_escritorio(self, escritorio_id).await?
        else {
            return Ok(true);
        };
        let pool = self.pool.clone();
        let total = tokio::task::spawn_blocking(move || {
            use crate::infrastructure::persistence::schema::usuarios::dsl as u;
            let mut conexao = pool
                .get()
                .map_err(|_| ErroMedicao::RepositorioIndisponivel)?;
            u::usuarios
                .filter(u::escritorio_id.eq(escritorio_id))
                .count()
                .get_result::<i64>(&mut conexao)
                .map_err(|_| ErroMedicao::RepositorioIndisponivel)
        })
        .await
        .map_err(|_| ErroMedicao::RepositorioIndisponivel)??;
        Ok(total < assinatura.usuarios_limite)
    }
}
