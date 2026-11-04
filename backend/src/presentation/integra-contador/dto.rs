use serde::Deserialize;
use serde::Serialize;

use crate::domain::integra_contador::envelope::VerboIntegraContador;
use crate::domain::serpro::ambiente::AmbienteSerpro;

/// GET /api/v1/serpro/integra-contador/status - status do sistema
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusResponse {
    pub ambiente_efetivo: AmbienteSerpro,
    pub producao_habilitada: bool,
    pub assinatura_habilitada: bool,
    pub motivo_bloqueio_producao: String,
    pub motivo_bloqueio_assinatura: String,
    pub servicos: Vec<ServicoIntegraContadorStatus>,
}

/// Status de um serviço no catálogo
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServicoIntegraContadorStatus {
    pub id_sistema: String,
    pub id_servico: String,
    pub verbo: String,
    pub versao_sistema: Option<String>,
    pub habilitado: bool,
}

/// Request PATCH /api/v1/serpro/integra-contador/modo-trial
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModoTrialRequest {
    pub modo_trial_serpro: bool,
    pub versao: i64,
}

/// Response PATCH /api/v1/serpro/integra-contador/modo-trial
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModoTrialResponse {
    pub modo_trial_serpro: bool,
    pub ambiente_efetivo: AmbienteSerpro,
    pub versao_atual: i64,
    pub atualizado_em: chrono::DateTime<chrono::Utc>,
}

/// DTO para execução de operação
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecucaoRequest {
    pub verbo: VerboIntegraContador,
    pub id_sistema: String,
    pub id_servico: String,
    pub versao_sistema: String,
    pub dados: String,
    pub confirmacao_explicita: bool,
    pub chave_idempotencia: Option<String>,
}

/// Response da execução da operação
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecucaoResponse {
    pub ambiente: String,
    pub verbo: String,
    pub id_sistema: String,
    pub id_servico: String,
    pub estado: String,
    pub status_serpro: u16,
    pub mensagem: String,
    pub referencia_resposta: Option<String>,
    pub tempo_espera_segundos: Option<u64>,
    pub etag: Option<String>,
    pub expires: Option<String>,
    pub codigos_mensagem: Vec<String>,
}