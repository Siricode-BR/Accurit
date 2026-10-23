use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::serpro::ambiente::AmbienteSerpro;

/// GET /admin/serpro/configuracao?escritorioId=... — o escritório alvo vai
/// na query: o frontoffice tem rota própria (SPEC §2.1) e o token de
/// backoffice não tem `escritorio_id` (ADR-0010, D3).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObterConfiguracaoQuery {
    pub escritorio_id: Uuid,
}

/// PATCH /admin/serpro/trial — `escritorio_id` alvo vem no corpo.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlterarModoTrialRequest {
    pub escritorio_id: Uuid,
    pub modo_trial_serpro: bool,
    pub versao: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SerproConfiguracaoResponse {
    pub escritorio_id: Uuid,
    pub modo_trial_serpro: bool,
    pub ambiente_efetivo: AmbienteSerpro,
    pub versao: i64,
    pub atualizado_em: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlterarModoTrialResponse {
    pub modo_trial_serpro: bool,
    pub ambiente_efetivo: AmbienteSerpro,
    pub versao_atual: i64,
    pub atualizado_em: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn deserializa_obter_query_camel_case() {
        let query: ObterConfiguracaoQuery =
            serde_json::from_str(r#"{"escritorioId": "00000000-0000-0000-0000-000000000001"}"#)
                .unwrap();
        assert_eq!(
            query.escritorio_id.to_string(),
            "00000000-0000-0000-0000-000000000001"
        );
    }

    #[test]
    fn deserializa_alterar_modo_request_com_escritorio() {
        let json = r#"{
            "escritorioId": "00000000-0000-0000-0000-000000000001",
            "modoTrialSerpro": false,
            "versao": 1
        }"#;
        let req: AlterarModoTrialRequest = serde_json::from_str(json).unwrap();
        assert!(!req.modo_trial_serpro);
        assert_eq!(req.versao, 1);
    }

    #[test]
    fn serializa_configuracao_camel_case() {
        let resp = SerproConfiguracaoResponse {
            escritorio_id: Uuid::nil(),
            modo_trial_serpro: true,
            ambiente_efetivo: AmbienteSerpro::Trial,
            versao: 1,
            atualizado_em: chrono::Utc
                .with_ymd_and_hms(2026, 10, 20, 12, 0, 0)
                .unwrap(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"escritorioId\""));
        assert!(json.contains("\"modoTrialSerpro\""));
        assert!(json.contains("\"ambienteEfetivo\":\"TRIAL\""));
    }

    #[test]
    fn serializa_alterar_modo_camel_case() {
        let resp = AlterarModoTrialResponse {
            modo_trial_serpro: false,
            ambiente_efetivo: AmbienteSerpro::Producao,
            versao_atual: 2,
            atualizado_em: chrono::Utc
                .with_ymd_and_hms(2026, 10, 20, 12, 0, 0)
                .unwrap(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"versaoAtual\":2"));
        assert!(json.contains("\"ambienteEfetivo\":\"PRODUCAO\""));
    }
}
