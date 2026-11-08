//! Repositório Diesel de emissões de guia.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_types::{Text, Timestamptz, Uuid as SqlUuid};
use diesel::sql_query;
use uuid::Uuid;

use crate::domain::guias::entidades::{GuiaEmissao, NovoGuiaEmissao};
use crate::domain::guias::enums::{StatusGuia, TipoGuia};
use crate::domain::guias::erros::ErroGuia;
use crate::domain::guias::ports::RepositorioGuiasEmissao;
use crate::infrastructure::persistence::pool::DbPool;

#[derive(Clone)]
pub struct GuiasEmissaoDiesel {
    pool: DbPool,
}

impl GuiasEmissaoDiesel {
    pub const fn novo(pool: DbPool) -> Self {
        Self { pool }
    }

    fn conexao(
        &self,
    ) -> Result<
        diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>,
        ErroGuia,
    > {
        self.pool
            .get()
            .map_err(|_| ErroGuia::Persistencia("pool indisponível".into()))
    }
}

#[derive(QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct GuiaRow {
    #[diesel(sql_type = SqlUuid)]
    id: Uuid,
    #[diesel(sql_type = SqlUuid)]
    usuario_id: Uuid,
    #[diesel(sql_type = SqlUuid)]
    escritorio_id: Uuid,
    #[diesel(sql_type = SqlUuid)]
    cliente_id: Uuid,
    #[diesel(sql_type = Text)]
    tipo_guia: String,
    #[diesel(sql_type = Text)]
    competencia: String,
    #[diesel(sql_type = Text)]
    valor_total: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<Text>)]
    pdf_url: Option<String>,
    #[diesel(sql_type = Text)]
    status: String,
    #[diesel(sql_type = Text)]
    idempotencia_chave: String,
    #[diesel(sql_type = Timestamptz)]
    criado_em: DateTime<Utc>,
    #[diesel(sql_type = Timestamptz)]
    atualizado_em: DateTime<Utc>,
}

impl GuiaRow {
    fn para_entidade(self) -> Result<GuiaEmissao, ErroGuia> {
        let tipo_guia = match self.tipo_guia.as_str() {
            "DAS" => TipoGuia::Das,
            "DARF" => TipoGuia::Darf,
            _ => return Err(ErroGuia::Persistencia("tipo de guia inválido".into())),
        };
        let status = match self.status.as_str() {
            "solicitada" => StatusGuia::Solicitada,
            "emitida" => StatusGuia::Emitida,
            "baixada" => StatusGuia::Baixada,
            "falhou" => StatusGuia::Falhou,
            _ => return Err(ErroGuia::Persistencia("status inválido".into())),
        };
        let valor_total: f64 = self.valor_total.parse().unwrap_or(0.0);
        Ok(GuiaEmissao {
            id: self.id,
            usuario_id: self.usuario_id,
            escritorio_id: self.escritorio_id,
            cliente_id: self.cliente_id,
            tipo_guia,
            competencia: self.competencia,
            valor_total,
            pdf_url: self.pdf_url,
            status,
            idempotencia_chave: self.idempotencia_chave,
            criado_em: self.criado_em,
            atualizado_em: self.atualizado_em,
        })
    }
}

