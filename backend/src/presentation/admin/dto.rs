use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProvisionarTenantRequest {
    pub slug: String,
    pub nome: String,
    pub email_titular: String,
    pub cpf_titular: String,
    pub senha_titular: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProvisionarTenantResponse {
    pub escritorio_id: Uuid,
    pub slug: String,
    pub titular_id: Uuid,
    pub email_titular: String,
}