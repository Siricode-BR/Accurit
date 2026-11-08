//! Repositório Diesel de notificações.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_types::{Bool, Nullable, Text, Timestamptz, Uuid as SqlUuid};
use uuid::Uuid;

use crate::domain::notificacoes::entidades::{
    Notificacao, NovoNotificacao,
};
use crate::domain::notificacoes::enums::{
    CanalNotificacao, TipoNotificacao,
};
use crate::domain::notificacoes::erros::ErroNotificacao;
use crate::domain::notificacoes::ports::RepositorioNotificacoes;
use crate::infrastructure::persistence::pool::DbPool;

#[derive(Clone)]
pub struct NotificacoesDiesel {
    pool: DbPool,
}

impl NotificacoesDiesel {
    pub const fn novo(pool: DbPool) -> Self {
        Self { pool }
    }

    fn conexao(
        &self,
    ) -> Result<
        diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>,
        ErroNotificacao,
    > {
        self.pool
            .get()
            .map_err(|_| ErroNotificacao::Persistencia("pool indisponível".into()))
    }
}

#[derive(QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct NotificacaoRow {
    #[diesel(sql_type = SqlUuid)]
    id: Uuid,
    #[diesel(sql_type = SqlUuid)]
    usuario_id: Uuid,
    #[diesel(sql_type = SqlUuid)]
    escritorio_id: Uuid,
    #[diesel(sql_type = Text)]
    canal: String,
    #[diesel(sql_type = Text)]
    tipo: String,
    #[diesel(sql_type = Text)]
    titulo: String,
    #[diesel(sql_type = Text)]
    corpo: String,
    #[diesel(sql_type = Bool)]
    lida: bool,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    lida_em: Option<DateTime<Utc>>,
    #[diesel(sql_type = Nullable<Text>)]
    referencia_tipo: Option<String>,
    #[diesel(sql_type = Nullable<SqlUuid>)]
    referencia_id: Option<Uuid>,
    #[diesel(sql_type = Timestamptz)]
    criado_em: DateTime<Utc>,
}

impl NotificacaoRow {
    fn para_entidade(self) -> Result<Notificacao, ErroNotificacao> {
        let canal = match self.canal.as_str() {
            "in_app" => CanalNotificacao::InApp,
            "email" => CanalNotificacao::Email,
            "whatsapp" => CanalNotificacao::WhatsApp,
            _ => return Err(ErroNotificacao::Persistencia("canal inválido".into())),
        };
        let tipo = match self.tipo.as_str() {
            "pendencia_divida_ativa" => TipoNotificacao::PendenciaDividaAtiva,
            "mensagem_caixa_postal" => TipoNotificacao::MensagemCaixaPostal,
            "procuracao_vencendo" => TipoNotificacao::ProcuracaoVencendo,
            "certificado_a1_vencendo" => TipoNotificacao::CertificadoA1Vencendo,
            "das_vencendo" => TipoNotificacao::DasVencendo,
            "falha_rotina" => TipoNotificacao::FalhaRotina,
            _ => return Err(ErroNotificacao::Persistencia("tipo inválido".into())),
        };
        Ok(Notificacao {
            id: self.id,
            usuario_id: self.usuario_id,
            escritorio_id: self.escritorio_id,
            canal,
            tipo,
            titulo: self.titulo,
            corpo: self.corpo,
            lida: self.lida,
            lida_em: self.lida_em,
            referencia_tipo: self.referencia_tipo,
            referencia_id: self.referencia_id,
            criado_em: self.criado_em,
        })
    }
}

