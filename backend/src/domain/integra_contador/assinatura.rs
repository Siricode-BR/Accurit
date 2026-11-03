//! Assinatura digital XML (XMLDSig) para o Termo de Autorização
//! (SPEC §7.1, ADR-0004).
//!
//! >= Formato fixado pelo SPEC:
//! - raiz `termoDeAutorizacao`; namespace XMLDSig **default**
//!   (`<Signature xmlns="http://www.w3.org/2000/09/xmldsig#">`), sem
//!   prefixo `ds:`;
//! - `Reference URI=""` único com transformações Enveloped Signature e
//!   C14N; RSA-SHA256; digest SHA-256; `KeyInfo` com apenas o certificado
//!   do usuário final (EndCertOnly);
//! - textos fixos copiados **exatamente** de
//!   `docs/Serpro/Integra-Contador/dados-dominio-autentica-procurador.md`
//!   (qualquer divergência invalida o termo).

use serde::{Deserialize, Serialize};

/// Termo de Autorização para ENVIOXMLASSINADO81.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TermoAutorizacao {
    pub sistema_id: String,
    pub termo_texto: String,
    pub aviso_legal_texto: String,
    pub finalidade_texto: String,
    /// formato `YYYYMMDD`; `dataAssinatura` ≤ hoje (SPEC §7.1).
    pub data_assinatura: String,
    /// formato `YYYYMMDD`; `vigencia` ≥ hoje.
    pub vigencia: String,
    pub destinatario_numero: String, // CNPJ do contratante
    pub destinatario_nome: String,
    pub destinatario_tipo: String,  // "PJ"
    pub destinatario_papel: String, // "contratante"
    pub assinado_por_numero: String,
    pub assinado_por_nome: String,
    pub assinado_por_tipo: String,  // "PF"/"PJ"
    pub assinado_por_papel: String, // "autor pedido de dados"
}

impl TermoAutorizacao {
    /// Cria o termo com os textos fixos oficiais transcritos do "Dados de
    /// Domínio do Autentica Procurador" (docs/Serpro/Integra-Contador/).
    #[allow(clippy::too_many_arguments)]
    pub fn novo_oficial(
        contratante_cnpj: impl Into<String>,
        contratante_nome: impl Into<String>,
        autor_numero: impl Into<String>,
        autor_nome: impl Into<String>,
        autor_tipo: impl Into<String>,
        data_assinatura: impl Into<String>,
        vigencia: impl Into<String>,
    ) -> Self {
        Self {
            sistema_id: "API Integra Contador".to_owned(),
            termo_texto: TEXTO_TERMO_OFICIAL.to_owned(),
            aviso_legal_texto: TEXTO_AVISO_LEGAL_OFICIAL.to_owned(),
            finalidade_texto: TEXTO_FINALIDADE_OFICIAL.to_owned(),
            data_assinatura: data_assinatura.into(),
            vigencia: vigencia.into(),
            destinatario_numero: contratante_cnpj.into(),
            destinatario_nome: contratante_nome.into(),
            destinatario_tipo: "PJ".to_owned(),
            destinatario_papel: "contratante".to_owned(),
            assinado_por_numero: autor_numero.into(),
            assinado_por_nome: autor_nome.into(),
            assinado_por_tipo: autor_tipo.into(),
            assinado_por_papel: "autor pedido de dados".to_owned(),
        }
    }

    /// XML do termo (sem assinatura). Dados em <dados> com atributos
    /// `id`/`texto`/`texto`/`texto` conforme SPEC §7.1.
    pub fn para_xml(&self) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?><termoDeAutorizacao><dados><sistema id="{sistema}"/><termo texto="{termo}"/><avisoLegal texto="{aviso}"/><finalidade texto="{finalidade}"/><dataAssinatura data="{data}"/><vigencia data="{vigencia}"/><destinatario numero="{dest_num}" nome="{dest_nome}" tipo="{dest_tipo}" papel="{dest_papel}"/><assinadoPor numero="{ass_num}" nome="{ass_nome}" tipo="{ass_tipo}" papel="{ass_papel}"/></dados></termoDeAutorizacao>"#,
            sistema = escape_xml(&self.sistema_id),
            termo = escape_xml(&self.termo_texto),
            aviso = escape_xml(&self.aviso_legal_texto),
            finalidade = escape_xml(&self.finalidade_texto),
            data = escape_xml(&self.data_assinatura),
            vigencia = escape_xml(&self.vigencia),
            dest_num = escape_xml(&self.destinatario_numero),
            dest_nome = escape_xml(&self.destinatario_nome),
            dest_tipo = escape_xml(&self.destinatario_tipo),
            dest_papel = escape_xml(&self.destinatario_papel),
            ass_num = escape_xml(&self.assinado_por_numero),
            ass_nome = escape_xml(&self.assinado_por_nome),
            ass_tipo = escape_xml(&self.assinado_por_tipo),
            ass_papel = escape_xml(&self.assinado_por_papel),
        )
    }
}

