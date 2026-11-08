//! Repositório Diesel de consultas em lote.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_types::{Integer, Nullable, Text, Timestamptz, Uuid as SqlUuid};
use uuid::Uuid;

use crate::domain::trabalhos::entidades::{ConsultaLote, NovoConsultaLote};
use crate::domain::trabalhos::enums::StatusLote;
use crate::domain::trabalhos::erros::ErroTrabalho;
use crate::domain::trabalhos::ports::RepositorioConsultasLote;
use crate::infrastructure::persistence::pool::DbPool;

#[derive(Clone)]
pub struct ConsultasLoteDiesel {
    pool: DbPool,
}

impl ConsultasLoteDiesel {
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
struct LoteRow {
    #[diesel(sql_type = SqlUuid)]
    id: Uuid,
    #[diesel(sql_type = SqlUuid)]
    escritorio_id: Uuid,
    #[diesel(sql_type = Text)]
    nome: String,
    #[diesel(sql_type = Text)]
    produto: String,
    #[diesel(sql_type = Text)]
    servico: String,
    #[diesel(sql_type = Nullable<Text>)]
    verbo: Option<String>,
    #[diesel(sql_type = Text)]
    status: String,
    #[diesel(sql_type = Integer)]
    total_itens: i32,
    #[diesel(sql_type = Integer)]
    itens_concluidos: i32,
    #[diesel(sql_type = Integer)]
    itens_com_erro: i32,
    #[diesel(sql_type = Timestamptz)]
    criado_em: DateTime<Utc>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    iniciado_em: Option<DateTime<Utc>>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    finalizado_em: Option<DateTime<Utc>>,
}

impl LoteRow {
    fn para_entidade(self) -> Result<ConsultaLote, ErroTrabalho> {
        let status = match self.status.as_str() {
            "pendente" => StatusLote::Pendente,
            "executando" => StatusLote::Executando,
            "concluido" => StatusLote::Concluido,
            "falhou" => StatusLote::Falhou,
            "parcial" => StatusLote::Parcial,
            _ => return Err(ErroTrabalho::Persistencia("status inválido".into())),
        };
        Ok(ConsultaLote {
            id: self.id,
            escritorio_id: self.escritorio_id,
            nome: self.nome,
            produto: self.produto,
            servico: self.servico,
            verbo: self.verbo,
            status,
            total_itens: self.total_itens,
            itens_concluidos: self.itens_concluidos,
            itens_com_erro: self.itens_com_erro,
            criado_em: self.criado_em,
            iniciado_em: self.iniciado_em,
            finalizado_em: self.finalizado_em,
        })
    }
}

#[async_trait]
impl RepositorioConsultasLote for ConsultasLoteDiesel {
    async fn criar(
        &self,
        lote: &NovoConsultaLote,
    ) -> Result<ConsultaLote, ErroTrabalho> {
        let pool = self.pool.clone();
        let novo = lote.clone();
        tokio::task::spawn_blocking(move || {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroTrabalho::Persistencia("pool".into()))?;
            let linha = diesel::sql_query(
                "INSERT INTO consultas_em_lote \
                 (id, escritorio_id, nome, produto, servico, verbo, status) \
                 VALUES ($1, $2, $3, $4, $5, $6, 'pendente') \
                 RETURNING id, escritorio_id, nome, produto, servico, verbo, status, \
                          total_itens, itens_concluidos, itens_com_erro, \
                          criado_em, iniciado_em, finalizado_em",
            )
            .bind::<SqlUuid, _>(Uuid::now_v7())
            .bind::<SqlUuid, _>(novo.escritorio_id)
            .bind::<Text, _>(novo.nome)
            .bind::<Text, _>(novo.produto)
            .bind::<Text, _>(novo.servico)
            .bind::<Nullable<Text>, _>(novo.verbo)
            .get_result::<LoteRow>(&mut conexao)
            .map_err(|_| ErroTrabalho::Persistencia("inserir lote".into()))?
            .para_entidade()?;
            Ok(linha)
        })
        .await
        .map_err(|_| ErroTrabalho::Persistencia("spawn_blocking".into()))?
    }

    async fn buscar_por_id(&self, id: Uuid) -> Result<Option<ConsultaLote>, ErroTrabalho> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroTrabalho::Persistencia("pool".into()))?;
            let linha = diesel::sql_query(
                "SELECT id, escritorio_id, nome, produto, servico, verbo, status, \
                         total_itens, itens_concluidos, itens_com_erro, \
                         criado_em, iniciado_em, finalizado_em \
                  FROM consultas_em_lote \
                  WHERE id = $1",
            )
            .bind::<SqlUuid, _>(id)
            .get_result::<LoteRow>(&mut conexao)
            .optional()
            .map_err(|_| ErroTrabalho::Persistencia("buscar lote".into()))?
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
    ) -> Result<Vec<ConsultaLote>, ErroTrabalho> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroTrabalho::Persistencia("pool".into()))?;
            let linhas = diesel::sql_query(
                "SELECT id, escritorio_id, nome, produto, servico, verbo, status, \
                         total_itens, itens_concluidos, itens_com_erro, \
                         criado_em, iniciado_em, finalizado_em \
                  FROM consultas_em_lote \
                  WHERE escritorio_id = $1 \
                  ORDER BY criado_em DESC",
            )
            .bind::<SqlUuid, _>(escritorio_id)
            .load::<LoteRow>(&mut conexao)
            .map_err(|_| ErroTrabalho::Persistencia("listar lotes".into()))?
            .into_iter()
            .map(|row| row.para_entidade())
            .collect::<Result<Vec<_>, _>>()?;
            Ok(linhas)
        })
        .await
        .map_err(|_| ErroTrabalho::Persistencia("spawn_blocking".into()))?
    }

    #[allow(clippy::too_many_arguments)]
    async fn atualizar_status(
        &self,
        id: Uuid,
        status: StatusLote,
        iniciado_em: Option<DateTime<Utc>>,
        finalizado_em: Option<DateTime<Utc>>,
        total_itens: i32,
        itens_concluidos: i32,
        itens_com_erro: i32,
    ) -> Result<(), ErroTrabalho> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroTrabalho::Persistencia("pool".into()))?;
            let atualizados = diesel::sql_query(
                "UPDATE consultas_em_lote \
                 SET status = $2, iniciado_em = $3, finalizado_em = $4, \
                     total_itens = $5, itens_concluidos = $6, itens_com_erro = $7 \
                 WHERE id = $1",
            )
            .bind::<SqlUuid, _>(id)
            .bind::<Text, _>(status.as_str())
            .bind::<Nullable<Timestamptz>, _>(iniciado_em)
            .bind::<Nullable<Timestamptz>, _>(finalizado_em)
            .bind::<Integer, _>(total_itens)
            .bind::<Integer, _>(itens_concluidos)
            .bind::<Integer, _>(itens_com_erro)
            .execute(&mut conexao)
            .map_err(|_| ErroTrabalho::Persistencia("atualizar status lote".into()))?;
            if atualizados != 1 {
                return Err(ErroTrabalho::ConsultaLoteNaoEncontrada);
            }
            Ok(())
        })
        .await
        .map_err(|_| ErroTrabalho::Persistencia("spawn_blocking".into()))?
    }
}