#[async_trait]
impl RepositorioGuiasEmissao for GuiasEmissaoDiesel {
    async fn criar(
        &self,
        guia: &NovoGuiaEmissao,
    ) -> Result<GuiaEmissao, ErroGuia> {
        let pool = self.pool.clone();
        let novo = guia.clone();
        tokio::task::spawn_blocking(move || -> Result<GuiaEmissao, ErroGuia> {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroGuia::Persistencia("pool".into()))?;
            let linha = diesel::sql_query(
                "INSERT INTO guias_emissao \
                 (id, usuario_id, escritorio_id, cliente_id, tipo_guia, competencia, \
                  valor_total, status, idempotencia_chave) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, 'solicitada', $8) \
                 RETURNING id, usuario_id, escritorio_id, cliente_id, tipo_guia, \
                          competencia, valor_total, pdf_url, status, \
                          idempotencia_chave, criado_em, atualizado_em",
            )
            .bind::<SqlUuid, _>(Uuid::now_v7())
            .bind::<SqlUuid, _>(novo.usuario_id)
            .bind::<SqlUuid, _>(novo.escritorio_id)
            .bind::<SqlUuid, _>(novo.cliente_id)
            .bind::<Text, _>(novo.tipo_guia.as_str())
            .bind::<Text, _>(novo.competencia)
            .bind::<Text, _>(novo.valor_total.to_string())
            .bind::<Text, _>(novo.idempotencia_chave)
            .get_result::<GuiaRow>(&mut conexao)
            .map_err(|_| ErroGuia::Persistencia("inserir guia".into()))?
            .para_entidade();
            linha
        })
        .await
        .map_err(|_| ErroGuia::Persistencia("spawn_blocking".into()))?
    }

    async fn buscar_por_id(
        &self,
        id: Uuid,
    ) -> Result<Option<GuiaEmissao>, ErroGuia> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || -> Result<Option<GuiaEmissao>, ErroGuia> {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroGuia::Persistencia("pool".into()))?;
            let linha = diesel::sql_query(
                "SELECT id, usuario_id, escritorio_id, cliente_id, tipo_guia, \
                         competencia, valor_total, pdf_url, status, \
                         idempotencia_chave, criado_em, atualizado_em \
                  FROM guias_emissao \
                  WHERE id = $1",
            )
            .bind::<SqlUuid, _>(id)
            .get_result::<GuiaRow>(&mut conexao)
            .optional()
            .map_err(|_| ErroGuia::Persistencia("buscar guia".into()))?
            .map(|row| row.para_entidade())
            .transpose();
            linha
        })
        .await
        .map_err(|_| ErroGuia::Persistencia("spawn_blocking".into()))?
    }

    async fn buscar_por_idempotencia(
        &self,
        chave: &str,
    ) -> Result<Option<GuiaEmissao>, ErroGuia> {
        let pool = self.pool.clone();
        let chave_owned = chave.to_owned();
        tokio::task::spawn_blocking(move || -> Result<Option<GuiaEmissao>, ErroGuia> {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroGuia::Persistencia("pool".into()))?;
            let linha = diesel::sql_query(
                "SELECT id, usuario_id, escritorio_id, cliente_id, tipo_guia, \
                         competencia, valor_total, pdf_url, status, \
                         idempotencia_chave, criado_em, atualizado_em \
                  FROM guias_emissao \
                  WHERE idempotencia_chave = $1",
            )
            .bind::<Text, _>(chave_owned)
            .get_result::<GuiaRow>(&mut conexao)
            .optional()
            .map_err(|_| ErroGuia::Persistencia("buscar guia por idempotência".into()))?
            .map(|row| row.para_entidade())
            .transpose();
            linha
        })
        .await
        .map_err(|_| ErroGuia::Persistencia("spawn_blocking".into()))?
    }

    async fn listar_por_usuario(
        &self,
        usuario_id: Uuid,
        limite: i64,
    ) -> Result<Vec<GuiaEmissao>, ErroGuia> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || -> Result<Vec<GuiaEmissao>, ErroGuia> {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroGuia::Persistencia("pool".into()))?;
            let linhas = diesel::sql_query(
                "SELECT id, usuario_id, escritorio_id, cliente_id, tipo_guia, \
                         competencia, valor_total, pdf_url, status, \
                         idempotencia_chave, criado_em, atualizado_em \
                  FROM guias_emissao \
                  WHERE usuario_id = $1 \
                  ORDER BY criado_em DESC \
                  LIMIT $2",
            )
            .bind::<SqlUuid, _>(usuario_id)
            .bind::<diesel::sql_types::BigInt, _>(limite)
            .load::<GuiaRow>(&mut conexao)
            .map_err(|_| ErroGuia::Persistencia("listar guias por usuário".into()))?
            .into_iter()
            .map(|row| row.para_entidade())
            .collect::<Result<Vec<_>, _>>()?;
            Ok(linhas)
        })
        .await
        .map_err(|_| ErroGuia::Persistencia("spawn_blocking".into()))?
    }

    async fn listar_por_escritorio(
        &self,
        escritorio_id: Uuid,
        limite: i64,
    ) -> Result<Vec<GuiaEmissao>, ErroGuia> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || -> Result<Vec<GuiaEmissao>, ErroGuia> {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroGuia::Persistencia("pool".into()))?;
            let linhas = diesel::sql_query(
                "SELECT id, usuario_id, escritorio_id, cliente_id, tipo_guia, \
                         competencia, valor_total, pdf_url, status, \
                         idempotencia_chave, criado_em, atualizado_em \
                  FROM guias_emissao \
                  WHERE escritorio_id = $1 \
                  ORDER BY criado_em DESC \
                  LIMIT $2",
            )
            .bind::<SqlUuid, _>(escritorio_id)
            .bind::<diesel::sql_types::BigInt, _>(limite)
            .load::<GuiaRow>(&mut conexao)
            .map_err(|_| ErroGuia::Persistencia("listar guias por escritório".into()))?
            .into_iter()
            .map(|row| row.para_entidade())
            .collect::<Result<Vec<_>, _>>()?;
            Ok(linhas)
        })
        .await
        .map_err(|_| ErroGuia::Persistencia("spawn_blocking".into()))?
    }

    async fn listar_por_cliente(
        &self,
        cliente_id: Uuid,
        limite: i64,
    ) -> Result<Vec<GuiaEmissao>, ErroGuia> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || -> Result<Vec<GuiaEmissao>, ErroGuia> {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroGuia::Persistencia("pool".into()))?;
            let linhas = diesel::sql_query(
                "SELECT id, usuario_id, escritorio_id, cliente_id, tipo_guia, \
                         competencia, valor_total, pdf_url, status, \
                         idempotencia_chave, criado_em, atualizado_em \
                  FROM guias_emissao \
                  WHERE cliente_id = $1 \
                  ORDER BY criado_em DESC \
                  LIMIT $2",
            )
            .bind::<SqlUuid, _>(cliente_id)
            .bind::<diesel::sql_types::BigInt, _>(limite)
            .load::<GuiaRow>(&mut conexao)
            .map_err(|_| ErroGuia::Persistencia("listar guias por cliente".into()))?
            .into_iter()
            .map(|row| row.para_entidade())
            .collect::<Result<Vec<_>, _>>()?;
            Ok(linhas)
        })
        .await
        .map_err(|_| ErroGuia::Persistencia("spawn_blocking".into()))?
    }

    async fn atualizar_status(
        &self,
        id: Uuid,
        status: StatusGuia,
        pdf_url: Option<String>,
    ) -> Result<(), ErroGuia> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || -> Result<(), ErroGuia> {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroGuia::Persistencia("pool".into()))?;
            let atualizados = diesel::sql_query(
                "UPDATE guias_emissao \
                 SET status = $2, pdf_url = $3, atualizado_em = NOW() \
                 WHERE id = $1",
            )
            .bind::<SqlUuid, _>(id)
            .bind::<Text, _>(status.as_str())
            .bind::<diesel::sql_types::Nullable<Text>, _>(pdf_url)
            .execute(&mut conexao)
            .map_err(|_| ErroGuia::Persistencia("atualizar status guia".into()))?;
            if atualizados != 1 {
                return Err(ErroGuia::GuiaNaoEncontrada);
            }
            Ok(())
        })
        .await
        .map_err(|_| ErroGuia::Persistencia("spawn_blocking".into()))?
    }
}
