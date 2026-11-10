//! Repositório Diesel de itens de consulta em lote.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_types::{Nullable, Text, Timestamptz, Uuid as SqlUuid};
use serde_json::Value;
use uuid::Uuid;

use crate::domain::trabalhos::entidades::{ItemConsultaLote, NovoItemConsultaLote};
use crate::domain::trabalhos::erros::ErroTrabalho;
use crate::domain::trabalhos::ports::RepositorioItensConsultaLote;
use crate::infrastructure::persistence::pool::DbPool;

#[derive(Clone)]
pub struct ItensConsultaLoteDiesel {
    pool: DbPool,
}

impl ItensConsultaLoteDiesel {
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
struct ItemRow {
    #[diesel(sql_type = SqlUuid)]
    id: Uuid,
    #[diesel(sql_type = SqlUuid)]
    consulta_lote_id: Uuid,
    #[diesel(sql_type = Text)]
    documento: String,
    #[diesel(sql_type = Text)]
    tipo: String,
    #[diesel(sql_type = Text)]
    status: String,
    #[diesel(sql_type = Nullable<diesel::sql_types::Jsonb>)]
    resultado_redigido: Option<Value>,
    #[diesel(sql_type = Nullable<Text>)]
    erro: Option<String>,
    #[diesel(sql_type = Timestamptz)]
    criado_em: DateTime<Utc>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    processado_em: Option<DateTime<Utc>>,
}

impl ItemRow {
    fn para_entidade(self) -> ItemConsultaLote {
        ItemConsultaLote {
            id: self.id,
            consulta_lote_id: self.consulta_lote_id,
            documento: self.documento,
            tipo: self.tipo,
            status: self.status,
            resultado_redigido: self.resultado_redigido,
            erro: self.erro,
            criado_em: self.criado_em,
            processado_em: self.processado_em,
        }
    }
}

#[async_trait]
impl RepositorioItensConsultaLote for ItensConsultaLoteDiesel {
    async fn criar(
        &self,
        consulta_lote_id: Uuid,
        itens: &[NovoItemConsultaLote],
    ) -> Result<Vec<ItemConsultaLote>, ErroTrabalho> {
        if itens.is_empty() {
            return Ok(Vec::new());
        }
        let pool = self.pool.clone();
        let lote_id = consulta_lote_id;
        let itens_novos = itens.to_vec();
        tokio::task::spawn_blocking(move || {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroTrabalho::Persistencia("pool".into()))?;
            let mut criados = Vec::with_capacity(itens_novos.len());
            for item in itens_novos {
                let linha = diesel::sql_query(
                    "INSERT INTO itens_consulta_lote \
                     (id, consulta_lote_id, documento, tipo, status) \
                     VALUES ($1, $2, $3, $4, 'pendente') \
                     RETURNING id, consulta_lote_id, documento, tipo, status, \
                              resultado_redigido, erro, criado_em, processado_em",
                )
                .bind::<SqlUuid, _>(Uuid::now_v7())
                .bind::<SqlUuid, _>(lote_id)
                .bind::<Text, _>(item.documento)
                .bind::<Text, _>(item.tipo)
                .get_result::<ItemRow>(&mut conexao)
                .map_err(|_| ErroTrabalho::Persistencia("inserir item lote".into()))?;
                criados.push(linha.para_entidade());
            }
            Ok(criados)
        })
        .await
        .map_err(|_| ErroTrabalho::Persistencia("spawn_blocking".into()))?
    }

    async fn listar_por_lote(
        &self,
        consulta_lote_id: Uuid,
    ) -> Result<Vec<ItemConsultaLote>, ErroTrabalho> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroTrabalho::Persistencia("pool".into()))?;
            let linhas = diesel::sql_query(
                "SELECT id, consulta_lote_id, documento, tipo, status, \
                         resultado_redigido, erro, criado_em, processado_em \
                  FROM itens_consulta_lote \
                  WHERE consulta_lote_id = $1 \
                  ORDER BY criado_em ASC",
            )
            .bind::<SqlUuid, _>(consulta_lote_id)
            .load::<ItemRow>(&mut conexao)
            .map_err(|_| ErroTrabalho::Persistencia("listar itens lote".into()))?
            .into_iter()
            .map(|row| row.para_entidade())
            .collect();
            Ok(linhas)
        })
        .await
        .map_err(|_| ErroTrabalho::Persistencia("spawn_blocking".into()))?
    }

    async fn atualizar_item(
        &self,
        id: Uuid,
        status: String,
        resultado_redigido: Option<Value>,
        erro: Option<String>,
        processado_em: Option<DateTime<Utc>>,
    ) -> Result<(), ErroTrabalho> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroTrabalho::Persistencia("pool".into()))?;
            let atualizados = diesel::sql_query(
                "UPDATE itens_consulta_lote \
                 SET status = $2, resultado_redigido = $3, erro = $4, processado_em = $5 \
                 WHERE id = $1",
            )
            .bind::<SqlUuid, _>(id)
            .bind::<Text, _>(status)
            .bind::<Nullable<diesel::sql_types::Jsonb>, _>(resultado_redigido)
            .bind::<Nullable<Text>, _>(erro)
            .bind::<Nullable<Timestamptz>, _>(processado_em)
            .execute(&mut conexao)
            .map_err(|_| ErroTrabalho::Persistencia("atualizar item lote".into()))?;
            if atualizados != 1 {
                return Err(ErroTrabalho::ItemLoteNaoEncontrado);
            }
            Ok(())
        })
        .await
        .map_err(|_| ErroTrabalho::Persistencia("spawn_blocking".into()))?
    }
}
