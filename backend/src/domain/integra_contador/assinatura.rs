use crate::domain::clientes::{CpfCnpj, TipoDocumento};
use chrono::NaiveDate;

/// Representa os três atores no modelo Software House para o Termo de Autorização.
/// Conforme documentação SERPRO Integra Contador - Autentica Procurador.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtoresTermoAutorizacao {
    /// Contratante da API (nossa plataforma SaaS) - CNPJ da Software House
    pub contratante_numero: String,
    pub contratante_nome: String,
    /// Autor do Pedido de Dados (Escritório de Contabilidade / Procurador) - CNPJ do escritório
    pub autor_pedido_dados_numero: String,
    pub autor_pedido_dados_nome: String,
    /// Contribuinte (Cliente do escritório) - CPF/CNPJ do cliente
    pub contribuinte_numero: String,
    pub contribuinte_nome: String,
}

impl AtoresTermoAutorizacao {
    pub fn validar(&self) -> Result<(), ErroAssinatura> {
        let contratante = CpfCnpj::novo(&self.contratante_numero)
            .map_err(|_| ErroAssinatura::TermoInvalido)?;
        let autor = CpfCnpj::novo(&self.autor_pedido_dados_numero)
            .map_err(|_| ErroAssinatura::TermoInvalido)?;
        CpfCnpj::novo(&self.contribuinte_numero).map_err(|_| ErroAssinatura::TermoInvalido)?;
        if contratante.tipo() != TipoDocumento::Cnpj {
            return Err(ErroAssinatura::TermoInvalido);
        }
        if autor.tipo() != TipoDocumento::Cnpj {
            return Err(ErroAssinatura::TermoInvalido);
        }
        if self.contratante_nome.trim().is_empty()
            || self.autor_pedido_dados_nome.trim().is_empty()
            || self.contribuinte_nome.trim().is_empty()
        {
            return Err(ErroAssinatura::TermoInvalido);
        }
        Ok(())
    }
}

/// Termo de Autorização completo para o endpoint AUTENTICAPROCURADOR/ENVIOXMLASSINADO81.
/// Contém os três CNPJs/CPFs exigidos pelo SERPRO no modelo Software House.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TermoAutorizacao {
    pub data_assinatura: NaiveDate,
    pub vigencia: NaiveDate,
    pub atores: AtoresTermoAutorizacao,
}

impl TermoAutorizacao {
    /// Gera o XML do Termo de Autorização com a estrutura de 3 atores exigida pelo SERPRO
    /// para o modelo Software House (AUTENTICAPROCURADOR).
    pub fn para_xml(&self) -> Result<String, ErroAssinatura> {
        if self.vigencia < self.data_assinatura {
            return Err(ErroAssinatura::TermoInvalido);
        }

        self.atores.validar()?;

        let contratante_numero = escapar_atributo(&self.atores.contratante_numero)?;
        let contratante_nome = escapar_atributo(&self.atores.contratante_nome)?;
        let autor_numero = escapar_atributo(&self.atores.autor_pedido_dados_numero)?;
        let autor_nome = escapar_atributo(&self.atores.autor_pedido_dados_nome)?;
        let contribuinte_numero = escapar_atributo(&self.atores.contribuinte_numero)?;
        let contribuinte_nome = escapar_atributo(&self.atores.contribuinte_nome)?;

        let contratante_doc = CpfCnpj::novo(&self.atores.contratante_numero).unwrap();
        let autor_doc = CpfCnpj::novo(&self.atores.autor_pedido_dados_numero).unwrap();
        let contribuinte_doc = CpfCnpj::novo(&self.atores.contribuinte_numero).unwrap();

        let tipo_contratante = match contratante_doc.tipo() {
            TipoDocumento::Cpf => "PF",
            TipoDocumento::Cnpj => "PJ",
        };
        let tipo_autor = match autor_doc.tipo() {
            TipoDocumento::Cpf => "PF",
            TipoDocumento::Cnpj => "PJ",
        };
        let tipo_contribuinte = match contribuinte_doc.tipo() {
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
             <contratante numero=\"{contratante_numero}\" nome=\"{contratante_nome}\" tipo=\"{tipo_contratante}\" papel=\"contratante\"/>\
             <autorPedidoDados numero=\"{autor_numero}\" nome=\"{autor_nome}\" tipo=\"{tipo_autor}\" papel=\"autor pedido de dados\"/>\
             <contribuinte numero=\"{contribuinte_numero}\" nome=\"{contribuinte_nome}\" tipo=\"{tipo_contribuinte}\" papel=\"contribuinte\"/>\
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
    /// Assina o Termo de Autorização usando o certificado A1 (arquivo PFX/P12).
    /// 
    /// # Parâmetros
    /// - `termo`: O termo de autorização a ser assinado
    /// - `pfx_der`: Conteúdo do arquivo .pfx/.p12 em formato DER (bytes)
    /// - `senha`: Senha do arquivo PFX (será limpa da memória após uso)
    /// 
    /// # Retorna
    /// `XmlAssinado` contendo o XML assinado em base64, digest, signature value e certificado.
    async fn assinar(
        &self,
        termo: &TermoAutorizacao,
        pfx_der: &mut [u8],
        senha: &mut [u8],
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
            atores: AtoresTermoAutorizacao {
                contratante_numero: "11222333000181".to_owned(),
                contratante_nome: "Accurit Tecnologia Ltda".to_owned(),
                autor_pedido_dados_numero: "11222333000181".to_owned(),
                autor_pedido_dados_nome: "Escritório & Associados".to_owned(),
                contribuinte_numero: "52998224725".to_owned(),
                contribuinte_nome: "João da Silva".to_owned(),
            },
        }
    }

    #[test]
    fn gera_termo_xml_sem_perder_textos_fixos_ou_escapar_atributos() {
        let xml = termo().para_xml().expect("XML válido");
        assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(xml.contains("Escritório &amp; Associados"));
        assert!(xml.contains("artigos 7º ou 11º da LGPD"));
        assert!(xml.contains("<contratante"));
        assert!(xml.contains("<autorPedidoDados"));
        assert!(xml.contains("<contribuinte"));
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
        termo.atores.contribuinte_nome.push('\u{1}');
        assert_eq!(termo.para_xml(), Err(ErroAssinatura::CaractereXmlInvalido));
    }

    #[test]
    fn rejeita_contratante_nao_cnpj() {
        let mut termo = termo();
        termo.atores.contratante_numero = "52998224725".to_owned(); // CPF
        assert_eq!(termo.para_xml(), Err(ErroAssinatura::TermoInvalido));
    }

    #[test]
    fn rejeita_autor_pedido_dados_nao_cnpj() {
        let mut termo = termo();
        termo.atores.autor_pedido_dados_numero = "52998224725".to_owned(); // CPF
        assert_eq!(termo.para_xml(), Err(ErroAssinatura::TermoInvalido));
    }

    #[test]
    fn aceita_contribuinte_pf_ou_pj() {
        // PF
        let mut termo = termo();
        termo.atores.contribuinte_numero = "52998224725".to_owned();
        assert!(termo.para_xml().is_ok());

        // PJ
        termo.atores.contribuinte_numero = "12345678000195".to_owned();
        assert!(termo.para_xml().is_ok());
    }
}