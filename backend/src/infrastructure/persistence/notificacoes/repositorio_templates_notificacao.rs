//! RepositÃ³rio Diesel de templates de notificaÃ§Ã£o.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_types::{Bool, Text, Timestamptz, Uuid as SqlUuid};
use uuid::Uuid;

use crate::domain::notificacoes::entidades::TemplateNotificacao;
use crate::domain::notificacoes::enums::TipoNotificacao;
use crate::domain::notificacoes::erros::ErroNotificacao;
use crate::domain::notificacoes::ports::RepositorioTemplatesNotificacao;
use crate::infrastructure::persistence::pool::DbPool;

#[derive(Clone)]
pub struct TemplatesNotificacaoDiesel {
    pool: DbPool,
}

impl TemplatesNotificacaoDiesel {
    pub const fn novo(pool: DbPool) -> Self {
        Self { pool }
    }

    #[allow(dead_code)]
    fn conexao(
        &self,
    ) -> Result<
        diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>,
        ErroNotificacao,
    > {
        self.pool
            .get()
            .map_err(|_| ErroNotificacao::Persistencia("pool indispon\u{00ed}vel".into()))
    }
}

#[derive(QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct TemplateRow {
    #[diesel(sql_type = SqlUuid)]
    id: Uuid,
    #[diesel(sql_type = Text)]
    tipo: String,
    #[diesel(sql_type = Text)]
    assunto: String,
    #[diesel(sql_type = Text)]
    corpo_texto: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<Text>)]
    corpo_html: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Jsonb)]
    variaveis: serde_json::Value,
    #[diesel(sql_type = Bool)]
    ativo: bool,
    #[diesel(sql_type = Timestamptz)]
    criado_em: DateTime<Utc>,
    #[diesel(sql_type = Timestamptz)]
    atualizado_em: DateTime<Utc>,
}

impl TemplateRow {
    fn para_entidade(self) -> Result<TemplateNotificacao, ErroNotificacao> {
        let variaveis: Vec<String> = self.variaveis
            .as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();
        let tipo = match self.tipo.as_str() {
            "pendencia_divida_ativa" => TipoNotificacao::PendenciaDividaAtiva,
            "mensagem_caixa_postal" => TipoNotificacao::MensagemCaixaPostal,
            "procuracao_vencendo" => TipoNotificacao::ProcuracaoVencendo,
            "certificado_a1_vencendo" => TipoNotificacao::CertificadoA1Vencendo,
            "das_vencendo" => TipoNotificacao::DasVencendo,
            "falha_rotina" => TipoNotificacao::FalhaRotina,
            _ => return Err(ErroNotificacao::Persistencia("tipo invÃ¡lido".into())),
        };
        Ok(TemplateNotificacao {
            id: self.id,
            tipo,
            assunto: self.assunto,
            corpo_texto: self.corpo_texto,
            corpo_html: self.corpo_html,
            variaveis: variaveis,
            ativo: self.ativo,
            criado_em: self.criado_em,
            atualizado_em: self.atualizado_em,
        })
    }
}

#[async_trait]
impl RepositorioTemplatesNotificacao for TemplatesNotificacaoDiesel {
    async fn buscar_por_tipo(
        &self,
        tipo: TipoNotificacao,
    ) -> Result<Option<TemplateNotificacao>, ErroNotificacao> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || -> Result<Option<TemplateNotificacao>, ErroNotificacao> {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroNotificacao::Persistencia("pool".into()))?;
            let linha = diesel::sql_query(
                "SELECT id, tipo, assunto, corpo_texto, corpo_html, variaveis, \
                         ativo, criado_em, atualizado_em \
                  FROM templates_notificacao \
                  WHERE tipo = $1 AND ativo = TRUE",
            )
            .bind::<Text, _>(tipo.as_str())
            .get_result::<TemplateRow>(&mut conexao)
            .optional()
            .map_err(|_| ErroNotificacao::Persistencia("buscar template".into()))?
            .map(|row| row.para_entidade())
            .transpose();
            linha
        })
        .await
        .map_err(|_| ErroNotificacao::Persistencia("spawn_blocking".into()))?
    }

    async fn listar_ativos(&self) -> Result<Vec<TemplateNotificacao>, ErroNotificacao> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || -> Result<Vec<TemplateNotificacao>, ErroNotificacao> {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroNotificacao::Persistencia("pool".into()))?;
            let linhas = diesel::sql_query(
                "SELECT id, tipo, assunto, corpo_texto, corpo_html, variaveis, \
                         ativo, criado_em, atualizado_em \
                  FROM templates_notificacao \
                  WHERE ativo = TRUE \
                  ORDER BY tipo",
            )
            .load::<TemplateRow>(&mut conexao)
            .map_err(|_| ErroNotificacao::Persistencia("listar templates".into()))?
            .into_iter()
            .map(|row| row.para_entidade())
            .collect::<Result<Vec<_>, _>>()?;
            Ok(linhas)
        })
        .await
        .map_err(|_| ErroNotificacao::Persistencia("spawn_blocking".into()))?
    }
}
