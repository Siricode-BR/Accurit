//! Repositório Diesel de execuções de trabalho.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Integer, Nullable, Text, Timestamptz, Uuid as SqlUuid};
use uuid::Uuid;

use crate::domain::trabalhos::entidades::ExecucaoTrabalho;
use crate::domain::trabalhos::enums::StatusExecucao;
use crate::domain::trabalhos::erros::ErroTrabalho;
use crate::domain::trabalhos::ports::RepositorioExecucoesTrabalho;
use crate::infrastructure::persistence::pool::DbPool;

#[derive(Clone)]
pub struct ExecucoesTrabalhoDiesel {
    pool: DbPool,
}

impl ExecucoesTrabalhoDiesel {
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
struct UuidRetorno {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: uuid::Uuid,
}

#[derive(QueryableByName)]
struct ExecucaoRow {
    #[diesel(sql_type = SqlUuid)]
    id: Uuid,
    #[diesel(sql_type = SqlUuid)]
    trabalho_agendado_id: Uuid,
    #[diesel(sql_type = SqlUuid)]
    escritorio_id: Uuid,
    #[diesel(sql_type = Text)]
    status: String,
    #[diesel(sql_type = Timestamptz)]
    iniciado_em: DateTime<Utc>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    finalizado_em: Option<DateTime<Utc>>,
    #[diesel(sql_type = Nullable<Text>)]
    erro_mensagem: Option<String>,
    #[diesel(sql_type = Integer)]
    registros_processados: i32,
    #[diesel(sql_type = Integer)]
    registros_com_erro: i32,
    #[diesel(sql_type = Integer)]
    tentativas: i32,
    #[diesel(sql_type = Timestamptz)]
    criado_em: DateTime<Utc>,
}

impl ExecucaoRow {
    fn para_entidade(self) -> Result<ExecucaoTrabalho, ErroTrabalho> {
        let status = match self.status.as_str() {
            "pendente" => StatusExecucao::Pendente,
            "executando" => StatusExecucao::Executando,
            "concluido" => StatusExecucao::Concluido,
            "falhou" => StatusExecucao::Falhou,
            "bloqueado" => StatusExecucao::Bloqueado,
            _ => return Err(ErroTrabalho::Persistencia("status inválido".into())),
        };
        Ok(ExecucaoTrabalho {
            id: self.id,
            trabalho_agendado_id: self.trabalho_agendado_id,
            escritorio_id: self.escritorio_id,
            status,
            iniciado_em: self.iniciado_em,
            finalizado_em: self.finalizado_em,
            erro_mensagem: self.erro_mensagem,
            registros_processados: self.registros_processados,
            registros_com_erro: self.registros_com_erro,
            tentativas: self.tentativas,
            criado_em: self.criado_em,
        })
    }
}

const COLUNAS: &str = "id, trabalho_agendado_id, escritorio_id, status, iniciado_em, \
                       finalizado_em, erro_mensagem, registros_processados, \
                       registros_com_erro, tentativas, criado_em";

#[async_trait]
impl RepositorioExecucoesTrabalho for ExecucoesTrabalhoDiesel {
    async fn criar(&self, execucao: &ExecucaoTrabalho) -> Result<Uuid, ErroTrabalho> {
        let pool = self.pool.clone();
        let novo = execucao.clone();
        tokio::task::spawn_blocking(move || {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroTrabalho::Persistencia("pool".into()))?;
            let id = diesel::sql_query(
                "INSERT INTO execucoes_trabalho \
                 (id, trabalho_agendado_id, escritorio_id, status, erro_mensagem, \
                  registros_processados, registros_com_erro) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7) \
                 RETURNING id",
            )
            .bind::<SqlUuid, _>(novo.id)
            .bind::<SqlUuid, _>(novo.trabalho_agendado_id)
            .bind::<SqlUuid, _>(novo.escritorio_id)
            .bind::<Text, _>(novo.status.as_str())
            .bind::<Nullable<Text>, _>(novo.erro_mensagem)
            .bind::<Integer, _>(novo.registros_processados)
            .bind::<Integer, _>(novo.registros_com_erro)
            .get_result::<UuidRetorno>(&mut conexao)
            .map(|row| row.id)
            .map_err(|_| ErroTrabalho::Persistencia("inserir execução".into()))?;
            Ok(id)
        })
        .await
        .map_err(|_| ErroTrabalho::Persistencia("spawn_blocking".into()))?
    }

