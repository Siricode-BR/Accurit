//! Envelope comum do Integra Contador (SPEC §5.2–§5.3).
//!
//! Contrato fixado pelo SPEC:
//! - `Identificacao`: `numero` sem pontuação e `tipo` numérico (`1` = CPF,
//!   `2` = CNPJ) — nunca `"PF"`/`"PJ"`.
//! - `PedidoDados` carrega `idSistema`, `idServico`, `versaoSistema` e o
//!   `dados` como **string JSON** específica do serviço.
//! - `IntegraContadorRequest` tem apenas os três atores + `pedidoDados`;
//!   o `autenticar_procurador_token` vai no **header** da requisição
//!   (transporte), nunca no corpo.
//! - Resposta: `status` numérico (`i32`) e `mensagens` estruturadas
//!   (`MensagemSerpro`), nunca `Vec<String>`.

use serde::{Deserialize, Serialize};
use std::fmt;

/// `1` = CPF (11 dígitos); `2` = CNPJ (14 dígitos) — SPEC §5.2.
pub const TIPO_CPF: u8 = 1;
pub const TIPO_CNPJ: u8 = 2;

/// Identificação do contratante/autor/contribuinte (SPEC §5.2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Identificacao {
    /// CPF (11 dígitos) ou CNPJ (14 dígitos), sem pontuação.
    pub numero: String,
    /// 1 = CPF; 2 = CNPJ.
    pub tipo: u8,
}

impl Identificacao {
    /// Constrói com validação estrita de dígitos: 11 → CPF (`1`),
    /// 14 → CNPJ (`2`). Qualquer outra combinação é rejeitada.
    pub fn novo(numero: impl Into<String>) -> Result<Self, String> {
        let numero: String = numero.into();
        let so_digitos = numero.chars().all(|c| c.is_ascii_digit());
        let tipo = match numero.len() {
            11 if so_digitos => TIPO_CPF,
            14 if so_digitos => TIPO_CNPJ,
            _ => {
                return Err(format!(
                    "documento deve ter 11 (CPF) ou 14 (CNPJ) dígitos: {numero:?}"
                ));
            }
        };
        Ok(Self { numero, tipo })
    }

    pub fn cpf(numero: impl Into<String>) -> Result<Self, String> {
        let identificacao = Self::novo(numero)?;
        if identificacao.tipo != TIPO_CPF {
            return Err("documento não tem 11 dígitos de CPF".to_owned());
        }
        Ok(identificacao)
    }

    pub fn cnpj(numero: impl Into<String>) -> Result<Self, String> {
        let identificacao = Self::novo(numero)?;
        if identificacao.tipo != TIPO_CNPJ {
            return Err("documento não tem 14 dígitos de CNPJ".to_owned());
        }
        Ok(identificacao)
    }

    /// Se `tipo` diverge do comprimento de `numero`, monta com o tipo corrigido.
    /// Usado ao decodificar respostas tolerantes da SERPRO.
    pub fn coerente(numero: String, tipo: u8) -> Self {
        let corrigido = match numero.len() {
            11 => TIPO_CPF,
            14 => TIPO_CNPJ,
            _ => tipo,
        };
        Self {
            numero,
            tipo: corrigido,
        }
    }
}

/// Dados do pedido — `dados` é **string JSON** específica do serviço
/// (SPEC §5.2; para ENVIOXMLASSINADO81 contém `{"xml":"<base64>"}`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PedidoDados {
    /// Ex.: PGDASD, DCTFWEB, SITFIS, SICALC, PROCURACOES, AUTENTICAPROCURADOR.
    pub id_sistema: String,
    /// Ex.: CONSDECLARACAO13, GERARGUIA31, OBTERPROCURACAO41.
    pub id_servico: String,
    pub versao_sistema: String,
    /// JSON específico do serviço, serializado como string.
    pub dados: String,
}

/// Requisição completa do envelope Integra Contador (SPEC §5.2).
///
/// Em produção (SAPI), o token do procurador é enviado no header
/// `autenticar-procurador-token` pelo cliente HTTP — não existe no corpo.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegraContadorRequest {
    /// CPF/CNPJ do escritório contratante.
    pub contratante: Identificacao,
    /// Certificado A1 ou procurador autorizado.
    pub autor_pedido_dados: Identificacao,
    /// Cliente da contabilidade.
    pub contribuinte: Identificacao,
    pub pedido_dados: PedidoDados,
}

/// Mensagem estruturada de retorno da SERPRO (SPEC §5.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MensagemSerpro {
    pub codigo: String,
    pub descricao: String,
    #[serde(rename = "tipo")]
    pub tipo: Option<String>,
}

/// Envelope de resposta do Integra Contador (SPEC §5.3).
///
/// Semânticas de transporte (202/204/304/504) são tratadas pela camada
/// de infraestrutura, que instancia estados extras fora do envelope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegraContadorResponse {
    pub status: i32,
    pub mensagens: Vec<MensagemSerpro>,
    /// JSON específico do serviço, serializado como string.
    #[serde(default)]
    pub dados: Option<String>,
}

