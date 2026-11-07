//! Leitura do histórico `consultas_fiscais` por cliente (SPEC §9.3).
//!
//! Somente leitura: o registro continua na Fase 4
//! (`ConsultasFiscaisRepositorioDiesel`, via Dívida Ativa). Aqui só
//! existem as consultas que alimentam a tela de detalhes do cliente —
//! paginação estável (`criado_em DESC, id DESC`) e a última tentativa por
//! serviço (`DISTINCT ON (servico)`) para os cartões de situação. Ordem
//! determinística de empate por id: `Uuid::now_v7` é monotônico.

use diesel::{
    QueryableByName, RunQueryDsl, sql_query,
    sql_types::{BigInt, Int4 as SqlInt4, Nullable, Text, Timestamptz, Uuid as SqlUuid},
};
use uuid::Uuid;

use crate::application::clientes::{
    ConsultaFiscalDoCliente, FiltroConsultasCliente, ListagemConsultasDoCliente,
    RepositorioConsultasDoCliente, RepositorioError, SituacaoFiscalServico,
};
use crate::domain::clientes::Paginacao;
use crate::infrastructure::persistence::pool::DbPool;

const COLUNAS_CONSULTA: &str = "id, produto, servico, verbo, ambiente, estado, \
     fonte_consulta, chave_consulta_mascarada, status_code, criado_em";

#[derive(QueryableByName)]
struct LinhaConsulta {
    #[diesel(sql_type = SqlUuid)]
    id: Uuid,
    #[diesel(sql_type = Text)]
    produto: String,
    #[diesel(sql_type = Text)]
    servico: String,
    #[diesel(sql_type = Nullable<Text>)]
    verbo: Option<String>,
    #[diesel(sql_type = Text)]
    ambiente: String,
    #[diesel(sql_type = Text)]
    estado: String,
    #[diesel(sql_type = Nullable<Text>)]
    fonte_consulta: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    chave_consulta_mascarada: Option<String>,
    #[diesel(sql_type = Nullable<SqlInt4>)]
    status_code: Option<i32>,
    #[diesel(sql_type = Timestamptz)]
    criado_em: chrono::DateTime<chrono::Utc>,
}

/// Projeto enxuto do `DISTINCT ON (servico)` — sem `id`, `produto` e
/// `verbo`, que não alimentam o cartão de situação.
#[derive(QueryableByName)]
struct LinhaResumo {
    #[diesel(sql_type = Text)]
    servico: String,
    #[diesel(sql_type = Text)]
    estado: String,
    #[diesel(sql_type = Text)]
    ambiente: String,
    #[diesel(sql_type = Nullable<Text>)]
    fonte_consulta: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    chave_consulta_mascarada: Option<String>,
    #[diesel(sql_type = Nullable<SqlInt4>)]
    status_code: Option<i32>,
    #[diesel(sql_type = Timestamptz)]
    criado_em: chrono::DateTime<chrono::Utc>,
}

#[derive(QueryableByName)]
struct Contagem {
    #[diesel(sql_type = BigInt)]
    total: i64,
}

#[derive(Clone)]
pub struct ConsultasDoClienteRepositorioDiesel {
    pool: DbPool,
}

impl ConsultasDoClienteRepositorioDiesel {
    pub const fn novo(pool: DbPool) -> Self {
        Self { pool }
    }

    fn conexao(
        &self,
    ) -> Result<
        diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>,
        RepositorioError,
    > {
        self.pool.get().map_err(|_| RepositorioError::Indisponivel)
    }
}

