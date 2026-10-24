use serde::Deserialize;
use serde::Serialize;

use crate::domain::serpro::ambiente::AmbienteSerpro;

/// PATCH /admin/escritorios/{id}/serpro/trial — o escritório alvo vem no
/// **path** (D7 de DECISOES-PENDENTES); o corpo só decide o modo e a versão
/// esperada da concorrência otimista.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlterarModoTrialRequest {
    pub modo_trial_serpro: bool,
    pub versao: i64,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlterarModoTrialResponse {
    pub modo_trial_serpro: bool,
    pub ambiente_efetivo: AmbienteSerpro,
    pub versao_atual: i64,
    pub atualizado_em: chrono::DateTime<chrono::Utc>,
}

/// GET /serpro/ambiente (portal) — somente o ambiente efetivo e o instante
/// da última alteração (D8 de DECISOES-PENDENTES; minimização de dados).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AmbienteResponse {
    pub ambiente_efetivo: AmbienteSerpro,
    pub atualizado_em: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn deserializa_alterar_modo_request_sem_escritorio() {
        let json = r#"{
            "modoTrialSerpro": false,
            "versao": 1
        }"#;
        let req: AlterarModoTrialRequest = serde_json::from_str(json).unwrap();
        assert!(!req.modo_trial_serpro);
        assert_eq!(req.versao, 1);
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

    #[test]
    fn ambiente_response_contem_apenas_ambiente_e_timestamp() {
        let resp = AmbienteResponse {
            ambiente_efetivo: AmbienteSerpro::Trial,
            atualizado_em: chrono::Utc
                .with_ymd_and_hms(2026, 10, 20, 12, 0, 0)
                .unwrap(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"ambienteEfetivo\":\"TRIAL\""));
        assert!(json.contains("\"atualizadoEm\""));
        assert_eq!(
            json,
            r#"{"ambienteEfetivo":"TRIAL","atualizadoEm":"2026-10-20T12:00:00Z"}"#
        );
    }
}
