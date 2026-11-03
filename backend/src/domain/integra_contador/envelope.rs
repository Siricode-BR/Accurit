//! Envelope comum do Integra Contador (ADR-0003).
//! Preserva `pedidoDados.dados` como string JSON.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Identificação do contratante/autor/contribuinte (somente dígitos).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Identificacao {
    /// CPF/CNPJ sem pontuação (11 ou 14 dígitos).
    pub numero: String,
    /// "PF" ou "PJ".
    pub tipo: String,
    /// Nome/razão social.
    pub nome: String,
}

impl Identificacao {
    pub fn new(numero: String, tipo: String, nome: String) -> Self {
        Self { numero, tipo, nome }
    }

    pub fn cpf(numero: String, nome: String) -> Self {
        Self::new(numero, "PF".to_owned(), nome)
    }

    pub fn cnpj(numero: String, nome: String) -> Self {
        Self::new(numero, "PJ".to_owned(), nome)
    }
}

/// Dados do pedido — `dados` é string JSON (pode ser XML em Base64 para assinatura).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PedidoDados {
    /// JSON string com os parâmetros do serviço ou `{"xml": "<base64>"}` para assinatura.
    pub dados: String,
}

/// Requisição completa do envelope Integra Contador.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegraContadorRequest {
    /// Identificação do contratante (quem contratou o serviço).
    pub contratante: Identificacao,
    /// Autor do pedido de dados (quem assina/autoriza).
    pub autor_pedido_dados: Identificacao,
    /// Contribuinte alvo da operação.
    pub contribuinte: Identificacao,
    /// Sistema alvo (ex.: "SICALC", "PROCURACOES", "PGDASD", "DCTFWEB", "SITFIS", "CAIXAPOSTAL", "AUTENTICAPROCURADOR").
    pub id_sistema: String,
    /// Serviço específico (ex.: "CONSULTAAPOIORECEITAS52", "OBTERPROCURACAO41", "ENVIOXMLASSINADO81").
    pub id_servico: String,
    /// Versão do sistema (ex.: "1", "1.0").
    pub versao_sistema: String,
    /// Dados do pedido.
    pub pedido_dados: PedidoDados,
    /// Token de autenticação do procurador (quando autor ≠ contratante).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub autenticar_procurador_token: Option<String>,
}

/// Resposta do envelope Integra Contador.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegraContadorResponse {
    /// Status do processamento.
    pub status: String,
    /// Mensagens de retorno.
    #[serde(default)]
    pub mensagens: Vec<String>,
    /// Dados de resposta (JSON string).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dados: Option<String>,
    /// Tempo estimado de espera para polling (em segundos).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tempo_espera: Option<u32>,
    /// Token de autenticação do procurador (retornado por ENVIOXMLASSINADO81).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub autenticar_procurador_token: Option<String>,
    /// Data/hora de expiração do token de procurador.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_hora_expiracao: Option<String>,
}

/// Envelope completo para transporte HTTP.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegraContadorEnvelope {
    pub request: IntegraContadorRequest,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<IntegraContadorResponse>,
}

/// Verbos suportados pelo Integra Contador.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VerboIntegraContador {
    Apoiar,
    Consultar,
    Declarar,
    Emitir,
    Monitorar,
}

impl VerboIntegraContador {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Apoiar => "APOIAR",
            Self::Consultar => "CONSULTAR",
            Self::Declarar => "DECLARAR",
            Self::Emitir => "EMITIR",
            Self::Monitorar => "MONITORAR",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "APOIAR" => Some(Self::Apoiar),
            "CONSULTAR" => Some(Self::Consultar),
            "DECLARAR" => Some(Self::Declarar),
            "EMITIR" => Some(Self::Emitir),
            "MONITORAR" => Some(Self::Monitorar),
            _ => None,
        }
    }

    /// Verifica se o verbo é mutável (requer idempotência e confirmação).
    pub fn e_mutavel(&self) -> bool {
        matches!(self, Self::Declarar | Self::Emitir)
    }

    /// Verifica se o verbo é bilhetado (conforme SPEC).
    pub fn e_bilhetado(&self) -> bool {
        !matches!(self, Self::Apoiar | Self::Monitorar)
    }
}

impl fmt::Display for VerboIntegraContador {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verbo_mutavel_declarar() {
        assert!(VerboIntegraContador::Declarar.e_mutavel());
        assert!(VerboIntegraContador::Emitir.e_mutavel());
        assert!(!VerboIntegraContador::Consultar.e_mutavel());
        assert!(!VerboIntegraContador::Apoiar.e_mutavel());
        assert!(!VerboIntegraContador::Monitorar.e_mutavel());
    }

    #[test]
    fn verbo_bilhetado() {
        assert!(VerboIntegraContador::Consultar.e_bilhetado());
        assert!(VerboIntegraContador::Declarar.e_bilhetado());
        assert!(VerboIntegraContador::Emitir.e_bilhetado());
        assert!(!VerboIntegraContador::Apoiar.e_bilhetado());
        assert!(!VerboIntegraContador::Monitorar.e_bilhetado());
    }

    #[test]
    fn envelope_serializacao() {
        let request = IntegraContadorRequest {
            contratante: Identificacao::cnpj("12345678000190".to_owned(), "Escritório Teste".to_owned()),
            autor_pedido_dados: Identificacao::cnpj("12345678000190".to_owned(), "Escritório Teste".to_owned()),
            contribuinte: Identificacao::cnpj("98765432000190".to_owned(), "Cliente Teste".to_owned()),
            id_sistema: "SICALC".to_owned(),
            id_servico: "CONSULTAAPOIORECEITAS52".to_owned(),
            versao_sistema: "1".to_owned(),
            pedido_dados: PedidoDados { dados: "{}".to_owned() },
            autenticar_procurador_token: None,
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("CONSULTAAPOIORECEITAS52"));
        assert!(json.contains("contratante"));
    }
}