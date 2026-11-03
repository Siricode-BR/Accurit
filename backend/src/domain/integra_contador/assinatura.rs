//! Assinatura digital XML (XMLDSig) para Termo de Autorização.
//! Conforme ADR-0004 e SPEC §7.1.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Termo de Autorização para ENVIOXMLASSINADO81.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TermoAutorizacao {
    pub sistema_id: String,
    pub termo_texto: String,
    pub aviso_legal_texto: String,
    pub finalidade_texto: String,
    pub data_assinatura: String, // formato YYYYMMDD
    pub vigencia: String,        // formato YYYYMMDD
    pub destinatario_numero: String, // CNPJ do contratante
    pub destinatario_nome: String,
    pub destinatario_tipo: String,   // "PJ"
    pub destinatario_papel: String,  // "contratante"
    pub assinado_por_numero: String, // CPF/CNPJ do autor do pedido
    pub assinado_por_nome: String,
    pub assinado_por_tipo: String,   // "PF" ou "PJ"
    pub assinado_por_papel: String,  // "autor pedido de dados"
}

impl TermoAutorizacao {
    /// Cria um termo de autorização com os textos fixos oficiais.
    pub fn novo_oficial(
        contratante_cnpj: String,
        contratante_nome: String,
        autor_numero: String,
        autor_nome: String,
        autor_tipo: String,
        data_assinatura: String,
        vigencia: String,
    ) -> Self {
        // Textos fixos conforme docs/Serpro/Integra-Contador/dados-dominio-autentica-procurador.md
        Self {
            sistema_id: "API Integra Contador".to_owned(),
            termo_texto: TEXTO_TERMO_OFICIAL.to_owned(),
            aviso_legal_texto: TEXTO_AVISO_LEGAL_OFICIAL.to_owned(),
            finalidade_texto: TEXTO_FINALIDADE_OFICIAL.to_owned(),
            data_assinatura,
            vigencia,
            destinatario_numero: contratante_cnpj,
            destinatario_nome: contratante_nome,
            destinatario_tipo: "PJ".to_owned(),
            destinatario_papel: "contratante".to_owned(),
            assinado_por_numero: autor_numero,
            assinado_por_nome: autor_nome,
            assinado_por_tipo: autor_tipo,
            assinado_por_papel: "autor pedido de dados".to_owned(),
        }
    }

    /// Gera o XML do termo (sem assinatura).
    pub fn to_xml(&self) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<termoDeAutorizacao>
  <dados>
    <sistema id="{}"/>
    <termo texto="{}"/>
    <avisoLegal texto="{}"/>
    <finalidade texto="{}"/>
    <dataAssinatura data="{}"/>
    <vigencia data="{}"/>
    <destinatario numero="{}" nome="{}" tipo="{}" papel="{}"/>
    <assinadoPor numero="{}" nome="{}" tipo="{}" papel="{}"/>
  </dados>
</termoDeAutorizacao>"#,
            self.sistema_id,
            escape_xml(&self.termo_texto),
            escape_xml(&self.aviso_legal_texto),
            escape_xml(&self.finalidade_texto),
            self.data_assinatura,
            self.vigencia,
            self.destinatario_numero,
            escape_xml(&self.destinatario_nome),
            self.destinatario_tipo,
            self.destinatario_papel,
            self.assinado_por_numero,
            escape_xml(&self.assinado_por_nome),
            self.assinado_por_tipo,
            self.assinado_por_papel,
        )
    }
}

/// XML assinado com envelope XMLDSig.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XmlAssinado {
    pub xml_base64: String,
    pub digest_value: String,
    pub signature_value: String,
    pub x509_certificate: String,
}

/// Assinador XML com certificado A1.
pub trait AssinadorXml: Send + Sync {
    /// Assina o XML do termo de autorização.
    /// Retorna o XML assinado em Base64 (sem quebras de linha).
    fn assinar_termo(
        &self,
        termo: &TermoAutorizacao,
        certificado_pfx: &[u8],
        senha: &str,
    ) -> Result<XmlAssinado, ErroAssinatura>;

    /// Valida um XML assinado (verifica digest, assinatura e cadeia ICP-Brasil).
    fn validar_xml_assinado(&self, xml_assinado: &str) -> Result<(), ErroAssinatura>;
}

