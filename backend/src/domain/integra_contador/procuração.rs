//! Regras de procuração e-CAC para o Integra Contador.
//! Baseado no catálogo oficial e ADR-0007.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Regra de procuração para um serviço.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "tipo", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RegraProcuracao {
    /// Serviço não exige procuração.
    NaoSeAplica,
    /// Qualquer um dos códigos listados satisfaz a autorização.
    AnyOf { codigos: Vec<String> },
    /// Exige avaliação de contexto (ex.: tipo de contribuinte).
    Condicional { codigos: Vec<String>, contexto: String },
}

impl RegraProcuracao {
    /// Verifica se a procuração se aplica ao serviço.
    pub fn se_aplica(&self) -> bool {
        !matches!(self, Self::NaoSeAplica)
    }

    /// Verifica se um código de serviço satisfaz a regra.
    pub fn codigo_satisfaz(&self, codigo: &str) -> bool {
        match self {
            Self::NaoSeAplica => false,
            Self::AnyOf { codigos } => codigos.iter().any(|c| c == codigo),
            Self::Condicional { codigos, .. } => codigos.iter().any(|c| c == codigo),
        }
    }

    /// Obtém os códigos válidos para a regra.
    pub fn codigos_validos(&self) -> Vec<String> {
        match self {
            Self::NaoSeAplica => vec![],
            Self::AnyOf { codigos } | Self::Condicional { codigos, .. } => codigos.clone(),
        }
    }
}

/// Códigos de serviço que exigem procuração (catálogo oficial).
/// Fonte: docs/Serpro/Integra-Contador/catalogo-procuracoes.md
pub fn catalogo_procuracoes() -> HashMap<&'static str, RegraProcuracao> {
    let mut mapa = HashMap::new();

    // SICALC
    mapa.insert("CONSULTAAPOIORECEITAS52", RegraProcuracao::NaoSeAplica);

    // PROCURACOES
    mapa.insert("OBTERPROCURACAO41", RegraProcuracao::NaoSeAplica);

    // SITFIS
    mapa.insert("RELATORIOSITFIS92", RegraProcuracao::AnyOf {
        codigos: vec!["00002".to_owned()],
    });
    mapa.insert("SOLICITARPROTOCOLO91", RegraProcuracao::AnyOf {
        codigos: vec!["00002".to_owned()],
    });

    // PGDASD (inclui DEFIS)
    mapa.insert("CONSDECLARACAO13", RegraProcuracao::AnyOf {
        codigos: vec!["00146".to_owned()],
    });
    mapa.insert("TRANSDECLARACAO11", RegraProcuracao::AnyOf {
        codigos: vec!["00146".to_owned()],
    });

    // DCTFWeb
    mapa.insert("GERARGUIA31", RegraProcuracao::AnyOf {
        codigos: vec!["00103".to_owned()],
    });
    mapa.insert("TRANSDECLARACAO310", RegraProcuracao::AnyOf {
        codigos: vec!["00103".to_owned()],
    });

    // Caixa Postal
    mapa.insert("MSGCONTRIBUINTE61", RegraProcuracao::AnyOf {
        codigos: vec!["00006".to_owned()],
    });
    mapa.insert("MSGDETALHAMENTO62", RegraProcuracao::AnyOf {
        codigos: vec!["00006".to_owned()],
    });
    mapa.insert("INNOVAMSG63", RegraProcuracao::AnyOf {
        codigos: vec!["00006".to_owned()],
    });

    // DTE
    mapa.insert("CONSULTASITUACAODTE111", RegraProcuracao::AnyOf {
        codigos: vec!["00050".to_owned()],
    });

    // Pagamentos (PAGTOWEB)
    mapa.insert("PAGAMENTOS71", RegraProcuracao::AnyOf {
        codigos: vec!["00004".to_owned()],
    });
    mapa.insert("COMPARRECADACAO72", RegraProcuracao::AnyOf {
        codigos: vec!["00004".to_owned()],
    });
    mapa.insert("CONTACONSDOCARRPG73", RegraProcuracao::AnyOf {
        codigos: vec!["00004".to_owned()],
    });

    // Regime de apuração
    mapa.insert("EFETUAROPCAOREGIME101", RegraProcuracao::AnyOf {
        codigos: vec!["00060".to_owned()],
    });

    // MEI (DASNSIMEI)
    mapa.insert("CONSULTIMADECREC152", RegraProcuracao::AnyOf {
        codigos: vec!["00229".to_owned()],
    });
    mapa.insert("GERARDASEXCESSO153", RegraProcuracao::AnyOf {
        codigos: vec!["00229".to_owned()],
    });

    // e-Processo (EPROCESSO)
    mapa.insert("CONSPROCPORINTER271", RegraProcuracao::AnyOf {
        codigos: vec!["00051".to_owned()],
    });

    // Parcelamentos (PARCSN) - BLOQUEADOS pela allowlist (D4)
    mapa.insert("PARCELAMENTO_SN", RegraProcuracao::AnyOf {
        codigos: vec!["00076".to_owned(), "00188".to_owned()],
    });

    // Parcelamento Especial (PARCSN-ESP) - BLOQUEADO pela allowlist (D4)
    mapa.insert("PARCELAMENTO_ESP", RegraProcuracao::AnyOf {
        codigos: vec!["00125".to_owned()],
    });

    // EVENTOSATUALIZACAO
    mapa.insert("EVENTOSATUALIZACAO", RegraProcuracao::Condicional {
        codigos: vec!["00103".to_owned()],
        contexto: "validar caso a caso".to_owned(),
    });

    // AUTENTICAPROCURADOR
    mapa.insert("ENVIOXMLASSINADO81", RegraProcuracao::NaoSeAplica);

    mapa
}