impl RepositorioConsultasDoCliente for ConsultasDoClienteRepositorioDiesel {
    fn listar(
        &self,
        escritorio_id: Uuid,
        cliente_id: Uuid,
        filtro: FiltroConsultasCliente,
        paginacao: Paginacao,
    ) -> Result<ListagemConsultasDoCliente, RepositorioError> {
        let mut conexao = self.conexao()?;

        // Tenancy na cláusula WHERE: `escritorio_id` vem do RequestContext
        // e `cliente_id` da rota — nenhum dos dois vem do corpo.
        let total = sql_query(
            "SELECT COUNT(*) AS total \
             FROM consultas_fiscais \
             WHERE escritorio_id = $1 \
               AND cliente_id = $2 \
               AND ($3::text IS NULL OR estado = $3)",
        )
        .bind::<SqlUuid, _>(escritorio_id)
        .bind::<SqlUuid, _>(cliente_id)
        .bind::<Nullable<Text>, _>(filtro.estado.as_deref())
        .get_result::<Contagem>(&mut conexao)
        .map(|contagem| contagem.total)
        .map_err(|_| RepositorioError::Dados)?;

        let itens = sql_query(format!(
            "SELECT {COLUNAS_CONSULTA} \
             FROM consultas_fiscais \
             WHERE escritorio_id = $1 \
               AND cliente_id = $2 \
               AND ($3::text IS NULL OR estado = $3) \
             ORDER BY criado_em DESC, id DESC \
             LIMIT $4 OFFSET $5"
        ))
        .bind::<SqlUuid, _>(escritorio_id)
        .bind::<SqlUuid, _>(cliente_id)
        .bind::<Nullable<Text>, _>(filtro.estado.as_deref())
        .bind::<BigInt, _>(paginacao.tamanho)
        .bind::<BigInt, _>(paginacao.deslocamento())
        .load::<LinhaConsulta>(&mut conexao)
        .map_err(|e| {
            tracing::error!("listar consultas do cliente: {e}");
            RepositorioError::Dados
        })?
        .into_iter()
        .map(Into::into)
        .collect();

        Ok(ListagemConsultasDoCliente {
            itens,
            total,
            pagina: paginacao.pagina,
            tamanho: paginacao.tamanho,
        })
    }

    fn situacao_fiscal(
        &self,
        escritorio_id: Uuid,
        cliente_id: Uuid,
    ) -> Result<Vec<SituacaoFiscalServico>, RepositorioError> {
        let mut conexao = self.conexao()?;

        sql_query(
            "SELECT DISTINCT ON (servico) \
                 servico, estado, ambiente, fonte_consulta, \
                 chave_consulta_mascarada, status_code, criado_em \
             FROM consultas_fiscais \
             WHERE escritorio_id = $1 AND cliente_id = $2 \
             ORDER BY servico, criado_em DESC, id DESC",
        )
        .bind::<SqlUuid, _>(escritorio_id)
        .bind::<SqlUuid, _>(cliente_id)
        .load::<LinhaResumo>(&mut conexao)
        .map(|linhas| linhas.into_iter().map(Into::into).collect())
        .map_err(|e| {
            tracing::error!("situacao fiscal do cliente: {e}");
            RepositorioError::Dados
        })
    }
}
impl From<LinhaConsulta> for ConsultaFiscalDoCliente {
    fn from(linha: LinhaConsulta) -> Self {
        Self {
            id: linha.id,
            produto: linha.produto,
            servico: linha.servico,
            verbo: linha.verbo,
            ambiente: linha.ambiente,
            estado: linha.estado,
            fonte_consulta: linha.fonte_consulta,
            chave_consulta_mascarada: linha.chave_consulta_mascarada,
            status_code: linha.status_code,
            criado_em: linha.criado_em,
        }
    }
}

impl From<LinhaResumo> for SituacaoFiscalServico {
    fn from(linha: LinhaResumo) -> Self {
        Self {
            servico: linha.servico,
            estado: linha.estado,
            ambiente: linha.ambiente,
            fonte_consulta: linha.fonte_consulta,
            chave_consulta_mascarada: linha.chave_consulta_mascarada,
            status_code: linha.status_code,
            criado_em: linha.criado_em,
        }
    }
}
