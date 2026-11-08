//! Repositório Diesel de trabalhos agendados.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Bool, Nullable, Text, Timestamptz, Uuid as SqlUuid};
use uuid::Uuid;

use crate::domain::trabalhos::entidades::{NovoTrabalhoAgendado, TrabalhoAgendado};
use crate::domain::trabalhos::enums::TipoTrabalho;
use crate::domain::trabalhos::erros::ErroTrabalho;
use crate::domain::trabalhos::ports::RepositorioTrabalhosAgendados;
use crate::infrastructure::persistence::pool::DbPool;

#[derive(Clone)]
pub struct TrabalhosAgendadosDiesel {
    pool: DbPool,
}

impl TrabalhosAgendadosDiesel {
    pub const fn novo(pool: DbPool) -> Self {
        Self { pool }
    }

    #[allow(dead_code)]
    fn conexao(
        &self,
    ) -> Result<
        diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>,
        ErroTrabalho,
    > {
        self.pool
            .get()
            .map_err(|_| ErroTrabalho::Persistencia("pool indisponível".into()))
    }
}

#[derive(QueryableByName)]
struct TrabalhoRow {
    #[diesel(sql_type = SqlUuid)]
    id: Uuid,
    #[diesel(sql_type = SqlUuid)]
    escritorio_id: Uuid,
    #[diesel(sql_type = Text)]
    nome: String,
    #[diesel(sql_type = Text)]
    tipo: String,
    #[diesel(sql_type = Text)]
    produto: String,
    #[diesel(sql_type = Text)]
    ambiente: String,
    #[diesel(sql_type = Text)]
    cron_expression: String,
    #[diesel(sql_type = Bool)]
    habilitado: bool,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    proxima_execucao: Option<DateTime<Utc>>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    ultima_execucao: Option<DateTime<Utc>>,
    #[diesel(sql_type = Timestamptz)]
    criado_em: DateTime<Utc>,
    #[diesel(sql_type = Timestamptz)]
    atualizado_em: DateTime<Utc>,
}

impl TrabalhoRow {
    fn para_entidade(self) -> Result<TrabalhoAgendado, ErroTrabalho> {
        let tipo = match self.tipo.as_str() {
            "consulta_divida_ativa" => TipoTrabalho::ConsultaDividaAtiva,
            "validar_procuracao" => TipoTrabalho::ValidarProcuracao,
            "consulta_integra_contador" => TipoTrabalho::ConsultaIntegraContador,
            _ => return Err(ErroTrabalho::Persistencia("tipo inválido".into())),
        };
        Ok(TrabalhoAgendado {
            id: self.id,
            escritorio_id: self.escritorio_id,
            nome: self.nome,
            tipo,
            produto: self.produto,
            ambiente: self.ambiente,
            cron_expression: self.cron_expression,
            habilitado: self.habilitado,
            proxima_execucao: self.proxima_execucao,
            ultima_execucao: self.ultima_execucao,
            criado_em: self.criado_em,
            atualizado_em: self.atualizado_em,
        })
    }
}

