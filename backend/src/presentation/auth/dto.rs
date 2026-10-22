use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::application::auth::{AdministradorPublico, UsuarioPublico};

/// Login da plataforma do escritório: o slug seleciona o tenant.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub escritorio: String,
    pub email: String,
    pub senha: String,
}

/// Login do backoffice SaaS: sem slug de escritório (ADR-0010).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginAdminRequest {
    pub email: String,
    pub senha: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    pub access_token: String,
    pub expires_in: u64,
    pub usuario: UsuarioResponse,
}

/// O backoffice responde `usuario` para reutilizar o contrato da plataforma,
/// com papel fixo `ADMIN` e `escritorioId` `null`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginAdminResponse {
    pub access_token: String,
    pub expires_in: u64,
    pub usuario: AdministradorResponse,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsuarioAtualResponse {
    pub usuario: UsuarioResponse,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdministradorAtualResponse {
    pub usuario: AdministradorResponse,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsuarioResponse {
    pub id: Uuid,
    pub email: String,
    pub papel: String,
    pub escritorio_id: Uuid,
}

impl From<UsuarioPublico> for UsuarioResponse {
    fn from(usuario: UsuarioPublico) -> Self {
        Self {
            id: usuario.id,
            email: usuario.email,
            papel: usuario.papel.como_str().to_owned(),
            escritorio_id: usuario.escritorio_id,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdministradorResponse {
    pub id: Uuid,
    pub email: String,
    pub papel: String,
    pub escritorio_id: Option<Uuid>,
}

impl From<AdministradorPublico> for AdministradorResponse {
    fn from(administrador: AdministradorPublico) -> Self {
        Self {
            id: administrador.id,
            email: administrador.email,
            papel: "ADMIN".to_owned(),
            escritorio_id: None,
        }
    }
}
