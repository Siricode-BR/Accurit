//! Fase C — DTOs de notificações.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::notificacoes::entidades::{Notificacao, PreferenciaNotificacao};

/// Requisição para criar uma notificação.
#[derive(Debug, Deserialize)]
pub struct RequisicaoCriarNotificacao {
    pub usuario_id: Uuid,
    pub canal: String,
    pub tipo: String,
    pub titulo: String,
    pub corpo: String,
    pub referencia_tipo: Option<String>,
    pub referencia_id: Option<Uuid>,
}

/// Resposta de uma notificação.
#[derive(Debug, Serialize)]
pub struct RespostaNotificacao {
    pub id: Uuid,
    pub usuario_id: Uuid,
    pub escritorio_id: Uuid,
    pub canal: String,
    pub tipo: String,
    pub titulo: String,
    pub corpo: String,
    pub lida: bool,
    pub lida_em: Option<DateTime<Utc>>,
    pub referencia_tipo: Option<String>,
    pub referencia_id: Option<Uuid>,
    pub criado_em: DateTime<Utc>,
}

/// Requisição para marcar notificação como lida.
#[derive(Debug, Deserialize)]
pub struct RequisicaoMarcarLida {
    pub id: Uuid,
}

/// Requisição para preferências de notificação.
#[derive(Debug, Deserialize)]
pub struct RequisicaoPreferenciaNotificacao {
    pub canal_email: bool,
    pub canal_whatsapp: bool,
    pub canal_in_app: bool,
    pub tipos_ativados: Vec<String>,
}

/// Resposta de preferências de notificação.
#[derive(Debug, Serialize)]
pub struct RespostaPreferenciaNotificacao {
    pub id: Uuid,
    pub usuario_id: Uuid,
    pub canal_email: bool,
    pub canal_whatsapp: bool,
    pub canal_in_app: bool,
    pub tipos_ativados: Vec<String>,
    pub criado_em: DateTime<Utc>,
    pub atualizado_em: DateTime<Utc>,
}

/// Contagem de notificações não lidas.
#[derive(Debug, Serialize)]
pub struct RespostaContagemNaoLidas {
    pub nao_lidas: i64,
}

/// Resposta de template de notificação.
#[derive(Debug, Serialize)]
pub struct RespostaTemplateNotificacao {
    pub id: Uuid,
    pub tipo: String,
    pub assunto: String,
    pub corpo_texto: String,
    pub corpo_html: Option<String>,
    pub variaveis: Vec<String>,
    pub ativo: bool,
    pub criado_em: DateTime<Utc>,
    pub atualizado_em: DateTime<Utc>,
}

impl From<Notificacao> for RespostaNotificacao {
    fn from(n: Notificacao) -> Self {
        Self {
            id: n.id,
            usuario_id: n.usuario_id,
            escritorio_id: n.escritorio_id,
            canal: n.canal.as_str().to_owned(),
            tipo: n.tipo.as_str().to_owned(),
            titulo: n.titulo,
            corpo: n.corpo,
            lida: n.lida,
            lida_em: n.lida_em,
            referencia_tipo: n.referencia_tipo,
            referencia_id: n.referencia_id,
            criado_em: n.criado_em,
        }
    }
}

impl From<PreferenciaNotificacao> for RespostaPreferenciaNotificacao {
    fn from(p: PreferenciaNotificacao) -> Self {
        Self {
            id: p.id,
            usuario_id: p.usuario_id,
            canal_email: p.canal_email,
            canal_whatsapp: p.canal_whatsapp,
            canal_in_app: p.canal_in_app,
            tipos_ativados: p.tipos_ativados,
            criado_em: p.criado_em,
            atualizado_em: p.atualizado_em,
        }
    }
}