/// XML assinado (transporte de resultado).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XmlAssinado {
    pub xml_base64: String,
    pub digest_value: String,
    pub signature_value: String,
    pub x509_certificate: String,
}

/// Assinador XML com certificado A1 (port de domínio; a implementação
/// concreta fica em `infrastructure/integra_contador`).
pub trait AssinadorXml: Send + Sync {
    /// Assina o XML do termo de autorização e devolve o XML assinado em
    /// Base64 (sem quebras de linha).
    fn assinar_termo(
        &self,
        termo: &TermoAutorizacao,
        chave_privada_pkcs8_der: &[u8],
        certificado_der: &[u8],
    ) -> Result<XmlAssinado, ErroAssinatura>;
}

/// Erros de assinatura XML — única fonte (sem duplicatas no domínio).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ErroAssinatura {
    #[error("certificado não é ICP-Brasil série A (A1/A3)")]
    CertificadoNaoIcpBrasil,
    #[error("certificado expirado em {valido_ate}")]
    CertificadoExpirado { valido_ate: String },
    #[error("certificado revogado")]
    CertificadoRevogado,
    #[error("certificado não é RSA 2048 bits (SPEC §7.1)")]
    ChaveRsaInvalida,
    #[error("cadeia de certificação inválida: {motivo}")]
    CadeiaInvalida { motivo: String },
    #[error("digest SHA-256 divergente")]
    DigestDivergente,
    #[error("assinatura RSA-SHA256 inválida")]
    AssinaturaInvalida,
    #[error("canonicalização C14N falhou: {motivo}")]
    CanonicalizacaoFalhou { motivo: String },
    #[error("XML malformado: {motivo}")]
    XmlMalformado { motivo: String },
    #[error("texto do termo diverge do oficial: {campo}")]
    TermoDivergente { campo: String },
    #[error("chave privada não disponível para assinatura")]
    ChavePrivadaIndisponivel,
    #[error("falha ao carregar certificado: {motivo}")]
    CertificadoIlegivel { motivo: String },
}

/// Textos fixos oficiais — transcritos de
/// `docs/Serpro/Integra-Contador/dados-dominio-autentica-procurador.md`
/// (documento oficial de 2026-02-13). Qualquer divergência invalida o
/// termo (SPEC §7.1).
pub const TEXTO_TERMO_OFICIAL: &str = "Autorizo a empresa CONTRATANTE, identificada neste termo de autorização como DESTINATÁRIO, a executar as requisições dos serviços web disponibilizados pela API INTEGRA CONTADOR, onde terei o papel de AUTOR PEDIDO DE DADOS no corpo da mensagem enviada na requisição do serviço web. Esse termo de autorização está assinado digitalmente com o certificado digital do PROCURADOR ou OUTORGADO DO CONTRIBUINTE responsável, identificado como AUTOR DO PEDIDO DE DADOS.";

pub const TEXTO_AVISO_LEGAL_OFICIAL: &str = "O acesso a estas informações foi autorizado pelo próprio PROCURADOR ou OUTORGADO DO CONTRIBUINTE, responsável pela informação, via assinatura digital. É dever do destinatário da autorização e consumidor deste acesso observar a adoção de base legal para o tratamento dos dados recebidos conforme artigos 7º ou 11º da LGPD (Lei n.º 13.709, de 14 de agosto de 2018), aos direitos do titular dos dados (art. 9º, 17 e 18, da LGPD) e aos princípios que norteiam todos os tratamentos de dados no Brasil (art. 6º, da LGPD).";

