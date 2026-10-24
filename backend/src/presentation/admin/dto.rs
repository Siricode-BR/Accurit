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

/// Query de paginação de `GET /admin/escritorios` (SPEC §10.3: `page`/`pageSize`).
#[derive(Debug, Clone, Default, Deserialize)]
pub struct PaginacaoQuery {
    #[serde(default)]
    pub page: Option<u32>,
    #[serde(rename = "pageSize", default)]
    pub page_size: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EscritorioResumoResponse {
    pub id: Uuid,
    pub slug: String,
    pub nome: String,
    pub modo_trial_serpro: bool,
    pub versao: i64,
    pub atualizado_em: chrono::DateTime<chrono::Utc>,
}

impl From<EscritorioResumo> for EscritorioResumoResponse {
    fn from(resumo: EscritorioResumo) -> Self {
        Self {
            id: resumo.id,
            slug: resumo.slug,
            nome: resumo.nome,
            modo_trial_serpro: resumo.modo_trial_serpro,
            versao: resumo.versao,
            atualizado_em: resumo.atualizado_em,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EscritoriosPaginadosResponse {
    pub itens: Vec<EscritorioResumoResponse>,
    pub page: u32,
    pub page_size: u32,
    pub total: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn default_da_paginacao_query_usa_opcionais() {
        let vazio = serde_json::from_str::<PaginacaoQuery>("{}").unwrap();
        assert!(vazio.page.is_none() && vazio.page_size.is_none());

        let preenchido =
            serde_json::from_str::<PaginacaoQuery>(r#"{"page":2,"pageSize":50}"#).unwrap();
        assert_eq!(preenchido.page, Some(2));
        assert_eq!(preenchido.page_size, Some(50));
    }

    #[test]
    fn serializa_resumo_com_modo_trial_e_versao() {
        let agora = chrono::Utc.with_ymd_and_hms(2026, 10, 24, 9, 0, 0).unwrap();
        let json = serde_json::to_string(&EscritorioResumoResponse {
            id: Uuid::nil(),
            slug: "contamais".into(),
            nome: "ContaMais".into(),
            modo_trial_serpro: true,
            versao: 3,
            atualizado_em: agora,
        })
        .unwrap();

        assert!(json.contains("\"modoTrialSerpro\":true"));
        assert!(json.contains("\"versao\":3"));
        assert!(json.contains("\"atualizadoEm\""));
        assert!(!json.contains("cnpj"));
    }
}