impl IntegraContadorResponse {
    /// Concatena as descrições das mensagens para logs redigidos.
    pub fn descricoes(&self) -> String {
        self.mensagens
            .iter()
            .map(|m| m.descricao.as_str())
            .collect::<Vec<_>>()
            .join("; ")
    }

    /// Código da primeira mensagem, quando houver (ex.:
    /// `AcessoNegado-ICGERENCIADOR`).
    pub fn primeiro_codigo(&self) -> Option<&str> {
        self.mensagens.first().map(|m| m.codigo.as_str())
    }
}

/// Verbos operacionais do Integra Contador (SPEC §5.1).
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
    /// Caminho do endpoint relativo à base (SPEC §5.1): `POST /Apoiar` etc.
    pub const fn path(&self) -> &'static str {
        match self {
            Self::Apoiar => "Apoiar",
            Self::Consultar => "Consultar",
            Self::Declarar => "Declarar",
            Self::Emitir => "Emitir",
            Self::Monitorar => "Monitorar",
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

    /// Verbo mutável: exige idempotência e confirmação explícita
    /// (SPEC §5.4).
    pub const fn e_mutavel(&self) -> bool {
        matches!(self, Self::Declarar | Self::Emitir)
    }

    /// Bilhetado conforme SPEC/rule: `Apoiar` e `Monitorar` não são
    /// registrados como chamada operacional.
    pub const fn e_bilhetado(&self) -> bool {
        !matches!(self, Self::Apoiar | Self::Monitorar)
    }
}

impl fmt::Display for VerboIntegraContador {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.path())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identificacao_valida_cpf_e_cnpj() {
        let cpf = Identificacao::novo("12345678909").expect("CPF válido");
        assert_eq!(cpf.tipo, TIPO_CPF);
        let cnpj = Identificacao::novo("12345678000190").expect("CNPJ válido");
        assert_eq!(cnpj.tipo, TIPO_CNPJ);
    }

    #[test]
    fn identificacao_rejeita_tamanho_invalido() {
        assert!(Identificacao::novo("123").is_err());
        assert!(Identificacao::novo("12345678901234567890").is_err());
        // Pontuado é rejeitado na borda de domínio (higienização é na
        // presentation; aqui só dígitos).
        assert!(Identificacao::novo("123.456.789-09").is_err());
    }

    #[test]
    fn tipo_determinado_pelo_comprimento_quando_diverge() {
        let corrigido = Identificacao::coerente("12345678909".to_owned(), TIPO_CNPJ);
        assert_eq!(corrigido.tipo, TIPO_CPF);
    }

    #[test]
    fn envelopa_serializacao_camel_case_com_dados_como_string() {
        let request = IntegraContadorRequest {
            contratante: Identificacao::novo("12345678000190").expect("contratante"),
            autor_pedido_dados: Identificacao::novo("12345678000190").expect("autor"),
            contribuinte: Identificacao::novo("98765432000190").expect("contribuinte"),
            pedido_dados: PedidoDados {
                id_sistema: "PGDASD".to_owned(),
                id_servico: "CONSDECLARACAO13".to_owned(),
                versao_sistema: "1".to_owned(),
                dados: "{\"pa\":202601,\"tipoPeriodoApuracao\":\"2\"}".to_owned(),
            },
        };
        let json = serde_json::to_string(&request).expect("serializa");
        assert!(json.contains("\"contratante\""));
        assert!(json.contains("\"autorPedidoDados\""));
        assert!(json.contains("\"pedidoDados\""));
        assert!(json.contains("\"idSistema\":\"PGDASD\""));
        assert!(json.contains("\"versaoSistema\":\"1\""));
        // `dados` sempre string JSON:
        assert!(json.contains("\"dados\":\"{\\\"pa\\\""));
        // `tipo` numérico:
        assert!(json.contains("\"tipo\":2"));
    }

    #[test]
    fn response_de_mensagens_estruturadas() {
        let corpo: IntegraContadorResponse = serde_json::from_str(
            r#"{"status":200,"mensagens":[{"codigo":"OK","descricao":"Feito","tipo":"INFORMACAO"}],"dados":"{}"}"#,
        )
        .expect("desserializa");
        assert_eq!(corpo.status, 200);
        assert_eq!(corpo.primeiro_codigo(), Some("OK"));
        assert!(corpo.descricoes().contains("Feito"));
    }

    #[test]
    fn verbos_mutaveis_e_bilhetados() {
        assert!(VerboIntegraContador::Declarar.e_mutavel());
        assert!(VerboIntegraContador::Emitir.e_mutavel());
        assert!(VerboIntegraContador::Declarar.e_bilhetado());
        assert!(!VerboIntegraContador::Apoiar.e_bilhetado());
        assert!(!VerboIntegraContador::Monitorar.e_bilhetado());
        assert_eq!(VerboIntegraContador::Consultar.path(), "Consultar");
    }
}
