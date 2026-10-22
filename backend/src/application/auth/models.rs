use std::net::IpAddr;

use uuid::Uuid;

use crate::domain::auth::{Audiencia, Papel, Usuario};

#[derive(Debug, Clone)]
pub struct PoliticaAuth {
    pub access_ttl_secs: u64,
    pub refresh_ttl_admin_secs: u64,
    pub refresh_ttl_client_secs: u64,
    pub max_failed_attempts: u32,
    pub lockout_secs: u64,
}

impl PoliticaAuth {
    pub fn ttl_refresh(&self, audiencia: Audiencia) -> u64 {
        match audiencia {
            Audiencia::Admin => self.refresh_ttl_admin_secs,
            Audiencia::Client => self.refresh_ttl_client_secs,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UsuarioPublico {
    pub id: Uuid,
    pub email: String,
    pub papel: Papel,
    pub escritorio_id: Uuid,
}

impl From<&Usuario> for UsuarioPublico {
    fn from(usuario: &Usuario) -> Self {
        Self {
            id: usuario.id,
            email: usuario.email.clone(),
            papel: usuario.papel,
            escritorio_id: usuario.escritorio_id,
        }
    }
}

/// Perfil público do gestor de tenants — sempre papel `ADMIN`, sem escritório.
#[derive(Debug, Clone)]
pub struct AdministradorPublico {
    pub id: Uuid,
    pub email: String,
}

#[derive(Debug, Clone)]
pub struct SessaoEmitida {
    pub access_token: String,
    pub expires_in: u64,
    pub refresh_token: String,
    pub refresh_ttl_secs: u64,
    pub usuario: UsuarioPublico,
}

#[derive(Debug, Clone)]
pub struct SessaoAdminEmitida {
    pub access_token: String,
    pub expires_in: u64,
    pub refresh_token: String,
    pub refresh_ttl_secs: u64,
    pub administrador: AdministradorPublico,
}

#[derive(Debug, Clone)]
pub struct LoginCommand {
    pub escritorio: String,
    pub email: String,
    pub senha: String,
    pub audiencia: Audiencia,
    pub ip: Option<IpAddr>,
    pub user_agent: Option<String>,
}

/// Login do backoffice: sem slug de escritório (ADR-0010).
#[derive(Debug, Clone)]
pub struct LoginAdminCommand {
    pub email: String,
    pub senha: String,
    pub ip: Option<IpAddr>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RenovarCommand {
    pub refresh_token: String,
    pub audiencia: Audiencia,
    pub ip: Option<IpAddr>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RenovarAdminCommand {
    pub refresh_token: String,
    pub ip: Option<IpAddr>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LogoutCommand {
    pub refresh_token: String,
    pub audiencia: Audiencia,
    pub ip: Option<IpAddr>,
}

#[derive(Debug, Clone)]
pub struct LogoutAdminCommand {
    pub refresh_token: String,
    pub ip: Option<IpAddr>,
}

#[derive(Debug, Clone, Copy)]
pub struct UsuarioAtualCommand {
    pub escritorio_id: Uuid,
    pub usuario_id: Uuid,
    pub audiencia: Audiencia,
}

#[derive(Debug, Clone, Copy)]
pub struct AdministradorAtualCommand {
    pub administrador_id: Uuid,
}

pub(crate) fn normalizar_slug(valor: &str) -> String {
    valor.trim().to_ascii_lowercase()
}

pub(crate) fn normalizar_email(valor: &str) -> String {
    valor.trim().to_ascii_lowercase()
}

pub(crate) fn truncar_user_agent(valor: Option<String>) -> Option<String> {
    valor.map(|valor| {
        valor
            .chars()
            .filter(|caractere| !caractere.is_control())
            .take(512)
            .collect()
    })
}
