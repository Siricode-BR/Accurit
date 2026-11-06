//! Regras de procuração e-CAC para o Integra Contador (SPEC §6).
//!
//! Fonte: tabela servicos-x-procuracoes do catálogo oficial
//! (`docs/Serpro/Integra-Contador/`) e [ADR-0007]. A allowlist (D4,
//! [ADR-0002]) bloqueia todos os sistemas de parcelamento (inclusive
//! serviços `Consultar`), `SICALC` `Emitir` (51, 53) e mantém
//! `PGMEI / DIVIDAATIVA24` como `AguardandoDecisao`.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Condição de contexto de uma regra `Condicional` (SPEC §6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CondicaoProcuracao {
    /// `n/a`/`sim¹`: o serviço não exige procuração; código a confirmar
    /// no catálogo vigente antes de habilitar (validar caso a caso).
    ValidarCasoACaso,
    /// Exige avaliação de tipo de contribuinte (PF/PJ) para escolher o
    /// conjunto de códigos aplicável.
    TipoContribuinte,
}

/// Regra de procuração para um serviço (SPEC §6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "tipo", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RegraProcuracao {
    /// Serviço não exige procuração (`n/a` no catálogo).
    NaoSeAplica,
    /// Qualquer um dos códigos listados satisfaz a autorização.
    AnyOf { codigos: Vec<String> },
    /// Exige avaliação de contexto (tipo de contribuinte, catálogo a
    /// confirmar).
    Condicional {
        codigos: Vec<String>,
        condicao: CondicaoProcuracao,
    },
}

impl RegraProcuracao {
    /// `true` quando a pré-validação via `OBTERPROCURACAO41` se aplica.
    pub const fn se_aplica(&self) -> bool {
        !matches!(self, Self::NaoSeAplica)
    }

    /// Verifica se um código retornado pela consulta cobre a regra.
    pub fn codigo_satisfaz(&self, codigo: &str) -> bool {
        match self {
            Self::NaoSeAplica => false,
            Self::AnyOf { codigos } => codigos.iter().any(|c| c == codigo),
            Self::Condicional { codigos, .. } => codigos.iter().any(|c| c == codigo),
        }
    }

    /// Códigos possivelmente exigidos pela regra.
    pub fn codigos_possiveis(&self) -> Vec<String> {
        match self {
            Self::NaoSeAplica => vec![],
            Self::AnyOf { codigos } | Self::Condicional { codigos, .. } => codigos.clone(),
        }
    }
}

/// Catálogo servicos-x-procuracoes do Integra Contador. Extensões
/// além dos serviços de referência exigem confirmação no catálogo
/// vigente antes de habilitar (SPEC §5.4/§6).
pub fn catalogo_procuracoes() -> HashMap<&'static str, RegraProcuracao> {
    let mut mapa = HashMap::new();
    let any = |codigo: &str| RegraProcuracao::AnyOf {
        codigos: vec![codigo.to_owned()],
    };

    // SICALC / PROCURACOES / AUTENTICAPROCURADOR: sem procuração.
    mapa.insert("CONSULTAAPOIORECEITAS52", RegraProcuracao::NaoSeAplica);
    mapa.insert("OBTERPROCURACAO41", RegraProcuracao::NaoSeAplica);
    mapa.insert("ENVIOXMLASSINADO81", RegraProcuracao::NaoSeAplica);

    // SITFIS (00002): RELATORIOSITFIS92 e SOLICITARPROTOCOLO91.
    mapa.insert("RELATORIOSITFIS92", any("00002"));
    mapa.insert("SOLICITARPROTOCOLO91", any("00002"));

    // PGDASD (inclui DEFIS) (00146).
    mapa.insert("CONSDECLARACAO13", any("00146"));
    mapa.insert("TRANSDECLARACAO11", any("00146"));

    // DCTFWeb (00103) — inclui TRANSDECLARACAO310.
    mapa.insert("GERARGUIA31", any("00103"));
    mapa.insert("TRANSDECLARACAO310", any("00103"));

    // Caixa Postal (00006) — MSGCONTRIBUINTE61, MSGDETALHAMENTO62, INNOVAMSG63.
    mapa.insert("MSGCONTRIBUINTE61", any("00006"));
    mapa.insert("MSGDETALHAMENTO62", any("00006"));
    mapa.insert("INNOVAMSG63", any("00006"));

    // DTE (00050) — CONSULTASITUACAODTE111; não confundir com 00006.
    mapa.insert("CONSULTASITUACAODTE111", any("00050"));

    // Pagamentos PAGTOWEB (00004).
    mapa.insert("PAGAMENTOS71", any("00004"));
    mapa.insert("COMPARRECADACAO72", any("00004"));
    mapa.insert("CONTACONSDOCARRPG73", any("00004"));

    // Regime de apuração REGIMEAPURACAO (00060).
    mapa.insert("EFETUAROPCAOREGIME101", any("00060"));

    // MEI DASNSIMEI (00229).
    mapa.insert("CONSULTIMADECREC152", any("00229"));
    mapa.insert("GERARDASEXCESSO153", any("00229"));

    // e-Processo EPROCESSO (00051).
    mapa.insert("CONSPROCPORINTER271", any("00051"));

    // Parcelamentos (PARCSN) 00076 OU 00188 (AnyOf) — sistema BLOQUEADO (D4).
    mapa.insert(
        "PARCSN",
        RegraProcuracao::AnyOf {
            codigos: vec!["00076".to_owned(), "00188".to_owned()],
        },
    );
    // Parcelamento Especial (PARCSN-ESP) 00125 — sistema BLOQUEADO (D4).
    mapa.insert("PARCSN-ESP", any("00125"));
    // PGMEI DIVIDAATIVA24 — AguardandoDecisao (D4): código confirmar primeiro.
    mapa.insert("DIVIDAATIVA24", any("00002"));

    // EVENTOSATUALIZACAO: `sim¹` mapeado para 00103; validar caso a caso.
    mapa.insert(
        "EVENTOSATUALIZACAO",
        RegraProcuracao::Condicional {
            codigos: vec!["00103".to_owned()],
            condicao: CondicaoProcuracao::ValidarCasoACaso,
        },
    );

    mapa
}

