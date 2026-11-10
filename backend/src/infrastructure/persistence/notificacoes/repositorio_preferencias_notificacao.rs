//! RepositÃ³rio Diesel de preferÃªncias de notificaÃ§Ã£o.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_types::{Bool, Timestamptz, Uuid as SqlUuid};
use uuid::Uuid;

use crate::domain::notificacoes::entidades::{NovoPreferenciaNotificacao, PreferenciaNotificacao};
use crate::domain::notificacoes::erros::ErroNotificacao;
use crate::domain::notificacoes::ports::RepositorioPreferenciasNotificacao;
use crate::infrastructure::persistence::pool::DbPool;

#[derive(Clone)]
pub struct PreferenciasNotificacaoDiesel {
    pool: DbPool,
}

impl PreferenciasNotificacaoDiesel {
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
struct PreferenciaRow {
    #[diesel(sql_type = SqlUuid)]
    id: Uuid,
    #[diesel(sql_type = SqlUuid)]
    usuario_id: Uuid,
    #[diesel(sql_type = Bool)]
    canal_email: bool,
    #[diesel(sql_type = Bool)]
    canal_whatsapp: bool,
    #[diesel(sql_type = Bool)]
    canal_in_app: bool,
    #[diesel(sql_type = diesel::sql_types::Jsonb)]
    tipos_ativados: serde_json::Value,
    #[diesel(sql_type = Timestamptz)]
    criado_em: DateTime<Utc>,
    #[diesel(sql_type = Timestamptz)]
    atualizado_em: DateTime<Utc>,
}

impl PreferenciaRow {
    fn para_entidade(self) -> PreferenciaNotificacao {
        let tipos: Vec<String> = self
            .tipos_ativados
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        PreferenciaNotificacao {
            id: self.id,
            usuario_id: self.usuario_id,
            canal_email: self.canal_email,
            canal_whatsapp: self.canal_whatsapp,
            canal_in_app: self.canal_in_app,
            tipos_ativados: tipos,
            criado_em: self.criado_em,
            atualizado_em: self.atualizado_em,
        }
    }
}

#[async_trait]
impl RepositorioPreferenciasNotificacao for PreferenciasNotificacaoDiesel {
    async fn buscar_por_usuario(
        &self,
        usuario_id: Uuid,
    ) -> Result<Option<PreferenciaNotificacao>, ErroNotificacao> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(
            move || -> Result<Option<PreferenciaNotificacao>, ErroNotificacao> {
                let mut conexao = pool
                    .get()
                    .map_err(|_| ErroNotificacao::Persistencia("pool".into()))?;
                let linha = diesel::sql_query(
                    "SELECT id, usuario_id, canal_email, canal_whatsapp, canal_in_app, \
                         tipos_ativados, criado_em, atualizado_em \
                  FROM preferencias_notificacao \
                  WHERE usuario_id = $1",
                )
                .bind::<SqlUuid, _>(usuario_id)
                .get_result::<PreferenciaRow>(&mut conexao)
                .optional()
                .map_err(|_| ErroNotificacao::Persistencia("buscar preferÃªncias".into()))?
                .map(|row| row.para_entidade())
                .ok_or(ErroNotificacao::PreferenciaNaoEncontrada)?;
                Ok(Some(linha))
            },
        )
        .await
        .map_err(|_| ErroNotificacao::Persistencia("spawn_blocking".into()))?
    }

    async fn criar_ou_atualizar(
        &self,
        preferencia: &NovoPreferenciaNotificacao,
    ) -> Result<PreferenciaNotificacao, ErroNotificacao> {
        let pool = self.pool.clone();
        let novo = preferencia.clone();
        tokio::task::spawn_blocking(move || -> Result<PreferenciaNotificacao, ErroNotificacao> {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroNotificacao::Persistencia("pool".into()))?;
            let tipos_json = serde_json::to_value(&novo.tipos_ativados)
                .map_err(|_| ErroNotificacao::Persistencia("serializar tipos".into()))?;
            let linha = diesel::sql_query(
                "INSERT INTO preferencias_notificacao \
                 (id, usuario_id, canal_email, canal_whatsapp, canal_in_app, \
                  tipos_ativados, criado_em, atualizado_em) \
                 VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW()) \
                 ON CONFLICT (usuario_id) DO UPDATE SET \
                  canal_email = EXCLUDED.canal_email, \
                  canal_whatsapp = EXCLUDED.canal_whatsapp, \
                  canal_in_app = EXCLUDED.canal_in_app, \
                  tipos_ativados = EXCLUDED.tipos_ativados, \
                  atualizado_em = NOW() \
                 RETURNING id, usuario_id, canal_email, canal_whatsapp, canal_in_app, \
                          tipos_ativados, criado_em, atualizado_em",
            )
            .bind::<SqlUuid, _>(Uuid::now_v7())
            .bind::<SqlUuid, _>(novo.usuario_id)
            .bind::<Bool, _>(novo.canal_email)
            .bind::<Bool, _>(novo.canal_whatsapp)
            .bind::<Bool, _>(novo.canal_in_app)
            .bind::<diesel::sql_types::Jsonb, _>(tipos_json)
            .get_result::<PreferenciaRow>(&mut conexao)
            .map_err(|_| ErroNotificacao::Persistencia("criar/atualizar preferÃªncias".into()))?
            .para_entidade();
            Ok(linha)
        })
        .await
        .map_err(|_| ErroNotificacao::Persistencia("spawn_blocking".into()))?
    }
}
