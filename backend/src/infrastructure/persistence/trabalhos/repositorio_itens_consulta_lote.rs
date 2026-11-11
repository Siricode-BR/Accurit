//! Repositório Diesel de itens de consulta em lote.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Integer, Nullable, Text, Timestamptz, Uuid as SqlUuid};
use serde_json::Value;
use uuid::Uuid;

use crate::domain::trabalhos::entidades::{ItemConsultaLote, NovoItemConsultaLote};
use crate::domain::trabalhos::erros::ErroTrabalho;
use crate::domain::trabalhos::ports::{ContagemItensLote, RepositorioItensConsultaLote};
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

const COLUNAS: &str = "id, consulta_lote_id, documento, tipo, status, \
                       resultado_redigido, erro, criado_em, processado_em, tentativas";

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
    #[diesel(sql_type = Integer)]
    tentativas: i32,
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
            tentativas: self.tentativas,
        }
    }
}

#[derive(QueryableByName)]
struct ContagemStatusRow {
    #[diesel(sql_type = Text)]
    status: String,
    #[diesel(sql_type = BigInt)]
    total: i64,
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
                               resultado_redigido, erro, criado_em, processado_em, tentativas",
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
            let sql = format!(
                "SELECT {COLUNAS} \
                 FROM itens_consulta_lote \
                 WHERE consulta_lote_id = $1 \
                 ORDER BY criado_em ASC"
            );
            let linhas = diesel::sql_query(sql)
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
                 SET status = $2, resultado_redigido = $3, erro = $4, processado_em = $5, \
                     lease_expira_em = NULL, proxima_tentativa_em = NULL \
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