/// Obtém a regra de procuração para um serviço.
pub fn regra_procuracao_para_servico(id_servico: &str) -> Option<RegraProcuracao> {
    catalogo_procuracoes().get(id_servico).cloned()
}

/// Validação de procuração para um serviço.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidacaoProcuracao {
    pub contribuinte_cpf_cnpj: String,
    pub procurador_cpf_cnpj: String,
    pub codigo_servico: String,
    pub status: String,
    pub valido_de: Option<chrono::DateTime<chrono::Utc>>,
    pub valido_ate: Option<chrono::DateTime<chrono::Utc>>,
    pub verificado_em: chrono::DateTime<chrono::Utc>,
    pub ambiente: String,
}

impl ValidacaoProcuracao {
    /// Verifica se a procuração é válida para o serviço no momento atual.
    pub fn e_valida_agora(&self, agora: chrono::DateTime<chrono::Utc>) -> bool {
        if self.status != "VALIDA" && self.status != "ATIVA" {
            return false;
        }
        if let Some(valido_de) = self.valido_de {
            if agora < valido_de {
                return false;
            }
        }
        if let Some(valido_ate) = self.valido_ate {
            if agora > valido_ate {
                return false;
            }
        }
        true
    }

    /// Verifica se a procuração cobre o código de serviço exigido.
    pub fn cobre_servico(&self, regra: &RegraProcuracao) -> bool {
        regra.codigo_satisfaz(&self.codigo_servico)
    }
}

/// Códigos de serviço de referência do Integra Contador.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CodigoServicoProcuracao {
    // SICALC
    ConsultaApoioReceitas52,
    // PROCURACOES
    ObterProcuracao41,
    // SITFIS
    RelatorioSitfis92,
    SolicitarProtocolo91,
    // PGDASD
    ConsDeclaracao13,
    TransDeclaracao11,
    // DCTFWEB
    GerarGuia31,
    TransDeclaracao310,
    // CAIXAPOSTAL
    MsgContribuinte61,
    MsgDetalhamento62,
    Innovamsg63,
    // DTE
    ConsultaSituacaoDte111,
    // PAGTOWEB
    Pagamentos71,
    ComparRecadacao72,
    ContaConsDocArRpg73,
    // REGIMEAPURACAO
    EfetuarOpcaoRegime101,
    // DASNSIMEI
    ConsultImadecrec152,
    GerarDasExcesso153,
    // EPROCESSO
    ConsProcPorInter271,
    // PARCSN (bloqueados)
    ParcelamentoSn,
    ParcelamentoEsp,
    // EVENTOSATUALIZACAO
    EventosAtualizacao,
    // AUTENTICAPROCURADOR
    EnvioXmlAssinado81,
}

