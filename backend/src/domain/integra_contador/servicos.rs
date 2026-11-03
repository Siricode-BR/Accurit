//! Catálogo de serviços de referência do Integra Contador.
//! Validação de idSistema, idServico, versaoSistema e payload.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Serviço de referência do Integra Contador.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServicoReferencia {
    pub id_sistema: String,
    pub id_servico: String,
    pub versao_sistema: String,
    pub descricao: String,
    pub exige_procuracao: bool,
    pub verbo_padrao: String,
    pub endpoint_path: String,
    pub bilhetado: bool,
}

/// Catálogo de serviços suportados.
pub fn catalogo_servicos() -> HashMap<&'static str, ServicoReferencia> {
    let mut mapa = HashMap::new();

    // SICALC
    mapa.insert("SICALC/CONSULTAAPOIORECEITAS52", ServicoReferencia {
        id_sistema: "SICALC".to_owned(),
        id_servico: "CONSULTAAPOIORECEITAS52".to_owned(),
        versao_sistema: "1".to_owned(),
        descricao: "Consulta Apoio Receitas".to_owned(),
        exige_procuracao: false,
        verbo_padrao: "CONSULTAR".to_owned(),
        endpoint_path: "/SICALC/CONSULTAAPOIORECEITAS52".to_owned(),
        bilhetado: true,
    });

    // PROCURACOES
    mapa.insert("PROCURACOES/OBTERPROCURACAO41", ServicoReferencia {
        id_sistema: "PROCURACOES".to_owned(),
        id_servico: "OBTERPROCURACAO41".to_owned(),
        versao_sistema: "1".to_owned(),
        descricao: "Obter Procuração".to_owned(),
        exige_procuracao: false,
        verbo_padrao: "CONSULTAR".to_owned(),
        endpoint_path: "/PROCURACOES/OBTERPROCURACAO41".to_owned(),
        bilhetado: true,
    });

    // SITFIS
    mapa.insert("SITFIS/RELATORIOSITFIS92", ServicoReferencia {
        id_sistema: "SITFIS".to_owned(),
        id_servico: "RELATORIOSITFIS92".to_owned(),
        versao_sistema: "1".to_owned(),
        descricao: "Relatório SITFIS".to_owned(),
        exige_procuracao: true,
        verbo_padrao: "EMITIR".to_owned(),
        endpoint_path: "/SITFIS/RELATORIOSITFIS92".to_owned(),
        bilhetado: true,
    });
    mapa.insert("SITFIS/SOLICITARPROTOCOLO91", ServicoReferencia {
        id_sistema: "SITFIS".to_owned(),
        id_servico: "SOLICITARPROTOCOLO91".to_owned(),
        versao_sistema: "1".to_owned(),
        descricao: "Solicitar Protocolo SITFIS".to_owned(),
        exige_procuracao: true,
        verbo_padrao: "DECLARAR".to_owned(),
        endpoint_path: "/SITFIS/SOLICITARPROTOCOLO91".to_owned(),
        bilhetado: true,
    });

    // PGDASD
    mapa.insert("PGDASD/CONSDECLARACAO13", ServicoReferencia {
        id_sistema: "PGDASD".to_owned(),
        id_servico: "CONSDECLARACAO13".to_owned(),
        versao_sistema: "1".to_owned(),
        descricao: "Consultar Declaração PGDASD".to_owned(),
        exige_procuracao: true,
        verbo_padrao: "CONSULTAR".to_owned(),
        endpoint_path: "/PGDASD/CONSDECLARACAO13".to_owned(),
        bilhetado: true,
    });
    mapa.insert("PGDASD/TRANSDECLARACAO11", ServicoReferencia {
        id_sistema: "PGDASD".to_owned(),
        id_servico: "TRANSDECLARACAO11".to_owned(),
        versao_sistema: "1".to_owned(),
        descricao: "Transmitir Declaração PGDASD".to_owned(),
        exige_procuracao: true,
        verbo_padrao: "DECLARAR".to_owned(),
        endpoint_path: "/PGDASD/TRANSDECLARACAO11".to_owned(),
        bilhetado: true,
    });

    // DCTFWEB
    mapa.insert("DCTFWEB/GERARGUIA31", ServicoReferencia {
        id_sistema: "DCTFWEB".to_owned(),
        id_servico: "GERARGUIA31".to_owned(),
        versao_sistema: "1".to_owned(),
        descricao: "Gerar Guia DCTFWeb".to_owned(),
        exige_procuracao: true,
        verbo_padrao: "EMITIR".to_owned(),
        endpoint_path: "/DCTFWEB/GERARGUIA31".to_owned(),
        bilhetado: true,
    });
    mapa.insert("DCTFWEB/TRANSDECLARACAO310", ServicoReferencia {
        id_sistema: "DCTFWEB".to_owned(),
        id_servico: "TRANSDECLARACAO310".to_owned(),
        versao_sistema: "1".to_owned(),
        descricao: "Transmitir Declaração DCTFWeb".to_owned(),
        exige_procuracao: true,
        verbo_padrao: "DECLARAR".to_owned(),
        endpoint_path: "/DCTFWEB/TRANSDECLARACAO310".to_owned(),
        bilhetado: true,
    });

    // CAIXAPOSTAL
    mapa.insert("CAIXAPOSTAL/MSGCONTRIBUINTE61", ServicoReferencia {
        id_sistema: "CAIXAPOSTAL".to_owned(),
        id_servico: "MSGCONTRIBUINTE61".to_owned(),
        versao_sistema: "1".to_owned(),
        descricao: "Mensagem Contribuinte".to_owned(),
        exige_procuracao: true,
        verbo_padrao: "CONSULTAR".to_owned(),
        endpoint_path: "/CAIXAPOSTAL/MSGCONTRIBUINTE61".to_owned(),
        bilhetado: true,
    });
    mapa.insert("CAIXAPOSTAL/MSGDETALHAMENTO62", ServicoReferencia {
        id_sistema: "CAIXAPOSTAL".to_owned(),
        id_servico: "MSGDETALHAMENTO62".to_owned(),
        versao_sistema: "1".to_owned(),
        descricao: "Detalhamento Mensagem".to_owned(),
        exige_procuracao: true,
        verbo_padrao: "CONSULTAR".to_owned(),
        endpoint_path: "/CAIXAPOSTAL/MSGDETALHAMENTO62".to_owned(),
        bilhetado: true,
    });
    mapa.insert("CAIXAPOSTAL/INNOVAMSG63", ServicoReferencia {
        id_sistema: "CAIXAPOSTAL".to_owned(),
        id_servico: "INNOVAMSG63".to_owned(),
        versao_sistema: "1".to_owned(),
        descricao: "Inova Mensagem".to_owned(),
        exige_procuracao: true,
        verbo_padrao: "CONSULTAR".to_owned(),
        endpoint_path: "/CAIXAPOSTAL/INNOVAMSG63".to_owned(),
        bilhetado: true,
    });

    // AUTENTICAPROCURADOR
    mapa.insert("AUTENTICAPROCURADOR/ENVIOXMLASSINADO81", ServicoReferencia {
        id_sistema: "AUTENTICAPROCURADOR".to_owned(),
        id_servico: "ENVIOXMLASSINADO81".to_owned(),
        versao_sistema: "1.0".to_owned(),
        descricao: "Envio XML Assinado - Termo de Autorização".to_owned(),
        exige_procuracao: false,
        verbo_padrao: "DECLARAR".to_owned(),
        endpoint_path: "/AUTENTICAPROCURADOR/ENVIOXMLASSINADO81".to_owned(),
        bilhetado: false, // Não bilhetado conforme SPEC
    });

    mapa
}

