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
        "SITFIS",
        "RELATORIOSITFIS92",
        None,
        VerboIntegraContador::Emitir,
        false,
    ),
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
        "DCTFWEB",
        "GERARGUIA31",
        None,
        VerboIntegraContador::Emitir,
        false,
    ),
    (
        "CAIXAPOSTAL",
        "INNOVAMSG63",
        None,
        VerboIntegraContador::Monitorar,
        false,
    ),
    (
        "AUTENTICAPROCURADOR",
        "ENVIOXMLASSINADO81",
        Some("1.0"),
        VerboIntegraContador::Apoiar,
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

pub fn resolver_servico(
    id_sistema: &str,
    id_servico: &str,
    versao_sistema: &str,
    verbo: VerboIntegraContador,
) -> Result<ServicoIntegraContador, ErroIntegraContador> {
    if super::regras_procuracao::bloqueado_allowlist(id_sistema, id_servico) {
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
    fn bloqueia_todos_os_verbos_de_servico_de_parcelamento() {
        for verbo in [
            VerboIntegraContador::Apoiar,
            VerboIntegraContador::Consultar,
            VerboIntegraContador::Declarar,
            VerboIntegraContador::Emitir,
            VerboIntegraContador::Monitorar,
        ] {
            assert_eq!(
                resolver_servico("PARCSN", "OBTERPARC164", "1", verbo),
                Err(ErroIntegraContador::ServicoNaoHabilitado)
            );
        }
    }
}
