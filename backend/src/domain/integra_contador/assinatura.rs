use crate::domain::clientes::{CpfCnpj, TipoDocumento};
use chrono::NaiveDate;

/// Ator do Termo de Autorização exigido pelo SERPRO (layout oficial de
/// AUTENTICAPROCURADOR / ENVIOXMLASSINADO81 — ver
/// `docs/Serpro/Integra-Contador/padroes-tecnicos-assinatura-xml.md`):
///
/// - `destinatario` (`papel="contratante"`): o contratante da API (a Software
///   House/SaaS). `numero` = CNPJ (14 dígitos);
/// - `assinadoPor` (`papel="autor pedido de dados"`): o Procurador (titular do
///   certificado que assina o termo). `numero` = documento (CPF ou CNPJ) do
///   certificado A1 que assina — e deve ser o número do certificado usado.
///
/// Não existe ator `contribuinte` no Termo — o contribuinte vive apenas no
/// envelope JSON (`contratante`/`autorPedidoDados`/`contribuinte`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtoresTermoAutorizacao {
    pub destinatario_numero: String,
    pub destinatario_nome: String,
    pub assinado_por_numero: String,
    pub assinado_por_nome: String,
}

impl AtoresTermoAutorizacao {
    /// Valida o par conforme o layout oficial: destinatário obrigatoriamente
    /// CNPJ; assinadoPor pode ser CPF ou CNPJ; nomes obrigatórios.
    pub fn validar(&self) -> Result<(), ErroAssinatura> {
        let destinatario =
            CpfCnpj::novo(&self.destinatario_numero).map_err(|_| ErroAssinatura::TermoInvalido)?;
        CpfCnpj::novo(&self.assinado_por_numero).map_err(|_| ErroAssinatura::TermoInvalido)?;

        if destinatario.tipo() != TipoDocumento::Cnpj {
            return Err(ErroAssinatura::TermoInvalido);
        }
        if self.destinatario_nome.trim().is_empty() || self.assinado_por_nome.trim().is_empty() {
            return Err(ErroAssinatura::TermoInvalido);
        }
        Ok(())
    }

    /// Data corrente em Brasília (UTC-3 fixo — BRT sem horário de verão desde 2019).
    fn data_brasilia() -> NaiveDate {
        (chrono::Utc::now() - chrono::Duration::hours(3)).date_naive()
    }
}

/// Termo de Autorização completo. Os textos de `sistema`, `termo`,
/// `avisoLegal` e `finalidade` são fixos (dados de domínio oficiais do
/// AUTENTICAPROCURADOR).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TermoAutorizacao {
    pub data_assinatura: NaiveDate,
    pub vigencia: NaiveDate,
    pub atores: AtoresTermoAutorizacao,
}

impl TermoAutorizacao {
    /// Constrói o termo com as datas oficiais: assinatura hoje (Brasília) e
    /// vigência 365 dias após; falha em `ErroAssinatura::TermoInvalido` se
    /// os atores não passarem no layout oficial.
    pub fn novo_oficial(atores: AtoresTermoAutorizacao) -> Result<Self, ErroAssinatura> {
        let hoje = AtoresTermoAutorizacao::data_brasilia();
        let vigencia = hoje + chrono::Duration::days(365);
        let termo = Self {
            data_assinatura: hoje,
            vigencia,
            atores,
        };
        termo.validar()?;
        Ok(termo)
    }

    /// Valida o termo completo: vigência >= assinatura (exigido pelo SERPRO,
    /// `EntradaIncorreta-AUTENTICAPROCURADOR-017`) e atores válidos.
    pub fn validar(&self) -> Result<(), ErroAssinatura> {
        if self.vigencia < self.data_assinatura {
            return Err(ErroAssinatura::TermoInvalido);
        }
        self.atores.validar()
    }