/// Chave composta para busca no catálogo.
pub fn chave_servico(id_sistema: &str, id_servico: &str) -> String {
    format!("{}/{}", id_sistema.to_uppercase(), id_servico.to_uppercase())
}

/// Valida se o serviço existe no catálogo e retorna a referência.
pub fn validar_servico(id_sistema: &str, id_servico: &str, versao_sistema: &str) -> Result<&'static ServicoReferencia, &'static str> {
    let chave = chave_servico(id_sistema, id_servico);
    let catalogo = catalogo_servicos();
    
    let servico = catalogo.get(chave.as_str())
        .ok_or("Serviço não encontrado no catálogo oficial")?;
    
    if servico.versao_sistema != versao_sistema {
        return Err("Versão de sistema incompatível com o catálogo");
    }
    
    Ok(servico)
}

/// Obtém o endpoint path para um serviço.
pub fn endpoint_para_servico(id_sistema: &str, id_servico: &str) -> Option<String> {
    let chave = chave_servico(id_sistema, id_servico);
    catalogo_servicos().get(chave.as_str()).map(|s| s.endpoint_path.clone())
}

/// Verifica se o serviço é bilhetado.
pub fn servico_e_bilhetado(id_sistema: &str, id_servico: &str) -> bool {
    let chave = chave_servico(id_sistema, id_servico);
    catalogo_servicos().get(chave.as_str()).map_or(false, |s| s.bilhetado)
}

