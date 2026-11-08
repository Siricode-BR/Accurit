//! Entidades de domínio de notificações.
//! São objetos puros de negócio, sem dependência de Diesel ou Axum.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::notificacoes::enums::{
    CanalNotificacao, TipoNotificacao,
};

/// Notificação enviada a um usuário do escritório.
#[derive(Debug, Clone)]
pub struct Notificacao {
    pub id: Uuid,
    pub usuario_id: Uuid,
    pub escritorio_id: Uuid,
    pub canal: CanalNotificacao,
    pub tipo: TipoNotificacao,
    pub titulo: String,
    pub corpo: String,
    pub lida: bool,
    pub lida_em: Option<DateTime<Utc>>,
    pub referencia_tipo: Option<String>,
    pub referencia_id: Option<Uuid>,
    pub criado_em: DateTime<Utc>,
}

/// Dados para criação de uma nova notificação.
#[derive(Debug, Clone)]
pub struct NovoNotificacao {
    pub usuario_id: Uuid,
    pub escritorio_id: Uuid,
    pub canal: CanalNotificacao,
    pub tipo: TipoNotificacao,
    pub titulo: String,
    pub corpo: String,
    pub referencia_tipo: Option<String>,
    pub referencia_id: Option<Uuid>,
}

/// Preferências de notificação de um usuário.
#[derive(Debug, Clone)]
pub struct PreferenciaNotificacao {
    pub id: Uuid,
    pub usuario_id: Uuid,
    pub canal_email: bool,
    pub canal_whatsapp: bool,
    pub canal_in_app: bool,
    pub tipos_ativados: Vec<String>,
    pub criado_em: DateTime<Utc>,
    pub atualizado_em: DateTime<Utc>,
}

/// Dados para criação/atualização de preferências de notificação.
#[derive(Debug, Clone)]
pub struct NovoPreferenciaNotificacao {
    pub usuario_id: Uuid,
    pub canal_email: bool,
    pub canal_whatsapp: bool,
    pub canal_in_app: bool,
    pub tipos_ativados: Vec<String>,
}

/// Template de notificação.
#[derive(Debug, Clone)]
pub struct TemplateNotificacao {
    pub id: Uuid,
    pub tipo: TipoNotificacao,
    pub assunto: String,
    pub corpo_texto: String,
    pub corpo_html: Option<String>,
    pub variaveis: Vec<String>,
    pub ativo: bool,
    pub criado_em: DateTime<Utc>,
    pub atualizado_em: DateTime<Utc>,
}

/// Dados para criação de um novo template.
#[derive(Debug, Clone)]
pub struct NovoTemplateNotificacao {
    pub tipo: TipoNotificacao,
    pub assunto: String,
    pub corpo_texto: String,
    pub corpo_html: Option<String>,
    pub variaveis: Vec<String>,
    pub ativo: bool,
}
