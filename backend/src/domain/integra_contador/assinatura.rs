use crate::domain::clientes::{CpfCnpj, TipoDocumento};
use chrono::NaiveDate;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TermoAutorizacao {
    pub data_assinatura: NaiveDate,
    pub vigencia: NaiveDate,
    pub destinatario_numero: String,
    pub destinatario_nome: String,
    pub assinado_por_numero: String,
    pub assinado_por_nome: String,
}

impl TermoAutorizacao {
    pub fn para_xml(&self) -> Result<String, ErroAssinatura> {
        if self.vigencia < self.data_assinatura {
            return Err(ErroAssinatura::TermoInvalido);
        }

        let destinatario =
            CpfCnpj::novo(&self.destinatario_numero).map_err(|_| ErroAssinatura::TermoInvalido)?;
        let assinante =
            CpfCnpj::novo(&self.assinado_por_numero).map_err(|_| ErroAssinatura::TermoInvalido)?;
        if destinatario.tipo() != TipoDocumento::Cnpj
            || self.destinatario_nome.trim().is_empty()
            || self.assinado_por_nome.trim().is_empty()
        {
            return Err(ErroAssinatura::TermoInvalido);
        }

        let destinatario_numero = escapar_atributo(&self.destinatario_numero)?;
        let destinatario_nome = escapar_atributo(&self.destinatario_nome)?;
        let assinado_por_numero = escapar_atributo(&self.assinado_por_numero)?;
        let assinado_por_nome = escapar_atributo(&self.assinado_por_nome)?;
        let tipo_assinante = match assinante.tipo() {
            TipoDocumento::Cpf => "PF",
            TipoDocumento::Cnpj => "PJ",
        };
        let data_assinatura = self.data_assinatura.format("%Y%m%d");
        let vigencia = self.vigencia.format("%Y%m%d");

        Ok(format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?><termoDeAutorizacao><dados>\
             <sistema id=\"API Integra Contador\"/><termo texto=\"{}\"/>\
             <avisoLegal texto=\"{}\"/><finalidade texto=\"{}\"/>\
             <dataAssinatura data=\"{data_assinatura}\"/><vigencia data=\"{vigencia}\"/>\
             <destinatario numero=\"{destinatario_numero}\" nome=\"{destinatario_nome}\" \
             tipo=\"PJ\" papel=\"contratante\"/><assinadoPor numero=\"{assinado_por_numero}\" \
             nome=\"{assinado_por_nome}\" tipo=\"{tipo_assinante}\" papel=\"autor pedido de dados\"/>\
             </dados></termoDeAutorizacao>",
            escapar_atributo(TERMO)?,
            escapar_atributo(AVISO_LEGAL)?,
            escapar_atributo(FINALIDADE)?
        ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ErroAssinatura {
    #[error("termo de autorização inválido")]
    TermoInvalido,
    #[error("o documento contém caractere incompatível com XML 1.0")]
    CaractereXmlInvalido,
    #[error("chave privada ou certificado inválido")]
    CertificadoInvalido,
    #[error("não foi possível validar a cadeia ICP-Brasil e a revogação")]
    ValidacaoCadeiaIndisponivel,
}

pub struct XmlAssinado {
    pub xml_base64: String,
    pub digest_value: String,
    pub signature_value: String,
    pub x509_certificate: String,
}

#[async_trait::async_trait]
pub trait AssinadorXml: Send + Sync {
    async fn assinar(
        &self,
        termo: &TermoAutorizacao,
        chave_privada_pkcs8_der: &mut [u8],
        certificado_der: &mut [u8],
    ) -> Result<XmlAssinado, ErroAssinatura>;
}

const TERMO: &str = "Autorizo a empresa CONTRATANTE, identificada neste termo de autorização como DESTINATÁRIO, a executar as requisições dos serviços web disponibilizados pela API INTEGRA CONTADOR, onde terei o papel de AUTOR PEDIDO DE DADOS no corpo da mensagem enviada na requisição do serviço web. Esse termo de autorização está assinado digitalmente com o certificado digital do PROCURADOR ou OUTORGADO DO CONTRIBUINTE responsável, identificado como AUTOR DO PEDIDO DE DADOS.";
const AVISO_LEGAL: &str = "O acesso a estas informações foi autorizado pelo próprio PROCURADOR ou OUTORGADO DO CONTRIBUINTE, responsável pela informação, via assinatura digital. É dever do destinatário da autorização e consumidor deste acesso observar a adoção de base legal para o tratamento dos dados recebidos conforme artigos 7º ou 11º da LGPD (Lei n.º 13.709, de 14 de agosto de 2018), aos direitos do titular dos dados (art. 9º, 17 e 18, da LGPD) e aos princípios que norteiam todos os tratamentos de dados no Brasil (art. 6º, da LGPD).";
const FINALIDADE: &str = "A finalidade única e exclusiva desse TERMO DE AUTORIZAÇÃO, é garantir que o CONTRATANTE apresente a API INTEGRA CONTADOR esse consentimento do PROCURADOR ou OUTORGADO DO CONTRIBUINTE assinado digitalmente, para que possa realizar as requisições dos serviços web da API INTEGRA CONTADOR em nome do AUTOR PEDIDO DE DADOS (PROCURADOR ou OUTORGADO DO CONTRIBUINTE).";

fn escapar_atributo(valor: &str) -> Result<String, ErroAssinatura> {
    let mut escapado = String::with_capacity(valor.len());
    for caractere in valor.chars() {
        if !matches!(
            caractere,
            '\u{9}' | '\u{A}' | '\u{D}' | '\u{20}'..='\u{D7FF}' | '\u{E000}'..='\u{FFFD}' | '\u{10000}'..='\u{10FFFF}'
        ) {
            return Err(ErroAssinatura::CaractereXmlInvalido);
        }
        match caractere {
            '&' => escapado.push_str("&amp;"),
            '<' => escapado.push_str("&lt;"),
            '>' => escapado.push_str("&gt;"),
            '"' => escapado.push_str("&quot;"),
            '\'' => escapado.push_str("&apos;"),
            _ => escapado.push(caractere),
        }
    }
    Ok(escapado)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn termo() -> TermoAutorizacao {
        TermoAutorizacao {
            data_assinatura: NaiveDate::from_ymd_opt(2026, 10, 20).expect("data válida"),
            vigencia: NaiveDate::from_ymd_opt(2027, 10, 20).expect("data válida"),
            destinatario_numero: "11222333000181".to_owned(),
            destinatario_nome: "Escritório & Associados".to_owned(),
            assinado_por_numero: "11222333000181".to_owned(),
            assinado_por_nome: "Responsável".to_owned(),
        }
    }

    #[test]
    fn gera_termo_xml_sem_perder_textos_fixos_ou_escapar_atributos() {
        let xml = termo().para_xml().expect("XML válido");
        assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(xml.contains("Escritório &amp; Associados"));
        assert!(xml.contains("artigos 7º ou 11º da LGPD"));
        assert!(!xml.contains('\n'));
    }

    #[test]
    fn rejeita_vigencia_anterior_a_assinatura() {
        let mut termo = termo();
        termo.vigencia = termo
            .data_assinatura
            .pred_opt()
            .expect("dia anterior válido");
        assert_eq!(termo.para_xml(), Err(ErroAssinatura::TermoInvalido));
    }

    #[test]
    fn rejeita_caractere_xml_invalido() {
        let mut termo = termo();
        termo.destinatario_nome.push('\u{1}');
        assert_eq!(termo.para_xml(), Err(ErroAssinatura::CaractereXmlInvalido));
    }
}