/// Verifica se o serviço exige procuração.
pub fn servico_exige_procuracao(id_sistema: &str, id_servico: &str) -> bool {
    let chave = chave_servico(id_sistema, id_servico);
    catalogo_servicos().get(chave.as_str()).map_or(false, |s| s.exige_procuracao)
}

/// Verifica se o verbo é compatível com o serviço.
pub fn verbo_compativel(id_sistema: &str, id_servico: &str, verbo: &str) -> bool {
    let chave = chave_servico(id_sistema, id_servico);
    catalogo_servicos().get(chave.as_str()).map_or(false, |s| {
        s.verbo_padrao.eq_ignore_ascii_case(verbo)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalogo_tem_todos_servicos_referencia() {
        let catalogo = catalogo_servicos();
        assert!(catalogo.contains_key("SICALC/CONSULTAAPOIORECEITAS52"));
        assert!(catalogo.contains_key("PROCURACOES/OBTERPROCURACAO41"));
        assert!(catalogo.contains_key("SITFIS/RELATORIOSITFIS92"));
        assert!(catalogo.contains_key("PGDASD/CONSDECLARACAO13"));
        assert!(catalogo.contains_key("PGDASD/TRANSDECLARACAO11"));
        assert!(catalogo.contains_key("DCTFWEB/GERARGUIA31"));
        assert!(catalogo.contains_key("CAIXAPOSTAL/INNOVAMSG63"));
        assert!(catalogo.contains_key("AUTENTICAPROCURADOR/ENVIOXMLASSINADO81"));
    }

    #[test]
    fn validar_servico_existente() {
        let resultado = validar_servico("SICALC", "CONSULTAAPOIORECEITAS52", "1");
        assert!(resultado.is_ok());
        assert_eq!(resultado.unwrap().id_sistema, "SICALC");
    }

    #[test]
    fn validar_servico_versao_invalida() {
        let resultado = validar_servico("SICALC", "CONSULTAAPOIORECEITAS52", "2");
        assert!(resultado.is_err());
    }

    #[test]
    fn validar_servico_inexistente() {
        let resultado = validar_servico("INEXISTENTE", "SERVICO", "1");
        assert!(resultado.is_err());
    }

    #[test]
    fn servico_bilhetado() {
        assert!(servico_e_bilhetado("SICALC", "CONSULTAAPOIORECEITAS52"));
        assert!(servico_e_bilhetado("PGDASD", "TRANSDECLARACAO11"));
        assert!(!servico_e_bilhetado("AUTENTICAPROCURADOR", "ENVIOXMLASSINADO81"));
    }

    #[test]
    fn servico_exige_procuracao() {
        assert!(!servico_exige_procuracao("SICALC", "CONSULTAAPOIORECEITAS52"));
        assert!(!servico_exige_procuracao("PROCURACOES", "OBTERPROCURACAO41"));
        assert!(servico_exige_procuracao("PGDASD", "TRANSDECLARACAO11"));
        assert!(servico_exige_procuracao("DCTFWEB", "GERARGUIA31"));
    }
}