#[async_trait]
impl RepositorioNotificacoes for NotificacoesDiesel {
    async fn criar(
        &self,
        notificacao: &NovoNotificacao,
    ) -> Result<Notificacao, ErroNotificacao> {
        let pool = self.pool.clone();
        let novo = notificacao.clone();
        tokio::task::spawn_blocking(move || -> Result<Notificacao, ErroNotificacao> {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroNotificacao::Persistencia("pool".into()))?;
            let linha = diesel::sql_query(
                "INSERT INTO notificacoes \
                 (id, usuario_id, escritorio_id, canal, tipo, titulo, corpo, lida, \
                  referencia_tipo, referencia_id) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, FALSE, $8, $9) \
                 RETURNING id, usuario_id, escritorio_id, canal, tipo, titulo, corpo, \
                          lida, lida_em, referencia_tipo, referencia_id, criado_em",
            )
            .bind::<SqlUuid, _>(Uuid::now_v7())
            .bind::<SqlUuid, _>(novo.usuario_id)
            .bind::<SqlUuid, _>(novo.escritorio_id)
            .bind::<Text, _>(novo.canal.as_str())
            .bind::<Text, _>(novo.tipo.as_str())
            .bind::<Text, _>(novo.titulo)
            .bind::<Text, _>(novo.corpo)
            .bind::<Nullable<Text>, _>(novo.referencia_tipo)
            .bind::<Nullable<SqlUuid>, _>(novo.referencia_id)
            .get_result::<NotificacaoRow>(&mut conexao)
            .map_err(|_| ErroNotificacao::Persistencia("inserir notificação".into()))?
            .para_entidade();
            linha
        })
        .await
        .map_err(|_| ErroNotificacao::Persistencia("spawn_blocking".into()))?
    }

    async fn buscar_por_id(
        &self,
        id: Uuid,
    ) -> Result<Option<Notificacao>, ErroNotificacao> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || -> Result<Option<Notificacao>, ErroNotificacao> {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroNotificacao::Persistencia("pool".into()))?;
            let linha = diesel::sql_query(
                "SELECT id, usuario_id, escritorio_id, canal, tipo, titulo, corpo, \
                         lida, lida_em, referencia_tipo, referencia_id, criado_em \
                  FROM notificacoes \
                  WHERE id = $1",
            )
            .bind::<SqlUuid, _>(id)
            .get_result::<NotificacaoRow>(&mut conexao)
            .optional()
            .map_err(|_| ErroNotificacao::Persistencia("buscar notificação".into()))?
            .map(|row| row.para_entidade())
            .transpose();
            linha
        })
        .await
        .map_err(|_| ErroNotificacao::Persistencia("spawn_blocking".into()))?
    }

    async fn listar_por_usuario(
        &self,
        usuario_id: Uuid,
        apenas_nao_lidas: bool,
        limite: i64,
    ) -> Result<Vec<Notificacao>, ErroNotificacao> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || -> Result<Vec<Notificacao>, ErroNotificacao> {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroNotificacao::Persistencia("pool".into()))?;
            let query = if apenas_nao_lidas {
                "SELECT id, usuario_id, escritorio_id, canal, tipo, titulo, corpo, \
                         lida, lida_em, referencia_tipo, referencia_id, criado_em \
                  FROM notificacoes \
                  WHERE usuario_id = $1 AND lida = FALSE \
                  ORDER BY criado_em DESC \
                  LIMIT $2"
            } else {
                "SELECT id, usuario_id, escritorio_id, canal, tipo, titulo, corpo, \
                         lida, lida_em, referencia_tipo, referencia_id, criado_em \
                  FROM notificacoes \
                  WHERE usuario_id = $1 \
                  ORDER BY criado_em DESC \
                  LIMIT $2"
            };
            let linhas = diesel::sql_query(query)
                .bind::<SqlUuid, _>(usuario_id)
                .bind::<diesel::sql_types::BigInt, _>(limite)
                .load::<NotificacaoRow>(&mut conexao)
                .map_err(|_| ErroNotificacao::Persistencia("listar notificações".into()))?
                .into_iter()
                .map(|row| row.para_entidade())
                .collect::<Result<Vec<_>, _>>()?;
            Ok(linhas)
        })
        .await
        .map_err(|_| ErroNotificacao::Persistencia("spawn_blocking".into()))?
    }

    async fn listar_por_escritorio(
        &self,
        escritorio_id: Uuid,
        apenas_nao_lidas: bool,
        limite: i64,
    ) -> Result<Vec<Notificacao>, ErroNotificacao> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || -> Result<Vec<Notificacao>, ErroNotificacao> {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroNotificacao::Persistencia("pool".into()))?;
            let query = if apenas_nao_lidas {
                "SELECT id, usuario_id, escritorio_id, canal, tipo, titulo, corpo, \
                         lida, lida_em, referencia_tipo, referencia_id, criado_em \
                  FROM notificacoes \
                  WHERE escritorio_id = $1 AND lida = FALSE \
                  ORDER BY criado_em DESC \
                  LIMIT $2"
            } else {
                "SELECT id, usuario_id, escritorio_id, canal, tipo, titulo, corpo, \
                         lida, lida_em, referencia_tipo, referencia_id, criado_em \
                  FROM notificacoes \
                  WHERE escritorio_id = $1 \
                  ORDER BY criado_em DESC \
                  LIMIT $2"
            };
            let linhas = diesel::sql_query(query)
                .bind::<SqlUuid, _>(escritorio_id)
                .bind::<diesel::sql_types::BigInt, _>(limite)
                .load::<NotificacaoRow>(&mut conexao)
                .map_err(|_| ErroNotificacao::Persistencia("listar notificações por escritório".into()))?
                .into_iter()
                .map(|row| row.para_entidade())
                .collect::<Result<Vec<_>, _>>()?;
            Ok(linhas)
        })
        .await
        .map_err(|_| ErroNotificacao::Persistencia("spawn_blocking".into()))?
    }

    async fn marcar_como_lida(
        &self,
        id: Uuid,
        usuario_id: Uuid,
    ) -> Result<(), ErroNotificacao> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || -> Result<(), ErroNotificacao> {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroNotificacao::Persistencia("pool".into()))?;
            let atualizados = diesel::sql_query(
                "UPDATE notificacoes \
                 SET lida = TRUE, lida_em = NOW() \
                 WHERE id = $1 AND usuario_id = $2 AND lida = FALSE",
            )
            .bind::<SqlUuid, _>(id)
            .bind::<SqlUuid, _>(usuario_id)
            .execute(&mut conexao)
            .map_err(|_| ErroNotificacao::Persistencia("marcar como lida".into()))?;
            if atualizados == 0 {
                return Err(ErroNotificacao::NotificacaoNaoEncontrada);
            }
            Ok(())
        })
        .await
        .map_err(|_| ErroNotificacao::Persistencia("spawn_blocking".into()))?
    }

    async fn marcar_todas_como_lidas(
        &self,
        usuario_id: Uuid,
    ) -> Result<usize, ErroNotificacao> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || -> Result<usize, ErroNotificacao> {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroNotificacao::Persistencia("pool".into()))?;
            let atualizados = diesel::sql_query(
                "UPDATE notificacoes \
                 SET lida = TRUE, lida_em = NOW() \
                 WHERE usuario_id = $1 AND lida = FALSE",
            )
            .bind::<SqlUuid, _>(usuario_id)
            .execute(&mut conexao)
            .map_err(|_| ErroNotificacao::Persistencia("marcar todas como lidas".into()))?;
            Ok(atualizados)
        })
        .await
        .map_err(|_| ErroNotificacao::Persistencia("spawn_blocking".into()))?
    }

    async fn contar_nao_lidas(&self, usuario_id: Uuid) -> Result<i64, ErroNotificacao> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || -> Result<i64, ErroNotificacao> {
            let mut conexao = pool
                .get()
                .map_err(|_| ErroNotificacao::Persistencia("pool".into()))?;
            let total = diesel::sql_query(
                "SELECT COUNT(*) AS total FROM notificacoes \
                 WHERE usuario_id = $1 AND lida = FALSE",
            )
            .bind::<SqlUuid, _>(usuario_id)
            .get_result::<Contagem>(&mut conexao)
            .map_err(|_| ErroNotificacao::Persistencia("contar não lidas".into()))?
            .total;
            Ok(total)
        })
        .await
        .map_err(|_| ErroNotificacao::Persistencia("spawn_blocking".into()))?
    }
}

#[derive(QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct Contagem {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    total: i64,
}
