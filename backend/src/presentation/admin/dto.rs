use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::application::admin::EscritorioResumo;

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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EscritorioResumoResponse {
    pub id: Uuid,
    pub slug: String,
    pub nome: String,
    pub cnpj: Option<String>,
    pub modo_trial_serpro: bool,
    pub criado_em: chrono::DateTime<chrono::Utc>,
}

impl From<EscritorioResumo> for EscritorioResumoResponse {
    fn from(resumo: EscritorioResumo) -> Self {
        Self {
            id: resumo.id,
            slug: resumo.slug,
            nome: resumo.nome,
            cnpj: resumo.cnpj,
            modo_trial_serpro: resumo.modo_trial_serpro,
            criado_em: resumo.criado_em,
        }
    }
}