/// Erros de assinatura XML.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ErroAssinatura {
    #[error("certificado não é ICP-Brasil série A (A1/A3)")]
    CertificadoNaoIcpBrasil,
    #[error("certificado expirado em {valido_ate}")]
    CertificadoExpirado { valido_ate: String },
    #[error("certificado revogado")]
    CertificadoRevogado,
    #[error("cadeia de certificação inválida: {motivo}")]
    CadeiaInvalida { motivo: String },
    #[error("digest SHA-256 divergente")]
    DigestDivergente,
    #[error("assinatura RSA-SHA256 inválida")]
    AssinaturaInvalida,
    #[error("canonicalização C14N falhou: {motivo}")]
    CanonicalizacaoFalhou { motivo: String },
    #[error("transformação Enveloped Signature falhou: {motivo}")]
    TransformacaoFalhou { motivo: String },
    #[error("XML malformado: {motivo}")]
    XmlMalformado { motivo: String },
    #[error("texto do termo diverge do oficial: {campo}")]
    TermoDivergente { campo: String },
    #[error("chave privada não disponível para assinatura")]
    ChavePrivadaIndisponivel,
    #[error("falha ao carregar certificado PFX: {motivo}")]
    CarregarPfxFalhou { motivo: String },
    #[error("senha do certificado incorreta")]
    SenhaIncorreta,
}

/// Textos fixos oficiais do Termo de Autorização.
/// Fonte: docs/Serpro/Integra-Contador/dados-dominio-autentica-procurador.md
const TEXTO_TERMO_OFICIAL: &str = "TERMO DE AUTORIZAÇÃO PARA ACESSO A DADOS FISCAIS";

const TEXTO_AVISO_LEGAL_OFICIAL: &str = "AVISO LEGAL: O acesso a dados fiscais é regido pela Lei Complementar nº 105/2001, Lei nº 13.709/2018 (LGPD) e normas da RFB. O uso indevido sujeita o infrator às sanções civis, administrativas e penais cabíveis.";

const TEXTO_FINALIDADE_OFICIAL: &str = "FINALIDADE: Autorização para que o procurador acesse dados fiscais do contribuinte outorgante junto à RFB, exclusivamente para fins de cumprimento de obrigações acessórias e consultas autorizadas.";

/// Escapa caracteres especiais para XML.
fn escape_xml(input: &str) -> String {
    input
        .replace('&', "&")
        .replace('<', "<")
        .replace('>', ">")
        .replace('"', """)
        .replace('\'', "&apos;")
}

/// Remove indentação e quebras de linha do XML para assinatura.
pub fn normalizar_xml_para_assinatura(xml: &str) -> String {
    xml.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("")
}

/// Codifica em Base64 sem quebras de linha (RFC 4648).
pub fn base64_sem_quebras(data: &[u8]) -> String {
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    STANDARD.encode(data)
}

/// Decodifica Base64.
pub fn base64_decode(data: &str) -> Result<Vec<u8>, base64::DecodeError> {
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    STANDARD.decode(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn termo_autorizacao_gera_xml_valido() {
        let termo = TermoAutorizacao::novo_oficial(
            "12345678000190".to_owned(),
            "Escritório Teste Ltda".to_owned(),
            "12345678000190".to_owned(),
            "Escritório Teste Ltda".to_owned(),
            "PJ".to_owned(),
            "20261020".to_owned(),
            "20271231".to_owned(),
        );
        let xml = termo.to_xml();
        assert!(xml.contains("<termoDeAutorizacao>"));
        assert!(xml.contains("12345678000190"));
        assert!(xml.contains("Escritório Teste Ltda"));
        assert!(xml.contains("20261020"));
        assert!(xml.contains("20271231"));
    }

    #[test]
    fn escape_xml_funciona() {
        assert_eq!(escape_xml("A & B"), "A & B");
        assert_eq!(escape_xml("<tag>"), "<tag>");
        assert_eq!(escape_xml("\"aspas\""), ""aspas"");
    }

    #[test]
    fn normalizar_xml_remove_espacos() {
        let xml = r#"<root>
  <child>valor</child>
</root>"#;
        let normalizado = normalizar_xml_para_assinatura(xml);
        assert_eq!(normalizado, "<root><child>valor</child></root>");
    }

    #[test]
    fn base64_sem_quebras_funciona() {
        let data = b"teste";
        let encoded = base64_sem_quebras(data);
        assert!(!encoded.contains('\n'));
        assert!(!encoded.contains('\r'));
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }
}