/// Trio de identificação exigido pelo SERPRO no XML base64 do Termo de
/// Autorização do modelo **Software House** (AUTENTICAPROCURADOR):
///
/// 1. `contratante` — CNPJ da Software House (nossa plataforma SaaS,
///    contratante da API e detentora do Bearer Token principal);
/// 2. `autorPedidoDados` — CNPJ do Escritório de Contabilidade
///    (Procurador, titular do certificado A1 que assina o termo);
/// 3. `contribuinte` — CPF/CNPJ do cliente do escritório (outorgante da
///    procuração eletrônica via e-CAC).
///
/// O XML em base64 enviado a `ENVIOXMLASSINADO81` precisa declarar os três
/// de forma explícita — diferente do modelo por escritório, em que
/// `contratante == autorPedidoDados` torna o termo dispensável.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrioSoftwareHouse {
    pub contratante_documento: String,
    pub contratante_nome: String,
    pub autor_pedido_dados_documento: String,
    pub autor_pedido_dados_nome: String,
    pub contribuinte_documento: String,
    pub contribuinte_nome: String,
}

impl TrioSoftwareHouse {
    /// Valida o trio conforme a estrutura de 3 CNPJs do SERPRO:
    /// contratante e autor do pedido de dados são obrigatoriamente CNPJ
    /// (o contribuinte pode ser CPF ou CNPJ) e os três nomes são
    /// obrigatórios. Delega as regras de documento para
    /// `AtoresTermoAutorizacao`, garantindo uma única fonte de verdade.
    pub fn validar(&self) -> Result<(), crate::domain::integra_contador::assinatura::ErroAssinatura> {
        crate::domain::integra_contador::assinatura::AtoresTermoAutorizacao {
            contratante_numero: self.contratante_documento.clone(),
            contratante_nome: self.contratante_nome.clone(),
            autor_pedido_dados_numero: self.autor_pedido_dados_documento.clone(),
            autor_pedido_dados_nome: self.autor_pedido_dados_nome.clone(),
            contribuinte_numero: self.contribuinte_documento.clone(),
            contribuinte_nome: self.contribuinte_nome.clone(),
        }
        .validar()
    }

    /// Converte o trio em `AtoresTermoAutorizacao` para o XML do termo.
    pub fn para_atores(
        &self,
    ) -> crate::domain::integra_contador::assinatura::AtoresTermoAutorizacao {
        crate::domain::integra_contador::assinatura::AtoresTermoAutorizacao {
            contratante_numero: self.contratante_documento.clone(),
            contratante_nome: self.contratante_nome.clone(),
            autor_pedido_dados_numero: self.autor_pedido_dados_documento.clone(),
            autor_pedido_dados_nome: self.autor_pedido_dados_nome.clone(),
            contribuinte_numero: self.contribuinte_documento.clone(),
            contribuinte_nome: self.contribuinte_nome.clone(),
        }
    }
}