impl CodigoServicoProcuracao {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ConsultaApoioReceitas52 => "CONSULTAAPOIORECEITAS52",
            Self::ObterProcuracao41 => "OBTERPROCURACAO41",
            Self::RelatorioSitfis92 => "RELATORIOSITFIS92",
            Self::SolicitarProtocolo91 => "SOLICITARPROTOCOLO91",
            Self::ConsDeclaracao13 => "CONSDECLARACAO13",
            Self::TransDeclaracao11 => "TRANSDECLARACAO11",
            Self::GerarGuia31 => "GERARGUIA31",
            Self::TransDeclaracao310 => "TRANSDECLARACAO310",
            Self::MsgContribuinte61 => "MSGCONTRIBUINTE61",
            Self::MsgDetalhamento62 => "MSGDETALHAMENTO62",
            Self::Innovamsg63 => "INNOVAMSG63",
            Self::ConsultaSituacaoDte111 => "CONSULTASITUACAODTE111",
            Self::Pagamentos71 => "PAGAMENTOS71",
            Self::ComparRecadacao72 => "COMPARRECADACAO72",
            Self::ContaConsDocArRpg73 => "CONTACONSDOCARRPG73",
            Self::EfetuarOpcaoRegime101 => "EFETUAROPCAOREGIME101",
            Self::ConsultImadecrec152 => "CONSULTIMADECREC152",
            Self::GerarDasExcesso153 => "GERARDASEXCESSO153",
            Self::ConsProcPorInter271 => "CONSPROCPORINTER271",
            Self::ParcelamentoSn => "PARCELAMENTO_SN",
            Self::ParcelamentoEsp => "PARCELAMENTO_ESP",
            Self::EventosAtualizacao => "EVENTOSATUALIZACAO",
            Self::EnvioXmlAssinado81 => "ENVIOXMLASSINADO81",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "CONSULTAAPOIORECEITAS52" => Some(Self::ConsultaApoioReceitas52),
            "OBTERPROCURACAO41" => Some(Self::ObterProcuracao41),
            "RELATORIOSITFIS92" => Some(Self::RelatorioSitfis92),
            "SOLICITARPROTOCOLO91" => Some(Self::SolicitarProtocolo91),
            "CONSDECLARACAO13" => Some(Self::ConsDeclaracao13),
            "TRANSDECLARACAO11" => Some(Self::TransDeclaracao11),
            "GERARGUIA31" => Some(Self::GerarGuia31),
            "TRANSDECLARACAO310" => Some(Self::TransDeclaracao310),
            "MSGCONTRIBUINTE61" => Some(Self::MsgContribuinte61),
            "MSGDETALHAMENTO62" => Some(Self::MsgDetalhamento62),
            "INNOVAMSG63" => Some(Self::Innovamsg63),
            "CONSULTASITUACAODTE111" => Some(Self::ConsultaSituacaoDte111),
            "PAGAMENTOS71" => Some(Self::Pagamentos71),
            "COMPARRECADACAO72" => Some(Self::ComparRecadacao72),
            "CONTACONSDOCARRPG73" => Some(Self::ContaConsDocArRpg73),
            "EFETUAROPCAOREGIME101" => Some(Self::EfetuarOpcaoRegime101),
            "CONSULTIMADECREC152" => Some(Self::ConsultImadecrec152),
            "GERARDASEXCESSO153" => Some(Self::GerarDasExcesso153),
            "CONSPROCPORINTER271" => Some(Self::ConsProcPorInter271),
            "PARCELAMENTO_SN" => Some(Self::ParcelamentoSn),
            "PARCELAMENTO_ESP" => Some(Self::ParcelamentoEsp),
            "EVENTOSATUALIZACAO" => Some(Self::EventosAtualizacao),
            "ENVIOXMLASSINADO81" => Some(Self::EnvioXmlAssinado81),
            _ => None,
        }
    }

    /// Verifica se o serviço está bloqueado pela allowlist (D4).
    pub fn esta_bloqueado_allowlist(&self) -> bool {
        matches!(
            self,
            Self::ParcelamentoSn | Self::ParcelamentoEsp
        )
    }

    /// Verifica se o serviço exige procuração.
    pub fn exige_procuracao(&self) -> bool {
        let regra = regra_procuracao_para_servico(self.as_str());
        regra.map_or(false, |r| r.se_aplica())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalogo_tem_servicos_referencia() {
        let catalogo = catalogo_procuracoes();
        assert!(catalogo.contains_key("CONSULTAAPOIORECEITAS52"));
        assert!(catalogo.contains_key("OBTERPROCURACAO41"));
        assert!(catalogo.contains_key("RELATORIOSITFIS92"));
        assert!(catalogo.contains_key("CONSDECLARACAO13"));
        assert!(catalogo.contains_key("TRANSDECLARACAO11"));
        assert!(catalogo.contains_key("GERARGUIA31"));
        assert!(catalogo.contains_key("INNOVAMSG63"));
        assert!(catalogo.contains_key("ENVIOXMLASSINADO81"));
    }

    #[test]
    fn regra_nao_se_aplica_para_consulta_apoio() {
        let regra = regra_procuracao_para_servico("CONSULTAAPOIORECEITAS52").unwrap();
        assert!(!regra.se_aplica());
        assert_eq!(regra, RegraProcuracao::NaoSeAplica);
    }

    #[test]
    fn regra_anyof_para_pgdasd() {
        let regra = regra_procuracao_para_servico("CONSDECLARACAO13").unwrap();
        assert!(regra.se_aplica());
        assert!(regra.codigo_satisfaz("00146"));
        assert!(!regra.codigo_satisfaz("00002"));
    }

    #[test]
    fn servico_bloqueado_allowlist() {
        assert!(CodigoServicoProcuracao::ParcelamentoSn.esta_bloqueado_allowlist());
        assert!(CodigoServicoProcuracao::ParcelamentoEsp.esta_bloqueado_allowlist());
        assert!(!CodigoServicoProcuracao::ConsDeclaracao13.esta_bloqueado_allowlist());
    }

    #[test]
    fn validacao_procuracao_vencida() {
        let agora = chrono::Utc::now();
        let validacao = ValidacaoProcuracao {
            contribuinte_cpf_cnpj: "98765432000190".to_owned(),
            procurador_cpf_cnpj: "12345678000190".to_owned(),
            codigo_servico: "00146".to_owned(),
            status: "VALIDA".to_owned(),
            valido_de: Some(agora - chrono::Duration::days(30)),
            valido_ate: Some(agora - chrono::Duration::days(1)), // vencida ontem
            verificado_em: agora,
            ambiente: "PRODUCAO".to_owned(),
        };
        assert!(!validacao.e_valida_agora(agora));
    }
}