    async fn buscar_por_id(&self, id: Uuid) -> Result<Option<ExecucaoTrabalho>, ErroTrabalho> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroTrabalho::Persistencia("pool".into()))?;
            let linha = diesel::sql_query(
                "SELECT id, trabalho_agendado_id, escritorio_id, status, iniciado_em, \
                          finalizado_em, erro_mensagem, registros_processados, \
                          registros_com_erro, tentativas, criado_em \
                  FROM execucoes_trabalho \
                  WHERE id = $1",
            )
            .bind::<SqlUuid, _>(id)
            .get_result::<ExecucaoRow>(&mut conexao)
            .optional()
            .map_err(|_| ErroTrabalho::Persistencia("buscar execução".into()))?
            .map(|row| row.para_entidade())
            .transpose()?;
            Ok(linha)
        })
        .await
        .map_err(|_| ErroTrabalho::Persistencia("spawn_blocking".into()))?
    }

    async fn listar_por_trabalho(
        &self,
        trabalho_agendado_id: Uuid,
    ) -> Result<Vec<ExecucaoTrabalho>, ErroTrabalho> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroTrabalho::Persistencia("pool".into()))?;
            let linhas = diesel::sql_query(
                "SELECT id, trabalho_agendado_id, escritorio_id, status, iniciado_em, \
                          finalizado_em, erro_mensagem, registros_processados, \
                          registros_com_erro, tentativas, criado_em \
                  FROM execucoes_trabalho \
                  WHERE trabalho_agendado_id = $1 \
                  ORDER BY criado_em DESC",
            )
            .bind::<SqlUuid, _>(trabalho_agendado_id)
            .load::<ExecucaoRow>(&mut conexao)
            .map_err(|_| ErroTrabalho::Persistencia("listar execuções".into()))?
            .into_iter()
            .map(|row| row.para_entidade())
            .collect::<Result<Vec<_>, _>>()?;
            Ok(linhas)
        })
        .await
        .map_err(|_| ErroTrabalho::Persistencia("spawn_blocking".into()))?
    }

    async fn listar_por_escritorio(
        &self,
        escritorio_id: Uuid,
    ) -> Result<Vec<ExecucaoTrabalho>, ErroTrabalho> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroTrabalho::Persistencia("pool".into()))?;
            let linhas = diesel::sql_query(
                "SELECT id, trabalho_agendado_id, escritorio_id, status, iniciado_em, \
                          finalizado_em, erro_mensagem, registros_processados, \
                          registros_com_erro, tentativas, criado_em \
                  FROM execucoes_trabalho \
                  WHERE escritorio_id = $1 \
                  ORDER BY criado_em DESC",
            )
            .bind::<SqlUuid, _>(escritorio_id)
            .load::<ExecucaoRow>(&mut conexao)
            .map_err(|_| ErroTrabalho::Persistencia("listar execuções por escritório".into()))?
            .into_iter()
            .map(|row| row.para_entidade())
            .collect::<Result<Vec<_>, _>>()?;
            Ok(linhas)
        })
        .await
        .map_err(|_| ErroTrabalho::Persistencia("spawn_blocking".into()))?
    }

    async fn atualizar_status(
        &self,
        id: Uuid,
        status: StatusExecucao,
        finalizado_em: Option<DateTime<Utc>>,
        erro_mensagem: Option<String>,
        registros_processados: i32,
        registros_com_erro: i32,
    ) -> Result<(), ErroTrabalho> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroTrabalho::Persistencia("pool".into()))?;
            let atualizados = diesel::sql_query(
                "UPDATE execucoes_trabalho \
                 SET status = $2, finalizado_em = $3, erro_mensagem = $4, \
                     registros_processados = $5, registros_com_erro = $6 \
                 WHERE id = $1",
            )
            .bind::<SqlUuid, _>(id)
            .bind::<Text, _>(status.as_str())
            .bind::<Nullable<Timestamptz>, _>(finalizado_em)
            .bind::<Nullable<Text>, _>(erro_mensagem)
            .bind::<Integer, _>(registros_processados)
            .bind::<Integer, _>(registros_com_erro)
            .execute(&mut conexao)
            .map_err(|_| ErroTrabalho::Persistencia("atualizar status execução".into()))?;
            if atualizados != 1 {
                return Err(ErroTrabalho::ExecucaoNaoEncontrada);
            }
            Ok(())
        })
        .await
        .map_err(|_| ErroTrabalho::Persistencia("spawn_blocking".into()))?
    }

    async fn reivindicar_pendentes(
        &self,
        agora: DateTime<Utc>,
        lease_expira_em: DateTime<Utc>,
        max: usize,
    ) -> Result<Vec<ExecucaoTrabalho>, ErroTrabalho> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroTrabalho::Persistencia("pool".into()))?;
            // Reivindicação atômica: pendentes com backoff vencido (retry) —
            // `FOR UPDATE SKIP LOCKED` evita disputa entre workers.
            let sql = format!(
                "UPDATE execucoes_trabalho \
                 SET status = 'executando', lease_expira_em = $1, \
                     tentativas = tentativas + 1, \
                     iniciado_em = CASE WHEN tentativas = 0 THEN $2 ELSE iniciado_em END \
                 WHERE id IN (\
                   SELECT e.id FROM execucoes_trabalho e \
                   WHERE e.status = 'pendente' \
                     AND (e.proxima_tentativa_em IS NULL OR e.proxima_tentativa_em <= $2) \
                   ORDER BY e.criado_em ASC \
                   LIMIT $3 \
                   FOR UPDATE SKIP LOCKED\
                 ) \
                 RETURNING {COLUNAS}"
            );
            let linhas = diesel::sql_query(sql)
                .bind::<Timestamptz, _>(lease_expira_em)
                .bind::<Timestamptz, _>(agora)
                .bind::<BigInt, _>(i64::try_from(max).unwrap_or(i64::MAX))
                .load::<ExecucaoRow>(&mut conexao)
                .map_err(|_| ErroTrabalho::Persistencia("reivindicar execuções".into()))?
                .into_iter()
                .map(|row| row.para_entidade())
                .collect::<Result<Vec<_>, _>>()?;
            Ok(linhas)
        })
        .await
        .map_err(|_| ErroTrabalho::Persistencia("spawn_blocking".into()))?
    }

    async fn renovar_lease(
        &self,
        id: Uuid,
        lease_expira_em: DateTime<Utc>,
    ) -> Result<(), ErroTrabalho> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroTrabalho::Persistencia("pool".into()))?;
            let atualizados = diesel::sql_query(
                "UPDATE execucoes_trabalho \
                 SET lease_expira_em = $2 \
                 WHERE id = $1 AND status = 'executando'",
            )
            .bind::<SqlUuid, _>(id)
            .bind::<Timestamptz, _>(lease_expira_em)
            .execute(&mut conexao)
            .map_err(|_| ErroTrabalho::Persistencia("renovar lease execução".into()))?;
            if atualizados != 1 {
                // Perdeu o lease (recuperado por outro worker): falha,
                // sinalizando ao runner para abandonar a execução local.
                return Err(ErroTrabalho::ExecucaoNaoEncontrada);
            }
            Ok(())
        })
        .await
        .map_err(|_| ErroTrabalho::Persistencia("spawn_blocking".into()))?
    }

    async fn agendar_reexecucao(
        &self,
        id: Uuid,
        erro_mensagem: String,
        proxima_tentativa_em: Option<DateTime<Utc>>,
        registros_processados: i32,
        registros_com_erro: i32,
    ) -> Result<(), ErroTrabalho> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroTrabalho::Persistencia("pool".into()))?;
            let atualizados = diesel::sql_query(
                "UPDATE execucoes_trabalho \
                 SET status = 'pendente', erro_mensagem = $2, proxima_tentativa_em = $3, \
                     lease_expira_em = NULL, finalizado_em = NULL, \
                     registros_processados = $4, registros_com_erro = $5 \
                 WHERE id = $1",
            )
            .bind::<SqlUuid, _>(id)
            .bind::<Nullable<Text>, _>(Some(erro_mensagem))
            .bind::<Nullable<Timestamptz>, _>(proxima_tentativa_em)
            .bind::<Integer, _>(registros_processados)
            .bind::<Integer, _>(registros_com_erro)
            .execute(&mut conexao)
            .map_err(|_| ErroTrabalho::Persistencia("agendar reexecução".into()))?;
            if atualizados != 1 {
                return Err(ErroTrabalho::ExecucaoNaoEncontrada);
            }
            Ok(())
        })
        .await
        .map_err(|_| ErroTrabalho::Persistencia("spawn_blocking".into()))?
    }

    async fn recuperar_presos(&self, agora: DateTime<Utc>) -> Result<i64, ErroTrabalho> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroTrabalho::Persistencia("pool".into()))?;
            let recuperados = diesel::sql_query(
                "UPDATE execucoes_trabalho \
                 SET status = 'pendente', lease_expira_em = NULL, proxima_tentativa_em = NULL \
                 WHERE status = 'executando' AND lease_expira_em <= $1",
            )
            .bind::<Timestamptz, _>(agora)
            .execute(&mut conexao)
            .map_err(|_| ErroTrabalho::Persistencia("recuperar execuções presas".into()))?;
            Ok(i64::try_from(recuperados).unwrap_or(i64::MAX))
        })
        .await
        .map_err(|_| ErroTrabalho::Persistencia("spawn_blocking".into()))?
    }
}