/// Obtém a regra de procuração de um serviço.
pub fn regra_procuracao_para_servico(id_servico: &str) -> Option<RegraProcuracao> {
    catalogo_procuracoes().get(id_servico).cloned()
}

/// Serviço deve ser checado no catálogo de procurações antes de qualquer
/// chamada? (sempre que `autorPedidoDados` difere do `contribuinte`)
pub fn exige_verificacao_procuracao(id_servico: &str) -> bool {
    regra_procuracao_para_servico(id_servico)
        .map(|regra| regra.se_aplica())
        .unwrap_or(true)
}

/// Verificação de procuração para um serviço (resultado da consulta).
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
    /// Fail-closed: status deve indicar validade, dentro da janela.
    /// Falha = negar (SPEC §6: "negar a execução").
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

    /// A consulta cobre os códigos exigidos pelo serviço?
    pub fn cobre_regra(&self, regra: &RegraProcuracao) -> bool {
        regra.codigo_satisfaz(&self.codigo_servico)
    }
}

/// Serviço bloqueado pela allowlist (D4, ADR-0002):
/// - todos os sistemas de parcelamento (e.g. PARCSN, PARCSN-ESP, PERTSN,
///   RELPSN, PARCMEI, PARCMEI-ESP, PERTMEI, RELPMEI, PARC-PAEX, PARC-SIPADE),
///   inclusive serviços `Consultar`;
/// - `SICALC` `Emitir` (`CONSOLIDARGERARDARF51`, `GERARDARFCODBARRA53`);
/// - `PGMEI / DIVIDAATIVA24` fica `AguardandoDecisao` (separado).
pub fn bloqueado_allowlist(id_sistema: &str, id_servico: &str) -> bool {
    const SISTEMAS_PARCELAMENTO: [&str; 10] = [
        "PARCSN",
        "PARCSN-ESP",
        "PERTSN",
        "RELPSN",
        "PARCMEI",
        "PARCMEI-ESP",
        "PERTMEI",
        "RELPMEI",
        "PARC-PAEX",
        "PARC-SIPADE",
    ];
    if SISTEMAS_PARCELAMENTO.contains(&id_sistema.to_uppercase().as_str()) {
        return true;
    }
    if id_sistema.eq_ignore_ascii_case("SICALC")
        && matches!(
            id_servico.to_uppercase().as_str(),
            "CONSOLIDARGERARDARF51" | "GERARDARFCODBARRA53"
        )
    {
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalogo_tem_servicos_referencia() {
        let catalogo = catalogo_procuracoes();
        for id in [
            "CONSULTAAPOIORECEITAS52",
            "OBTERPROCURACAO41",
            "RELATORIOSITFIS92",
            "CONSDECLARACAO13",
            "TRANSDECLARACAO11",
            "GERARGUIA31",
            "MSGCONTRIBUINTE61",
            "INNOVAMSG63",
            "ENVIOXMLASSINADO81",
            "PARCSN",
        ] {
            assert!(catalogo.contains_key(id), "faltou {id}");
        }
    }

    #[test]
    fn regras_conforme_catalogo() {
        // n/a
        assert!(
            !regra_procuracao_para_servico("CONSULTAAPOIORECEITAS52")
                .unwrap()
                .se_aplica()
        );
        assert!(
            !regra_procuracao_para_servico("ENVIOXMLASSINADO81")
                .unwrap()
                .se_aplica()
        );
        // PGDASD 00146
        let regra = regra_procuracao_para_servico("CONSDECLARACAO13").unwrap();
        assert!(regra.se_aplica());
        assert!(regra.codigo_satisfaz("00146"));
        assert!(!regra.codigo_satisfaz("00002"));
        // DCTFWeb 00103
        assert!(
            regra_procuracao_para_servico("GERARGUIA31")
                .unwrap()
                .codigo_satisfaz("00103")
        );
        // SITFIS 00002
        assert!(
            regra_procuracao_para_servico("RELATORIOSITFIS92")
                .unwrap()
                .codigo_satisfaz("00002")
        );
        // Caixa Postal 00006
        assert!(
            regra_procuracao_para_servico("INNOVAMSG63")
                .unwrap()
                .codigo_satisfaz("00006")
        );
        // PARCSN AnyOf
        let parcsn = regra_procuracao_para_servico("PARCSN").unwrap();
        assert!(parcsn.codigo_satisfaz("00076") && parcsn.codigo_satisfaz("00188"));
    }

    #[test]
    fn validacao_procuracao_vencida_fail_closed() {
        let agora = chrono::Utc::now();
        for status in ["VENCIDA", "CANCELADA", "DESCONHECIDO", ""] {
            let validacao = ValidacaoProcuracao {
                contribuinte_cpf_cnpj: "98765432000190".to_owned(),
                procurador_cpf_cnpj: "12345678000190".to_owned(),
                codigo_servico: "00146".to_owned(),
                status: status.to_owned(),
                valido_de: None,
                valido_ate: None,
                verificado_em: agora,
                ambiente: "TRIAL".to_owned(),
            };
            assert!(!validacao.e_valida_agora(agora), "status {status}");
        }
        let valida = ValidacaoProcuracao {
            status: "VALIDA".to_owned(),
            valido_ate: Some(agora + chrono::TimeDelta::try_days(30).expect("dias")),
            contribuinte_cpf_cnpj: "98765432000190".to_owned(),
            procurador_cpf_cnpj: "12345678000190".to_owned(),
            codigo_servico: "00146".to_owned(),
            valido_de: None,
            verificado_em: agora,
            ambiente: "TRIAL".to_owned(),
        };
        assert!(valida.e_valida_agora(agora));
    }

    #[test]
    fn allowlist_d4_bloqueia() {
        // Parcelamentos: bloqueados em qualquer serviço, inclusive Consultar.
        assert!(bloqueado_allowlist("PARCSN", "SERVICO-QLQ"));
        assert!(bloqueado_allowlist("PARCMEI", "PARCELADO"));
        assert!(bloqueado_allowlist("PARC-SIPADE", "CONSULTA"));
        // SICALC Emitir (51, 53) bloqueado; outras ações Liberadas? Não —
        // apenas os dois serviços Sicalc de emissão de DARF:
        assert!(bloqueado_allowlist("SICALC", "GERARDARFCODBARRA53"));
        assert!(bloqueado_allowlist("SICALC", "CONSOLIDARGERARDARF51"));
        assert!(!bloqueado_allowlist("SICALC", "CONSULTAAPOIORECEITAS52"));
        // Demais serviços de referência não bloqueados
        assert!(!bloqueado_allowlist("PGDASD", "CONSDECLARACAO13"));
        assert!(!bloqueado_allowlist("DCTFWEB", "GERARGUIA31"));
    }

    #[test]
    fn trio_software_house_aceita_tres_cnpjs_validos() {
        let trio = TrioSoftwareHouse {
            contratante_documento: "11122233000183".to_owned(), // SaaS
            contratante_nome: "Accurit Tecnologia Ltda".to_owned(),
            autor_pedido_dados_documento: "11222333000181".to_owned(), // Escritório
            autor_pedido_dados_nome: "Escritório Modelo".to_owned(),
            contribuinte_documento: "98765432000198".to_owned(), // Cliente
            contribuinte_nome: "Cliente Ltda".to_owned(),
        };
        assert!(trio.validar().is_ok());
        assert_eq!(trio.para_atores().contratante_numero, "11122233000183");
    }

    #[test]
    fn trio_software_house_aceita_contribuinte_pf() {
        let trio = TrioSoftwareHouse {
            contratante_documento: "11122233000183".to_owned(),
            contratante_nome: "Accurit Tecnologia Ltda".to_owned(),
            autor_pedido_dados_documento: "11222333000181".to_owned(),
            autor_pedido_dados_nome: "Escritório Modelo".to_owned(),
            contribuinte_documento: "52998224725".to_owned(),
            contribuinte_nome: "João da Silva".to_owned(),
        };
        assert!(trio.validar().is_ok());
    }

    #[test]
    fn trio_software_house_rejeita_contratante_ou_autor_com_cpf() {
        let mut trio = TrioSoftwareHouse {
            contratante_documento: "52998224725".to_owned(), // CPF — inválido
            contratante_nome: "Accurit Tecnologia Ltda".to_owned(),
            autor_pedido_dados_documento: "11222333000181".to_owned(),
            autor_pedido_dados_nome: "Escritório Modelo".to_owned(),
            contribuinte_documento: "98765432000198".to_owned(),
            contribuinte_nome: "Cliente Ltda".to_owned(),
        };
        assert_eq!(
            trio.validar(),
            Err(crate::domain::integra_contador::assinatura::ErroAssinatura::TermoInvalido)
        );
        trio.contratante_documento = "11122233000183".to_owned();
        trio.autor_pedido_dados_documento = "52998224725".to_owned(); // CPF — inválido
        assert_eq!(
            trio.validar(),
            Err(crate::domain::integra_contador::assinatura::ErroAssinatura::TermoInvalido)
        );
    }
}
