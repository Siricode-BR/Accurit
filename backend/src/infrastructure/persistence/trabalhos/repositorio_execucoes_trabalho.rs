//! Repositório Diesel de execuções de trabalho.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_types::{Integer, Nullable, Text, Timestamptz, Uuid as SqlUuid};
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
            criado_em: self.criado_em,
        })
    }
}

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
                         registros_com_erro, criado_em \
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
                         registros_com_erro, criado_em \
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
                         registros_com_erro, criado_em \
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
}