    /// Gera o XML oficial (compacto, atributos escapados).
    pub fn para_xml(&self) -> Result<String, ErroAssinatura> {
        self.validar()?;

        let destinatario_numero = escapar_atributo(&self.atores.destinatario_numero)?;
        let destinatario_nome = escapar_atributo(&self.atores.destinatario_nome)?;
        let assinado_por_numero = escapar_atributo(&self.atores.assinado_por_numero)?;
        let assinado_por_nome = escapar_atributo(&self.atores.assinado_por_nome)?;

        let data_assinatura = self.data_assinatura.format("%Y%m%d");
        let vigencia = self.vigencia.format("%Y%m%d");

        Ok(format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?><termoDeAutorizacao><dados>\
             <sistema id=\"API Integra Contador\"/><termo texto=\"{}\"/>\
             <avisoLegal texto=\"{}\"/><finalidade texto=\"{}\"/>\
             <dataAssinatura data=\"{data_assinatura}\"/><vigencia data=\"{vigencia}\"/>\
             <destinatario numero=\"{destinatario_numero}\" nome=\"{destinatario_nome}\" tipo=\"PJ\" papel=\"contratante\"/>\
             <assinadoPor numero=\"{assinado_por_numero}\" nome=\"{assinado_por_nome}\" tipo=\"{}\" papel=\"autor pedido de dados\"/>\
             </dados></termoDeAutorizacao>",
            escapar_atributo(TERMO)?,
            escapar_atributo(AVISO_LEGAL)?,
            escapar_atributo(FINALIDADE)?,
            tipo_do_documento(&self.atores.assinado_por_numero),
        ))
    }
}

fn tipo_do_documento(numero: &str) -> &'static str {
    match CpfCnpj::novo(numero) {
        Ok(documento) => match documento.tipo() {
            TipoDocumento::Cpf => "PF",
            TipoDocumento::Cnpj => "PJ",
        },
        Err(_) => "PF",
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

/// Assina o Termo de Autorização (XML) e devolve o documento pronto para o
/// `pedidoDados.dados` do `/Apoiar`.
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
            '\u{9}' => escapado.push_str("&#x9;"),
            '\u{A}' => escapado.push_str("&#xA;"),
            '\u{D}' => escapado.push_str("&#xD;"),
            _ => escapado.push(caractere),
        }
    }
    Ok(escapado)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn termo() -> TermoAutorizacao {
        let atores = AtoresTermoAutorizacao {
            destinatario_numero: "11222333000181".to_owned(),
            destinatario_nome: "Accurit Tecnologia Ltda".to_owned(),
            assinado_por_numero: "11122233000183".to_owned(),
            assinado_por_nome: "Escritório & Associados".to_owned(),
        };
        TermoAutorizacao {
            data_assinatura: NaiveDate::from_ymd_opt(2026, 11, 6).expect("data válida"),
            vigencia: NaiveDate::from_ymd_opt(2027, 11, 6).expect("data válida"),
            atores,
        }
    }

    #[test]
    fn termo_oficial_usa_layout_serpro_sem_contribuinte() {
        let xml = termo().para_xml().expect("XML válido");
        assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(xml.contains("<destinatario"));
        assert!(xml.contains("<assinadoPor"));
        assert!(!xml.contains("<contribuinte"));
        assert!(!xml.contains("<contratante"));
        assert!(!xml.contains("<autorPedidoDados"));
        assert!(xml.contains("<sistema id=\"API Integra Contador\"/>"));
        assert!(xml.contains("papel=\"contratante\""));
        assert!(xml.contains("papel=\"autor pedido de dados\""));
        assert!(xml.contains("tipo=\"PJ\" papel=\"contratante\""));
        assert!(xml.contains("artigos 7º ou 11º da LGPD"));
        assert!(xml.contains("Escritório &amp; Associados"));
        assert!(!xml.contains('\n'));
    }

    #[test]
    fn rejeita_destinatario_nao_cnpj() {
        let mut termo = termo();
        termo.atores.destinatario_numero = "52998224725".to_owned(); // CPF
        assert_eq!(termo.para_xml(), Err(ErroAssinatura::TermoInvalido));
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
        termo.atores.assinado_por_nome.push('\u{1}');
        assert_eq!(termo.para_xml(), Err(ErroAssinatura::CaractereXmlInvalido));
    }

    #[test]
    fn aceita_assinado_por_cpf_ou_cnpj() {
        // CPF
        let mut termo = termo();
        termo.atores.assinado_por_numero = "52998224725".to_owned();
        assert!(termo.para_xml().is_ok());

        // CNPJ
        termo.atores.assinado_por_numero = "11122233000183".to_owned();
        assert!(termo.para_xml().is_ok());
    }

    #[test]
    fn novo_oficial_gera_datas_de_brasilia_em_aaaaaaaammdd() {
        let atores = AtoresTermoAutorizacao {
            destinatario_numero: "11222333000181".to_owned(),
            destinatario_nome: "Accurit Tecnologia Ltda".to_owned(),
            assinado_por_numero: "52998224725".to_owned(),
            assinado_por_nome: "João da Silva".to_owned(),
        };
        let termo = TermoAutorizacao::novo_oficial(atores).expect("termo válido");
        assert!(termo.data_assinatura <= chrono::Utc::now().date_naive());
        assert!(termo.vigencia >= termo.data_assinatura);
    }
}
