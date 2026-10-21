use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::application::auth::UsuarioPublico;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub escritorio: String,
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsuarioAtualResponse {
    pub usuario: UsuarioResponse,
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