    async fn reivindicar_pendentes(
        &self,
        consulta_lote_id: Uuid,
        agora: DateTime<Utc>,
        lease_expira_em: DateTime<Utc>,
        max: usize,
    ) -> Result<Vec<ItemConsultaLote>, ErroTrabalho> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroTrabalho::Persistencia("pool".into()))?;
            let sql = format!(
                "UPDATE itens_consulta_lote \
                 SET status = 'executando', lease_expira_em = $3, tentativas = tentativas + 1 \
                 WHERE id IN (\
                   SELECT i.id FROM itens_consulta_lote i \
                   WHERE i.consulta_lote_id = $1 \
                     AND i.status = 'pendente' \
                     AND (i.proxima_tentativa_em IS NULL OR i.proxima_tentativa_em <= $2) \
                   ORDER BY i.criado_em ASC \
                   LIMIT $4 \
                   FOR UPDATE SKIP LOCKED\
                 ) \
                 RETURNING {COLUNAS}"
            );
            let linhas = diesel::sql_query(sql)
                .bind::<SqlUuid, _>(consulta_lote_id)
                .bind::<Timestamptz, _>(agora)
                .bind::<Timestamptz, _>(lease_expira_em)
                .bind::<diesel::sql_types::BigInt, _>(i64::try_from(max).unwrap_or(i64::MAX))
                .load::<ItemRow>(&mut conexao)
                .map_err(|_| ErroTrabalho::Persistencia("reivindicar itens lote".into()))?
                .into_iter()
                .map(|row| row.para_entidade())
                .collect();
            Ok(linhas)
        })
        .await
        .map_err(|_| ErroTrabalho::Persistencia("spawn_blocking".into()))?
    }

    async fn agendar_reexecucao(
        &self,
        id: Uuid,
        erro: String,
        proxima_tentativa_em: Option<DateTime<Utc>>,
    ) -> Result<(), ErroTrabalho> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroTrabalho::Persistencia("pool".into()))?;
            let atualizados = diesel::sql_query(
                "UPDATE itens_consulta_lote \
                 SET status = 'pendente', erro = $2, proxima_tentativa_em = $3, \
                     lease_expira_em = NULL \
                 WHERE id = $1",
            )
            .bind::<SqlUuid, _>(id)
            .bind::<Text, _>(erro)
            .bind::<Nullable<Timestamptz>, _>(proxima_tentativa_em)
            .execute(&mut conexao)
            .map_err(|_| ErroTrabalho::Persistencia("agendar reexecução item".into()))?;
            if atualizados != 1 {
                return Err(ErroTrabalho::ItemLoteNaoEncontrado);
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
                "UPDATE itens_consulta_lote \
                 SET status = 'pendente', lease_expira_em = NULL \
                 WHERE status = 'executando' AND lease_expira_em <= $1",
            )
            .bind::<Timestamptz, _>(agora)
            .execute(&mut conexao)
            .map_err(|_| ErroTrabalho::Persistencia("recuperar itens presos".into()))?;
            Ok(i64::try_from(recuperados).unwrap_or(i64::MAX))
        })
        .await
        .map_err(|_| ErroTrabalho::Persistencia("spawn_blocking".into()))?
    }

    async fn contar_por_status(
        &self,
        consulta_lote_id: Uuid,
        agora: DateTime<Utc>,
    ) -> Result<ContagemItensLote, ErroTrabalho> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroTrabalho::Persistencia("pool".into()))?;
            let linhas = diesel::sql_query(
                "SELECT status, COUNT(*) AS total \
                 FROM itens_consulta_lote \
                 WHERE consulta_lote_id = $1 \
                 GROUP BY status",
            )
            .bind::<SqlUuid, _>(consulta_lote_id)
            .load::<ContagemStatusRow>(&mut conexao)
            .map_err(|_| ErroTrabalho::Persistencia("contar itens por status".into()))?;
            let pendente_futuras = diesel::sql_query(
                "SELECT COUNT(*) AS total \
                 FROM itens_consulta_lote \
                 WHERE consulta_lote_id = $1 AND status = 'pendente' \
                   AND proxima_tentativa_em IS NOT NULL AND proxima_tentativa_em > $2",
            )
            .bind::<SqlUuid, _>(consulta_lote_id)
            .bind::<Timestamptz, _>(agora)
            .get_result::<ContagemRow>(&mut conexao)
            .map_err(|_| ErroTrabalho::Persistencia("contar pendentes com backoff".into()))?
            .total;
            let mut contagem = ContagemItensLote::default();
            for linha in linhas {
                match linha.status.as_str() {
                    "pendente" => contagem.pendente = linha.total,
                    "executando" => contagem.executando = linha.total,
                    "concluido" => contagem.concluido = linha.total,
                    "falhou" => contagem.falhou = linha.total,
                    "bloqueado" => contagem.bloqueado = linha.total,
                    _ => {}
                }
            }
            contagem.pendente_backoff = pendente_futuras;
            Ok(contagem)
        })
        .await
        .map_err(|_| ErroTrabalho::Persistencia("spawn_blocking".into()))?
    }

    async fn resetar_para_reexecucao(&self, consulta_lote_id: Uuid) -> Result<i64, ErroTrabalho> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroTrabalho::Persistencia("pool".into()))?;
            let resetados = diesel::sql_query(
                "UPDATE itens_consulta_lote \
                 SET status = 'pendente', erro = NULL, resultado_redigido = NULL, \
                     processado_em = NULL, proxima_tentativa_em = NULL, lease_expira_em = NULL \
                 WHERE consulta_lote_id = $1 AND status IN ('falhou', 'bloqueado')",
            )
            .bind::<SqlUuid, _>(consulta_lote_id)
            .execute(&mut conexao)
            .map_err(|_| ErroTrabalho::Persistencia("resetar itens para reexecução".into()))?;
            Ok(i64::try_from(resetados).unwrap_or(i64::MAX))
        })
        .await
        .map_err(|_| ErroTrabalho::Persistencia("spawn_blocking".into()))?
    }
}

#[derive(QueryableByName)]
struct ContagemRow {
    #[diesel(sql_type = BigInt)]
    total: i64,
}
