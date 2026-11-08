use super::envelope::VerboIntegraContador;
use super::erros::ErroIntegraContador;
use super::regras_procuracao::{RegraProcuracao, regra_procuracao_para_servico};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EstadoCatalogo {
    Habilitado,
    AguardandoConfirmacao,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServicoIntegraContador {
    pub id_sistema: &'static str,
    pub id_servico: &'static str,
    pub versao_sistema: Option<&'static str>,
    pub verbo: VerboIntegraContador,
    pub estado: EstadoCatalogo,
    pub regra_procuracao: RegraProcuracao,
}

const SERVICOS: &[(&str, &str, Option<&str>, VerboIntegraContador, bool)] = &[
    (
        "SICALC",
        "CONSULTAAPOIORECEITAS52",
        None,
        VerboIntegraContador::Apoiar,
        false,
    ),
    (
        "PROCURACOES",
        "OBTERPROCURACAO41",
        Some("1"),
        VerboIntegraContador::Consultar,
        true,
    ),
    (
        "AUTENTICAPROCURADOR",
        "ENVIOXMLASSINADO81",
        Some("1.0"),
        VerboIntegraContador::Apoiar,
        false,
    ),
    // ===== FASE A — serviços priorizados pela rotina fiscal =====
    // Situação Fiscal / Relatório de pendências (SITFIS, procuração 00002).
    (
        "SITFIS",
        "RELATORIOSITFIS92",
        None,
        VerboIntegraContador::Emitir,
        false,
    ),
    // PGDAS-D/DAS (Simples Nacional, procuração 00146).
    (
        "PGDASD",
        "CONSDECLARACAO13",
        None,
        VerboIntegraContador::Consultar,
        false,
    ),
    (
        "PGDASD",
        "TRANSDECLARACAO11",
        None,
        VerboIntegraContador::Declarar,
        false,
    ),
    (
        "PGDASD",
        "GERARDAS12",
        None,
        VerboIntegraContador::Emitir,
        false,
    ),
    // DAS MEI (PGMEI — sem procuração no catálogo oficial).
    (
        "PGMEI",
        "GERARDASPDF21",
        None,
        VerboIntegraContador::Emitir,
        false,
    ),
    (
        "PGMEI",
        "GERARDASCODBARRA22",
        None,
        VerboIntegraContador::Emitir,
        false,
    ),
    // DCTFWeb (procuração 00103).
    (
        "DCTFWEB",
        "GERARGUIA31",
        None,
        VerboIntegraContador::Emitir,
        false,
    ),
    // Caixa Postal (procuração 00006).
    (
        "CAIXAPOSTAL",
        "INNOVAMSG63",
        None,
        VerboIntegraContador::Monitorar,
        false,
    ),
    (
        "CAIXAPOSTAL",
        "MSGCONTRIBUINTE61",
        None,
        VerboIntegraContador::Consultar,
        false,
    ),
    // Parcelamentos (Integra-Parcelamentos) — apenas consulta (ADR-0013),
    // procuração 00076 OU 00188.
    (
        "PARCSN",
        "PEDIDOSPARC163",
        None,
        VerboIntegraContador::Consultar,
        false,
    ),
    (
        "PARCSN",
        "OBTERPARC164",
        None,
        VerboIntegraContador::Consultar,
        false,
    ),
    (
        "PARCSN",
        "PARCELASPARAGERAR162",
        None,
        VerboIntegraContador::Consultar,
        false,
    ),
    (
        "PARCSN",
        "DETPAGTOPARC165",
        None,
        VerboIntegraContador::Consultar,
        false,
    ),
];

pub fn catalogo_de_referencia() -> Vec<ServicoIntegraContador> {
    SERVICOS
        .iter()
        .map(
            |(id_sistema, id_servico, versao_sistema, verbo, habilitado)| ServicoIntegraContador {
                id_sistema,
                id_servico,
                versao_sistema: *versao_sistema,
                verbo: *verbo,
                estado: if *habilitado && versao_sistema.is_some() {
                    EstadoCatalogo::Habilitado
                } else {
                    EstadoCatalogo::AguardandoConfirmacao
                },
                regra_procuracao: regra_procuracao_para_servico(id_servico).unwrap_or(
                    RegraProcuracao::Condicional {
                        codigos: Vec::new(),
                        condicao: super::regras_procuracao::CondicaoProcuracao::ValidarCasoACaso,
                    },
                ),
            },
        )
        .collect()
}

/// Bloqueio combinado pela allowlist considerando o verbo ([ADR-0013]):
/// - sistemas de parcelamento seguem bloqueados para `Emitir`/`Declarar`/
///   `Apoiar`/`Monitorar`, mas serviços `Consultar` de Integra-Parcelamentos
///   são viáveis para a rotina fiscal (dependem do catálogo configurável);
/// - `SICALC` `Emitir` (51, 53) permanece bloqueado em qualquer cenário.
pub fn bloqueado_por_verbo(
    id_sistema: &str,
    id_servico: &str,
    verbo: VerboIntegraContador,
) -> bool {
    if !super::regras_procuracao::bloqueado_allowlist(id_sistema, id_servico) {
        return false;
    }
    if super::regras_procuracao::sistema_de_parcelamento(id_sistema)
        && verbo == VerboIntegraContador::Consultar
    {
        return false;
    }
    true
}

pub fn resolver_servico(
    id_sistema: &str,
    id_servico: &str,
    versao_sistema: &str,
    verbo: VerboIntegraContador,
) -> Result<ServicoIntegraContador, ErroIntegraContador> {
    if bloqueado_por_verbo(id_sistema, id_servico, verbo) {
        return Err(ErroIntegraContador::ServicoNaoHabilitado);
    }

    let Some((sistema, servico, versao, verbo_catalogo, habilitado)) =
        SERVICOS.iter().find(|(sistema, servico, _, _, _)| {
            sistema.eq_ignore_ascii_case(id_sistema) && servico.eq_ignore_ascii_case(id_servico)
        })
    else {
        return Err(ErroIntegraContador::ServicoNaoHabilitado);
    };

    if *verbo_catalogo != verbo {
        return Err(ErroIntegraContador::ServicoNaoHabilitado);
    }
    let Some(versao_confirmada) = versao else {
        return Err(ErroIntegraContador::VersaoNaoConfirmada);
    };
    if *versao_confirmada != versao_sistema {
        return Err(ErroIntegraContador::VersaoNaoConfirmada);
    }

    Ok(ServicoIntegraContador {
        id_sistema: sistema,
        id_servico: servico,
        versao_sistema: Some(versao_confirmada),
        verbo,
        estado: if *habilitado {
            EstadoCatalogo::Habilitado
        } else {
            EstadoCatalogo::AguardandoConfirmacao
        },
        regra_procuracao: regra_procuracao_para_servico(servico).unwrap_or(
            RegraProcuracao::Condicional {
                codigos: Vec::new(),
                condicao: super::regras_procuracao::CondicaoProcuracao::ValidarCasoACaso,
            },
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn habilita_apenas_servico_confirmado() {
        let servico = resolver_servico(
            "PROCURACOES",
            "OBTERPROCURACAO41",
            "1",
            VerboIntegraContador::Consultar,
        )
        .expect("serviço documentado");
        assert_eq!(servico.estado, EstadoCatalogo::Habilitado);
    }

    #[test]
    fn bloqueia_servicos_com_versao_pendente() {
        assert_eq!(
            resolver_servico(
                "PGDASD",
                "TRANSDECLARACAO11",
                "1",
                VerboIntegraContador::Declarar,
            ),
            Err(ErroIntegraContador::VersaoNaoConfirmada)
        );
    }

    #[test]
    fn bloqueia_verbos_mutaveis_de_parcelamento() {
        // ADR-0013: parcelamentos liberados apenas para `Consultar`.
        for verbo in [
            VerboIntegraContador::Apoiar,
            VerboIntegraContador::Declarar,
            VerboIntegraContador::Emitir,
            VerboIntegraContador::Monitorar,
        ] {
            assert_eq!(
                resolver_servico("PARCSN", "OBTERPARC164", "1", verbo),
                Err(ErroIntegraContador::ServicoNaoHabilitado),
                "verbo {verbo:?} deve continuar bloqueado"
            );
        }
    }

    #[test]
    fn parcelamentos_consulta_ficam_pendentes_de_versao() {
        // ADR-0013: `Consultar` é viável, mas a versão dos serviços do
        // Integra-Parcelamentos não foi confirmada (P4): falha explícita.
        assert_eq!(
            resolver_servico(
                "PARCSN",
                "OBTERPARC164",
                "1",
                VerboIntegraContador::Consultar
            ),
            Err(ErroIntegraContador::VersaoNaoConfirmada)
        );
        assert_eq!(
            resolver_servico(
                "PARCSN",
                "PARCELASPARAGERAR162",
                "1",
                VerboIntegraContador::Consultar
            ),
            Err(ErroIntegraContador::VersaoNaoConfirmada)
        );
    }

    #[test]
    fn sicalc_emitir_permanece_bloqueado() {
        assert_eq!(
            resolver_servico(
                "SICALC",
                "GERARDARFCODBARRA53",
                "1",
                VerboIntegraContador::Emitir
            ),
            Err(ErroIntegraContador::ServicoNaoHabilitado)
        );
    }
}
