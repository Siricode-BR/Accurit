//! DTOs de interpretar o JSON do SERPRO (SPEC §4.3): a aplicação
//! traduz o contrato da API em valores saneados do domínio — o
//! parse usa `serde(default)` para tolerar campos ausentes; a
//! validação de negócio fica nos construtores do domínio.
//!
//! Dois formatos coexistem na amostra oficial: a consulta de
//! devedor usa chaves `snake_case`, e o detalhe de inscrição usa
//! `camelCase` (com siglas preservadas — `SV8`, `PFN` — via
//! `rename` explícito).

use serde::Deserialize;

/// Corpo da consulta de devedor: lista de inscrições, item único
/// ou array vazio (`#[serde(untagged)]`, SPEC §4.3).
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum DevedorResponse {
    Lista(Vec<DevedorPayload>),
    Item(Box<DevedorPayload>),
}

/// Campos da inscrição no consultório de devedor (JSON
/// `snake_case`); todos opcionais — dados defeituosos só existem
/// no domínio.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct DevedorPayload {
    pub numero_inscricao: Option<String>,
    pub numero_processo: Option<String>,
    pub situacao_inscricao: Option<String>,
    pub situacao_descricao: Option<String>,
    pub nome_devedor: Option<String>,
    pub tipo_devedor: Option<String>,
    pub valor_total_consolidado_moeda: Option<String>,
    pub cpf_cnpj: Option<String>,
    pub codigo_sida: Option<String>,
    pub nome_unidade: Option<String>,
    pub codigo_comprot: Option<String>,
    pub codigo_uorg: Option<String>,
}

impl DevedorPayload {
    /// "Objeto plano" nulo = corpo vazio (SPEC §4.4 passo 7).
    pub fn e_vazio(&self) -> bool {
        self.numero_inscricao.is_none()
            && self.numero_processo.is_none()
            && self.situacao_inscricao.is_none()
            && self.situacao_descricao.is_none()
            && self.nome_devedor.is_none()
            && self.tipo_devedor.is_none()
            && self.valor_total_consolidado_moeda.is_none()
            && self.cpf_cnpj.is_none()
            && self.codigo_sida.is_none()
            && self.nome_unidade.is_none()
            && self.codigo_comprot.is_none()
            && self.codigo_uorg.is_none()
    }
}

/// Resposta da consulta por inscrição: a amostra oficial retorna
/// um objeto; arrays podem aparecer em versões — um elemento único é
/// aceito, mais de um é contrato inválido.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum InscricaoResponse {
    Lista(Vec<InscricaoDetalhePayload>),
    Item(Box<InscricaoDetalhePayload>),
}

/// Detalhe da inscrição (JSON `camelCase`, SPEC §4.3).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct InscricaoDetalhePayload {
    pub inscricao: Option<String>,
    pub processo_administrativo: Option<String>,
    pub situacao: Option<String>,
    pub data_inscricao: Option<String>,
    #[serde(rename = "numeroPFNResponsavel")]
    pub numero_pfn_responsavel: Option<String>,
    #[serde(rename = "nomePFNResponsavel")]
    pub nome_pfn_responsavel: Option<String>,
    #[serde(rename = "numeroPFNInscricao")]
    pub numero_pfn_inscricao: Option<String>,
    #[serde(rename = "nomePFNInscricao")]
    pub nome_pfn_inscricao: Option<String>,
    pub codigo_receita_principal: Option<String>,
    pub nome_receita: Option<String>,
    #[serde(rename = "indicadorPrescricaoSV8")]
    pub indicador_prescricao_sv8: Option<String>,
    pub valor_total_inscrito_moeda: Option<String>,
    pub valor_total_consolidado_moeda: Option<String>,
    pub valor_remanescente_moeda: Option<String>,
    pub nome_devedor: Option<String>,
    pub tipo_devedor: Option<String>,
    pub nome_unidade: Option<String>,
    pub codigo_sistema_origem: Option<String>,
    pub descricao_sistema_origem: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn devedor_lista_e_itens_parseiam() {
        let corpo = r#"[
            { "numero_inscricao": "10 5 03 003880-14",
              "valor_total_consolidado_moeda": "4.457,56" }
        ]"#;
        let resposta: DevedorResponse = serde_json::from_str(corpo).expect("lista parse");
        let DevedorResponse::Lista(itens) = resposta else {
            panic!("era lista");
        };
        assert_eq!(itens.len(), 1);
        assert_eq!(
            itens[0].valor_total_consolidado_moeda.as_deref(),
            Some("4.457,56")
        );

        let item: DevedorResponse =
            serde_json::from_str(r#"{"codigo_sida": "1000"}"#).expect("item parse");
        let DevedorResponse::Item(payload) = item else {
            panic!("era item");
        };
        assert_eq!(payload.codigo_sida.as_deref(), Some("1000"));
    }

    #[test]
    fn devedor_aceita_objeto_plano_nulo_e_campos_faltantes() {
        let resposta: DevedorResponse = serde_json::from_str("{}").expect("objeto plano");
        let DevedorResponse::Item(payload) = resposta else {
            panic!("era item");
        };
        assert!(payload.e_vazio());

        let vazio: DevedorResponse = serde_json::from_str("[]").expect("array vazio");
        let DevedorResponse::Lista(itens) = vazio else {
            panic!("era lista");
        };
        assert!(itens.is_empty());
    }

    #[test]
    fn detalhe_preserva_siglas_e_camel_case() {
        let corpo = r#"{
            "inscricao": "90 6 17 000964-19",
            "processoAdministrativo": "46206 099222/99-01",
            "numeroPFNResponsavel": "1000",
            "nomePFNResponsavel": "PRIMEIRA REGIAO",
            "indicadorPrescricaoSV8": "NAO",
            "valorTotalInscritoMoeda": "805,06",
            "codigoSistemaOrigem": "M",
            "descricaoSistemaOrigem": "Manual"
        }"#;
        let detalhe: InscricaoDetalhePayload = serde_json::from_str(corpo).expect("detalhe parse");

        assert_eq!(detalhe.numero_pfn_responsavel.as_deref(), Some("1000"));
        assert_eq!(
            detalhe.nome_pfn_responsavel.as_deref(),
            Some("PRIMEIRA REGIAO")
        );
        assert_eq!(detalhe.indicador_prescricao_sv8.as_deref(), Some("NAO"));
        assert_eq!(detalhe.codigo_sistema_origem.as_deref(), Some("M"));
        assert_eq!(
            detalhe.processo_administrativo.as_deref(),
            Some("46206 099222/99-01")
        );
    }

    #[test]
    fn detalhe_objeto_plano_e_aceito() {
        let detalhe: InscricaoDetalhePayload = serde_json::from_str("{}").expect("detalhe plano");
        assert!(detalhe.inscricao.is_none());
    }

    #[test]
    fn corpo_fora_de_contrato_e_aceito() {
        let erro = serde_json::from_str::<DevedorResponse>("[1]");
        assert!(erro.is_err());
    }
}