pub const TEXTO_FINALIDADE_OFICIAL: &str = "A finalidade única e exclusiva desse TERMO DE AUTORIZAÇÃO, é garantir que o CONTRATANTE apresente a API INTEGRA CONTADOR esse consentimento do PROCURADOR ou OUTORGADO DO CONTRIBUINTE assinado digitalmente, para que possa realizar as requisições dos serviços web da API INTEGRA CONTADOR em nome do AUTOR PEDIDO DE DADOS (PROCURADOR ou OUTORGADO DO CONTRIBUINTE).";

/// Escapa caracteres especiais para atributos XML (espaços preservados
/// entre aspas; sem caracteres Unicode invisíveis).
pub fn escape_xml(input: &str) -> String {
    let mut resposta = String::with_capacity(input.len());
    for caractere in input.chars() {
        match caractere {
            '&' => resposta.push_str("&amp;"),
            '<' => resposta.push_str("&lt;"),
            '>' => resposta.push_str("&gt;"),
            '"' => resposta.push_str("&quot;"),
            '\'' => resposta.push_str("&apos;"),
            outro => resposta.extend(outro.to_string().chars()),
        }
    }
    resposta
}

/// Remove indentação e quebras de linha: o XML final vai todo em uma
/// linha (SPEC §7.1 — remover indentação/quebras antes do Base64).
pub fn normalizar_xml_para_assinatura(xml: &str) -> String {
    xml.lines()
        .map(str::trim)
        .filter(|linha| !linha.is_empty())
        .collect::<Vec<_>>()
        .join("")
}

/// Base64 RFC 4648 sem quebras de linha.
pub fn base64_sem_quebras(data: &[u8]) -> String {
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    STANDARD.encode(data)
}

/// Decodificação Base64.
pub fn base64_decode(data: &str) -> Result<Vec<u8>, base64::DecodeError> {
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    STANDARD.decode(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn termo() -> TermoAutorizacao {
        TermoAutorizacao::novo_oficial(
            "12345678000190",
            "Escritório Teste Ltda",
            "12345678909",
            "Procurador Teste",
            "PF",
            "20261020",
            "20271231",
        )
    }

    #[test]
    fn textos_oficiais_exatamente_transcritos() {
        assert!(TEXTO_TERMO_OFICIAL.starts_with("Autorizo a empresa CONTRATANTE"));
        assert!(TEXTO_TERMO_OFICIAL.contains("AUTOR DO PEDIDO DE DADOS"));
        assert!(TEXTO_AVISO_LEGAL_OFICIAL.contains("13.709, de 14 de agosto de 2018"));
        assert!(TEXTO_FINALIDADE_OFICIAL.starts_with("A finalidade única e exclusiva"));
    }

    #[test]
    fn xml_uma_unica_linha_com_campos() {
        let xml = termo().para_xml();
        assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(xml.contains("<termoDeAutorizacao>"));
        assert!(xml.contains("<sistema id=\"API Integra Contador\"/>"));
        assert!(xml.contains("tipo=\"PF\" papel=\"autor pedido de dados\""));
        assert!(xml.contains("20271231"));
        assert!(!xml.contains('\n'), "sem quebras de linha");
    }

    #[test]
    fn escape_xml_escapa_reservados() {
        assert_eq!(escape_xml("A & B"), "A &amp; B");
        assert_eq!(escape_xml("<tag>"), "&lt;tag&gt;");
        assert_eq!(escape_xml("\"aspas'\""), "&quot;aspas&apos;&quot;");
    }

    #[test]
    fn normalizar_remove_indentacao() {
        let xml = "<root>\n  <child>v</child>\n</root>";
        assert_eq!(
            normalizar_xml_para_assinatura(xml),
            "<root><child>v</child></root>"
        );
    }

    #[test]
    fn base64_sem_quebras_funciona() {
        let dados = b"teste";
        let codificado = base64_sem_quebras(dados);
        assert!(!codificado.contains('\n'));
        assert_eq!(base64_decode(&codificado).expect("decodifica"), dados);
    }
}