#[async_trait]
impl RepositorioTrabalhosAgendados for TrabalhosAgendadosDiesel {
    async fn criar(
        &self,
        trabalho: &NovoTrabalhoAgendado,
    ) -> Result<TrabalhoAgendado, ErroTrabalho> {
        let pool = self.pool.clone();
        let novo = trabalho.clone();
        tokio::task::spawn_blocking(move || {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroTrabalho::Persistencia("pool".into()))?;
            let linha = diesel::sql_query(
                "INSERT INTO trabalhos_agendados \
                 (id, escritorio_id, nome, tipo, produto, ambiente, cron_expression, habilitado) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, TRUE) \
                 RETURNING id, escritorio_id, nome, tipo, produto, ambiente, cron_expression, \
                          habilitado, proxima_execucao, ultima_execucao, criado_em, atualizado_em",
            )
            .bind::<SqlUuid, _>(Uuid::now_v7())
            .bind::<SqlUuid, _>(novo.escritorio_id)
            .bind::<Text, _>(novo.nome)
            .bind::<Text, _>(novo.tipo.as_str())
            .bind::<Text, _>(novo.produto)
            .bind::<Text, _>(novo.ambiente)
            .bind::<Text, _>(novo.cron_expression)
            .get_result::<TrabalhoRow>(&mut conexao)
            .map_err(|_| ErroTrabalho::Persistencia("inserir trabalho".into()))?
            .para_entidade()?;
            Ok(linha)
        })
        .await
        .map_err(|_| ErroTrabalho::Persistencia("spawn_blocking".into()))?
    }

    async fn buscar_por_id(&self, id: Uuid) -> Result<Option<TrabalhoAgendado>, ErroTrabalho> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroTrabalho::Persistencia("pool".into()))?;
            let linha = diesel::sql_query(
                "SELECT id, escritorio_id, nome, tipo, produto, ambiente, cron_expression, \
                         habilitado, proxima_execucao, ultima_execucao, criado_em, atualizado_em \
                  FROM trabalhos_agendados \
                  WHERE id = $1",
            )
            .bind::<SqlUuid, _>(id)
            .get_result::<TrabalhoRow>(&mut conexao)
            .optional()
            .map_err(|_| ErroTrabalho::Persistencia("buscar trabalho".into()))?
            .map(|row| row.para_entidade())
            .transpose()?;
            Ok(linha)
        })
        .await
        .map_err(|_| ErroTrabalho::Persistencia("spawn_blocking".into()))?
    }

    async fn listar_por_escritorio(
        &self,
        escritorio_id: Uuid,
    ) -> Result<Vec<TrabalhoAgendado>, ErroTrabalho> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroTrabalho::Persistencia("pool".into()))?;
            let linhas = diesel::sql_query(
                "SELECT id, escritorio_id, nome, tipo, produto, ambiente, cron_expression, \
                         habilitado, proxima_execucao, ultima_execucao, criado_em, atualizado_em \
                  FROM trabalhos_agendados \
                  WHERE escritorio_id = $1 \
                  ORDER BY criado_em DESC",
            )
            .bind::<SqlUuid, _>(escritorio_id)
            .load::<TrabalhoRow>(&mut conexao)
            .map_err(|_| ErroTrabalho::Persistencia("listar trabalhos".into()))?
            .into_iter()
            .map(|row| row.para_entidade())
            .collect::<Result<Vec<_>, _>>()?;
            Ok(linhas)
        })
        .await
        .map_err(|_| ErroTrabalho::Persistencia("spawn_blocking".into()))?
    }

    async fn listar_devidos(
        &self,
        agora: DateTime<Utc>,
    ) -> Result<Vec<TrabalhoAgendado>, ErroTrabalho> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroTrabalho::Persistencia("pool".into()))?;
            let linhas = diesel::sql_query(
                "SELECT id, escritorio_id, nome, tipo, produto, ambiente, cron_expression, \
                         habilitado, proxima_execucao, ultima_execucao, criado_em, atualizado_em \
                  FROM trabalhos_agendados \
                  WHERE habilitado = TRUE \
                    AND proxima_execucao IS NOT NULL \
                    AND proxima_execucao <= $1 \
                  ORDER BY proxima_execucao ASC",
            )
            .bind::<Timestamptz, _>(agora)
            .load::<TrabalhoRow>(&mut conexao)
            .map_err(|_| ErroTrabalho::Persistencia("listar devidos".into()))?
            .into_iter()
            .map(|row| row.para_entidade())
            .collect::<Result<Vec<_>, _>>()?;
            Ok(linhas)
        })
        .await
        .map_err(|_| ErroTrabalho::Persistencia("spawn_blocking".into()))?
    }

    async fn atualizar_proxima_execucao(
        &self,
        id: Uuid,
        proxima: Option<DateTime<Utc>>,
        ultima: Option<DateTime<Utc>>,
    ) -> Result<(), ErroTrabalho> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroTrabalho::Persistencia("pool".into()))?;
            let atualizados = diesel::sql_query(
                "UPDATE trabalhos_agendados \
                 SET proxima_execucao = $2, ultima_execucao = $3, atualizado_em = NOW() \
                 WHERE id = $1",
            )
            .bind::<SqlUuid, _>(id)
            .bind::<Nullable<Timestamptz>, _>(proxima)
            .bind::<Nullable<Timestamptz>, _>(ultima)
            .execute(&mut conexao)
            .map_err(|_| ErroTrabalho::Persistencia("atualizar próxima execução".into()))?;
            if atualizados != 1 {
                return Err(ErroTrabalho::TrabalhoNaoEncontrado);
            }
            Ok(())
        })
        .await
        .map_err(|_| ErroTrabalho::Persistencia("spawn_blocking".into()))?
    }

    async fn habilitar(&self, id: Uuid) -> Result<(), ErroTrabalho> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroTrabalho::Persistencia("pool".into()))?;
            let atualizados = diesel::sql_query(
                "UPDATE trabalhos_agendados \
                 SET habilitado = TRUE, atualizado_em = NOW() \
                 WHERE id = $1 AND habilitado = FALSE",
            )
            .bind::<SqlUuid, _>(id)
            .execute(&mut conexao)
            .map_err(|_| ErroTrabalho::Persistencia("habilitar trabalho".into()))?;
            if atualizados == 0 {
                let existe = diesel::sql_query(
                    "SELECT COUNT(*) AS total FROM trabalhos_agendados WHERE id = $1",
                )
                .bind::<SqlUuid, _>(id)
                .get_result::<Contagem>(&mut conexao)
                .map_err(|_| ErroTrabalho::Persistencia("verificar existência".into()))?
                .total;
                if existe == 0 {
                    return Err(ErroTrabalho::TrabalhoNaoEncontrado);
                }
                return Err(ErroTrabalho::JaHabilitado);
            }
            Ok(())
        })
        .await
        .map_err(|_| ErroTrabalho::Persistencia("spawn_blocking".into()))?
    }

    async fn desabilitar(&self, id: Uuid) -> Result<(), ErroTrabalho> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroTrabalho::Persistencia("pool".into()))?;
            let atualizados = diesel::sql_query(
                "UPDATE trabalhos_agendados \
                 SET habilitado = FALSE, atualizado_em = NOW() \
                 WHERE id = $1 AND habilitado = TRUE",
            )
            .bind::<SqlUuid, _>(id)
            .execute(&mut conexao)
            .map_err(|_| ErroTrabalho::Persistencia("desabilitar trabalho".into()))?;
            if atualizados == 0 {
                let existe = diesel::sql_query(
                    "SELECT COUNT(*) AS total FROM trabalhos_agendados WHERE id = $1",
                )
                .bind::<SqlUuid, _>(id)
                .get_result::<Contagem>(&mut conexao)
                .map_err(|_| ErroTrabalho::Persistencia("verificar existência".into()))?
                .total;
                if existe == 0 {
                    return Err(ErroTrabalho::TrabalhoNaoEncontrado);
                }
                return Err(ErroTrabalho::JaDesabilitado);
            }
            Ok(())
        })
        .await
        .map_err(|_| ErroTrabalho::Persistencia("spawn_blocking".into()))?
    }
}

#[derive(QueryableByName)]
struct Contagem {
    #[diesel(sql_type = BigInt)]
    total: i64,
}
