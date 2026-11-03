//! Catálogo de serviços do Integra Contador (SPEC §5.4 e P4).
//!
//! - `versaoSistema` é **por serviço**: apenas `OBTERPROCURACAO41` (`"1"`)
//!   e `ENVIOXMLASSINADO81` (`"1.0"`) estão confirmados; os demais ficam
//!   `AguardandoDecisao` até confirmação no catálogo/PDF com fixture
//!   própria (P4). Nunca usar valor global.
//! - Allowlist (D4): sistemas de parcelamento bloqueados,
//!   `SICALC` `Emitir` (51/53) bloqueado (ver `bloqueado_allowlist`).
//! - Classificação de verbo conforme catálogo oficial:
//!   `CONSULTAAPOIORECEITAS52`/`ENVIOXMLASSINADO81`/`SOLICITARPROTOCOLO91`
//!   = Apoiar; `OBTERPROCURACAO41`/`CONSDECLARACAO13` = Consultar;
//!   `RELATORIOSITFIS92`/`GERARGUIA31` = Emitir;
//!   `TRANSDECLARACAO11` = Declarar; `INNOVAMSG63` = Monitorar.

use serde::{Deserialize, Serialize};

use crate::domain::integra_contador::envelope::VerboIntegraContador;
use crate::domain::integra_contador::regras_procuracao::bloqueado_allowlist;

/// Estado do serviço na allowlist do catálogo interno (SPEC §5.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EstadoCatalogo {
    /// Habilitado para uso (com `versaoSistema` confirmado).
    Habilitado,
    /// Aguardando decisão de habilitação (`versaoSistema` por confirmar
    /// — P4 / D4). Nenhuma chamada é permitida enquanto este estado.
    AguardandoDecisao,
    /// Bloqueado pela allowlist (D4): sistemas de parcelamento, SICALC
    /// Emitir de DARF.
    Bloqueado,
}

/// Serviço do catálogo. O catálogo interno registra, no mínimo,
/// `id_sistema`, `id_servico`, `versao_sistema`, verbo,
/// `tipos_contribuinte`, `regra_procuracao` e estado da allowlist
/// (SPEC §5.4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServicoReferencia {
    pub id_sistema: String,
    pub id_servico: String,
    /// `None` enquanto `versaoSistema` não estiver confirmado (P4).
    pub versao_sistema: Option<String>,
    pub descricao: String,
    pub verbo: VerboIntegraContador,
    pub estado: EstadoCatalogo,
    /// Tipos de contribuinte aceitos (`1` CPF, `2` CNPJ).
    pub tipos_contribuinte: Vec<u8>,
}

fn catalogo() -> Vec<ServicoReferencia> {
    let servicos: Vec<(
        &str,
        &str,
        Option<&str>,
        &str,
        VerboIntegraContador,
        EstadoCatalogo,
        Vec<u8>,
        &str,
    )> = vec![
        // ===== Habilitados (SPEC §5.4) =====
        (
            "PROCURACOES",
            "OBTERPROCURACAO41",
            Some("1"),
            "Obter Procuração (e-CAC)",
            VerboIntegraContador::Consultar,
            EstadoCatalogo::Habilitado,
            vec![1, 2],
            "Consulta a procuração vigente entre outorgante e outorgado.",
        ),
        (
            "AUTENTICAPROCURADOR",
            "ENVIOXMLASSINADO81",
            Some("1.0"),
            "Envio de XML assinado (Termo de Autorização)",
            VerboIntegraContador::Apoiar,
            EstadoCatalogo::Habilitado,
            vec![1, 2],
            "Valida o Termo de Autorização assinado e devolve autenticar_procurador_token.",
        ),
        // ===== AguardandoDecisao (P4: versaoSistema a confirmar; D4) =====
        (
            "SICALC",
            "CONSULTAAPOIORECEITAS52",
            None,
            "Consulta de apoio de receitas (SICALC)",
            VerboIntegraContador::Apoiar,
            EstadoCatalogo::AguardandoDecisao,
            vec![1, 2],
            "Serviço de apoio SICALC.",
        ),
        (
            "SITFIS",
            "RELATORIOSITFIS92",
            None,
            "Relatório Situação Fiscal (SITFIS)",
            VerboIntegraContador::Emitir,
            EstadoCatalogo::AguardandoDecisao,
            vec![2],
            "Emissão do relatório SITFIS.",
        ),
        (
            "SITFIS",
            "SOLICITARPROTOCOLO91",
            None,
            "Solicitar protocolo de processamento (SITFIS)",
            VerboIntegraContador::Apoiar,
            EstadoCatalogo::AguardandoDecisao,
            vec![2],
            "Consulta protocolo emitido para o relatório.",
        ),
        (
            "PGDASD",
            "CONSDECLARACAO13",
            None,
            "Consultar declaração PGDASD",
            VerboIntegraContador::Consultar,
            EstadoCatalogo::AguardandoDecisao,
            vec![2],
            "Consulta declaração transmitida.",
        ),
        (
            "PGDASD",
            "TRANSDECLARACAO11",
            None,
            "Transmitir declaração PGDASD",
            VerboIntegraContador::Declarar,
            EstadoCatalogo::AguardandoDecisao,
            vec![2],
            "Transmissão mutável — exige idempotência e confirmação.",
        ),
        (
            "DCTFWEB",
            "GERARGUIA31",
            None,
            "Gerar guia (DCTFWeb)",
            VerboIntegraContador::Emitir,
            EstadoCatalogo::AguardandoDecisao,
            vec![2],
            "Emissão mutável — exige idempotência e confirmação.",
        ),
        (
            "CAIXAPOSTAL",
            "INNOVAMSG63",
            None,
            "Monitorar mensagens (Caixa Postal/Inova)",
            VerboIntegraContador::Monitorar,
            EstadoCatalogo::AguardandoDecisao,
            vec![2],
            "Acompanhamento de processamento.",
        ),
        (
            "PGMEI",
            "DIVIDAATIVA24",
            None,
            "Consultar dívida ativa do MEI",
            VerboIntegraContador::Consultar,
            EstadoCatalogo::AguardandoDecisao,
            vec![2],
            "D4: código de procuração a confirmar antes de habilitar.",
        ),
    ];

    servicos
        .into_iter()
        .map(
            |(sistema, servico_id, versao, descricao, verbo, mut estado, tipos, _)| {
                // D4: allowlist adaptada por serviço real.
                if bloqueado_allowlist(sistema, servico_id) {
                    estado = EstadoCatalogo::Bloqueado;
                }
                ServicoReferencia {
                    id_sistema: sistema.to_owned(),
                    id_servico: servico_id.to_owned(),
                    versao_sistema: versao.map(ToOwned::to_owned),
                    descricao: descricao.to_owned(),
                    verbo,
                    estado,
                    tipos_contribuinte: tipos,
                }
            },
        )
        .collect()
}

/// Busca de serviço por chave `SISTEMA/SERVICO` (case-insensitive).
pub fn buscar_servico(id_sistema: &str, id_servico: &str) -> Option<ServicoReferencia> {
    catalogo().into_iter().find(|s| {
        s.id_sistema.eq_ignore_ascii_case(id_sistema)
            && s.id_servico.eq_ignore_ascii_case(id_servico)
    })
}

/// Erros de validação de serviço do catálogo.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ErroCatalogo {
    #[error("serviço não consta no catálogo oficial: {0}")]
    Inexistente(String),
    #[error("serviço AguardandoDecisao: {0} (P4/D4)")]
    AguardandoDecisao(String),
    #[error("serviço bloqueado pela allowlist (D4): {0}")]
    Bloqueado(String),
    #[error("verbo {solicitado} incompatível; catálogo define {catalogo}")]
    VerboIncompativel {
        solicitado: String,
        catalogo: String,
    },
    #[error(
        "versaoSistema obrigatório e divergente do catálogo: recebido {recebido}, esperado {esperado}"
    )]
    VersaoDivergente { recebido: String, esperado: String },
    #[error("tipo de contribuinte {0} não aceito pelo serviço")]
    TipoContribuinteInaceitavel(u8),
}

/// Valida o pedido contra o catálogo/allowlist: existência, estado, verbo,
/// versão e tipo de contribuinte. Fail-closed — erro para qualquer
/// divergência (SPEC §5.4).
pub fn validar_servico(
    id_sistema: &str,
    id_servico: &str,
    versao_sistema_informada: Option<&str>,
    verbo: VerboIntegraContador,
    tipo_contribuinte: u8,
) -> Result<ServicoReferencia, ErroCatalogo> {
    // Allowlist (D4) avaliada ANTES do catálogo: sistemas bloqueados
    // (parcelamento, SICALC Emitir de DARF) são Barbados como Bloqueado
    // mesmo que não estejam enumerados no catálogo interno — fail-closed,
    // sem vazar detalhes do catálogo.
    if bloqueado_allowlist(id_sistema, id_servico) {
        return Err(ErroCatalogo::Bloqueado(format!(
            "{}/{}",
            id_sistema.to_uppercase(),
            id_servico.to_uppercase()
        )));
    }

    let servico = buscar_servico(id_sistema, id_servico)
        .ok_or_else(|| ErroCatalogo::Inexistente(format!("{id_sistema}/{id_servico}")))?;

    match servico.estado {
        EstadoCatalogo::Bloqueado => {
            return Err(ErroCatalogo::Bloqueado(format!(
                "{}/{}",
                servico.id_sistema, servico.id_servico
            )));
        }
        EstadoCatalogo::AguardandoDecisao => {
            return Err(ErroCatalogo::AguardandoDecisao(format!(
                "{}/{}",
                servico.id_sistema, servico.id_servico
            )));
        }
        EstadoCatalogo::Habilitado => {}
    }

    if servico.verbo != verbo {
        return Err(ErroCatalogo::VerboIncompativel {
            solicitado: verbo.to_owned().to_string(),
            catalogo: servico.verbo.to_string(),
        });
    }

    // Somente serviços Habilitados possuem versão confirmada; quando o
    // próprio catálogo tem versão, a informada deve ser idêntica (nunca
    // inventar valor global — P4).
    match (servico.versao_sistema.as_deref(), versao_sistema_informada) {
        (Some(catalogo), Some(informada)) => {
            if catalogo != informada {
                return Err(ErroCatalogo::VersaoDivergente {
                    recebido: informada.to_owned(),
                    esperado: catalogo.to_owned(),
                });
            }
        }
        (Some(catalogo), None) => {
            return Err(ErroCatalogo::VersaoDivergente {
                recebido: String::from("<ausente>"),
                esperado: catalogo.to_owned(),
            });
        }
        (None, _) => {
            return Err(ErroCatalogo::VersaoDivergente {
                recebido: versao_sistema_informada.unwrap_or("<ausente>").to_owned(),
                esperado: String::from("<a confirmar no catálogo>"),
            });
        }
    }

    if !servico.tipos_contribuinte.contains(&tipo_contribuinte) {
        return Err(ErroCatalogo::TipoContribuinteInaceitavel(tipo_contribuinte));
    }

    Ok(servico)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apenas_dois_servicos_habilitados() {
        let habilitados: Vec<_> = catalogo()
            .into_iter()
            .filter(|s| s.estado == EstadoCatalogo::Habilitado)
            .collect();
        assert_eq!(habilitados.len(), 2);
        let procuracoes = habilitados
            .iter()
            .find(|s| s.id_servico == "OBTERPROCURACAO41")
            .expect("OBTER41");
        assert_eq!(procuracoes.versao_sistema.as_deref(), Some("1"));
        assert_eq!(procuracoes.verbo, VerboIntegraContador::Consultar);
        let envio = habilitados
            .iter()
            .find(|s| s.id_servico == "ENVIOXMLASSINADO81")
            .expect("ENVIO81");
        assert_eq!(envio.versao_sistema.as_deref(), Some("1.0"));
        assert_eq!(envio.verbo, VerboIntegraContador::Apoiar);
    }

    #[test]
    fn verbos_conforme_catalogo_oficial() {
        assert_eq!(
            buscar_servico("SICALC", "CONSULTAAPOIORECEITAS52")
                .expect("sicalc")
                .verbo,
            VerboIntegraContador::Apoiar
        );
        assert_eq!(
            buscar_servico("CAIXAPOSTAL", "INNOVAMSG63")
                .expect("inova")
                .verbo,
            VerboIntegraContador::Monitorar
        );
        assert_eq!(
            buscar_servico("SITFIS", "RELATORIOSITFIS92")
                .expect("sitfis")
                .verbo,
            VerboIntegraContador::Emitir
        );
        assert_eq!(
            buscar_servico("PGDASD", "TRANSDECLARACAO11")
                .expect("trans")
                .verbo,
            VerboIntegraContador::Declarar
        );
    }

    #[test]
    fn acima_dois_niveis_validacao_fail_closed() {
        // Enexistente:
        assert!(matches!(
            validar_servico(
                "NOBODY",
                "SERVICO",
                Some("1"),
                VerboIntegraContador::Consultar,
                2
            ),
            Err(ErroCatalogo::Inexistente(_))
        ));
        // Aguardando decisão (P4):
        assert!(matches!(
            validar_servico(
                "PGDASD",
                "CONSDECLARACAO13",
                Some("1"),
                VerboIntegraContador::Consultar,
                2
            ),
            Err(ErroCatalogo::AguardandoDecisao(_))
        ));
        // Habilitado com verbo errado:
        assert!(matches!(
            validar_servico(
                "PROCURACOES",
                "OBTERPROCURACAO41",
                Some("1"),
                VerboIntegraContador::Emitir,
                2
            ),
            Err(ErroCatalogo::VerboIncompativel { .. })
        ));
        // Habilitado com versão divergente:
        assert!(matches!(
            validar_servico(
                "PROCURACOES",
                "OBTERPROCURACAO41",
                Some("2"),
                VerboIntegraContador::Consultar,
                2
            ),
            Err(ErroCatalogo::VersaoDivergente { .. })
        ));
        // bloqueio (D4):
        assert!(matches!(
            validar_servico(
                "SICALC",
                "GERARDARFCODBARRA53",
                Some("1"),
                VerboIntegraContador::Emitir,
                2
            ),
            Err(ErroCatalogo::Bloqueado(_))
        ));
    }

    #[test]
    fn validacao_ok_para_servico_habilitado() {
        let servico = validar_servico(
            "PROCURACOES",
            "OBTERPROCURACAO41",
            Some("1"),
            VerboIntegraContador::Consultar,
            2,
        )
        .expect("deve validar");
        assert_eq!(servico.estado, EstadoCatalogo::Habilitado);
        // Tipo não aceito:
        assert!(matches!(
            validar_servico(
                "PROCURACOES",
                "OBTERPROCURACAO41",
                Some("1"),
                VerboIntegraContador::Consultar,
                99
            ),
            Err(_)
        ));
    }
}
