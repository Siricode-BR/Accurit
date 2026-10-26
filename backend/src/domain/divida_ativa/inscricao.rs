//! Detalhe de uma inscrição na Dívida Ativa (consulta por
//! `/inscricao/{numeroInscricao}`, SPEC §4.3): campos renomeados com
//! `rename` explícito preservam `numeroPFNResponsavel` etc.
//!
//! O detalhamento é **leitura**: processo administrativo, receita
//! principal, prescrição e valores (ROADMAP Fase 4). Não existe
//! interpretação local de situações; a única tradução é
//! `indicadorPrescricaoSV8`: `SIM` → `true`, `NAO` → `false`,
//! ausente/branco → `None`.

use rust_decimal::Decimal;

use super::erros::ErroConsultaDivida;
use super::moeda::parse_moeda_ptbr_estrito;

/// Unidade da SERPRO (PFN responsável/inscrição): código e nome
/// preservados separadamente (SPEC §4.3).
#[derive(Debug, Clone, PartialEq)]
pub struct UnidadeServico {
    pub numero: String,
    pub nome: Option<String>,
}

/// Receita principal: código e nome preservados separadamente.
#[derive(Debug, Clone, PartialEq)]
pub struct ReceitaPrincipal {
    pub codigo: String,
    pub nome: Option<String>,
}

/// Sistema de origem (`codigoSistemaOrigem`/`descricaoSistemaOrigem` da
/// amostra oficial; presentes em alguns envelopes, opcionais).
#[derive(Debug, Clone, PartialEq)]
pub struct SistemaOrigem {
    pub codigo: String,
    pub descricao: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InscricaoDetalhe {
    /// Formatação preservada: "90 6 17 000964-19" (SPEC §4.3).
    pub inscricao_formatada: String,
    /// Somente dígitos, para busca por devedor e nova chamada.
    pub inscricao_digitos: String,
    pub processo_administrativo: Option<String>,
    pub situacao_descricao: Option<String>,
    pub data_inscricao: Option<String>,
    pub unidade_responsavel: Option<UnidadeServico>,
    pub unidade_inscricao: Option<UnidadeServico>,
    pub receita_principal: Option<ReceitaPrincipal>,
    /// `indicadorPrescricaoSV8`: `SIM` → `Some(true)`; `NAO` →
    /// `Some(false)`; ausente/branco → `None` (SPEC §4.3).
    pub prescrito: Option<bool>,
    pub valor_inscrito: Option<Decimal>,
    pub valor_consolidado: Option<Decimal>,
    pub valor_remanescente: Option<Decimal>,
    /// Campos de envelope presentes em algumas versões (opcionais).
    pub nome_devedor: Option<String>,
    pub tipo_devedor: Option<String>,
    pub nome_unidade: Option<String>,
    pub sistema_origem: Option<SistemaOrigem>,
}

impl InscricaoDetalhe {
    #[allow(clippy::result_large_err)]
    #[allow(clippy::too_many_arguments)]
    pub fn nova(
        inscricao_formatada: String,
        inscricao_digitos: String,
        processo_administrativo: Option<String>,
        situacao: Option<String>,
        data_inscricao: Option<String>,
        numero_pfn_responsavel: Option<String>,
        nome_pfn_responsavel: Option<String>,
        numero_pfn_inscricao: Option<String>,
        nome_pfn_inscricao: Option<String>,
        codigo_receita_principal: Option<String>,
        nome_receita: Option<String>,
        indicador_prescricao_sv8: Option<String>,
        valor_total_inscrito_moeda: Option<String>,
        valor_total_consolidado_moeda: Option<String>,
        valor_remanescente_moeda: Option<String>,
        nome_devedor: Option<String>,
        tipo_devedor: Option<String>,
        nome_unidade: Option<String>,
        codigo_sistema_origem: Option<String>,
        descricao_sistema_origem: Option<String>,
    ) -> Result<Self, ErroConsultaDivida> {
        let texto = |valor: &Option<String>| -> Option<String> {
            valor
                .as_ref()
                .map(|valor: &String| valor.trim())
                .filter(|t| !t.is_empty())
                .map(ToOwned::to_owned)
        };

        let moeda =
            |bruto: &Option<String>, rotulo: &str| -> Result<Option<Decimal>, ErroConsultaDivida> {
                match bruto {
                    None => Ok(None),
                    Some(valor_bruto) => {
                        parse_moeda_ptbr_estrito(valor_bruto)
                            .map(Some)
                            .map_err(|_| {
                                ErroConsultaDivida::RespostaInvalida(format!(
                                    "valor monetário inválido em {rotulo}: {valor_bruto}"
                                ))
                            })
                    }
                }
            };

        let valor_inscrito = moeda(&valor_total_inscrito_moeda, "valorTotalInscritoMoeda")?;
        let valor_consolidado =
            moeda(&valor_total_consolidado_moeda, "valorTotalConsolidadoMoeda")?;
        let valor_remanescente = moeda(&valor_remanescente_moeda, "valorRemanescenteMoeda")?;

        let prescrito = match indicador_prescricao_sv8.as_deref().map(str::trim) {
            None => None,
            Some("") => None,
            Some("SIM") => Some(true),
            Some("NAO") => Some(false),
            Some(outro) => {
                return Err(ErroConsultaDivida::RespostaInvalida(format!(
                    "indicadorPrescricaoSV8 inválido: {outro}"
                )));
            }
        };

        let unidade = |numero: &Option<String>, nome: &Option<String>| -> Option<UnidadeServico> {
            numero
                .as_ref()
                .map(|valor: &String| valor.trim())
                .filter(|valor| !valor.is_empty())
                .map(|numero| UnidadeServico {
                    numero: numero.to_owned(),
                    nome: texto(nome),
                })
        };

        let receita =
            |codigo: &Option<String>, nome: &Option<String>| -> Option<ReceitaPrincipal> {
                codigo
                    .as_ref()
                    .map(|valor: &String| valor.trim())
                    .filter(|valor| !valor.is_empty())
                    .map(|codigo| ReceitaPrincipal {
                        codigo: codigo.to_owned(),
                        nome: texto(nome),
                    })
            };

        let sistema =
            |codigo: &Option<String>, descricao: &Option<String>| -> Option<SistemaOrigem> {
                codigo
                    .as_ref()
                    .map(|valor: &String| valor.trim())
                    .filter(|valor| !valor.is_empty())
                    .map(|codigo| SistemaOrigem {
                        codigo: codigo.to_owned(),
                        descricao: texto(descricao),
                    })
            };

        Ok(Self {
            inscricao_formatada: inscricao_formatada.trim().to_owned(),
            inscricao_digitos,
            processo_administrativo: texto(&processo_administrativo),
            situacao_descricao: texto(&situacao),
            data_inscricao: texto(&data_inscricao),
            unidade_responsavel: unidade(&numero_pfn_responsavel, &nome_pfn_responsavel),
            unidade_inscricao: unidade(&numero_pfn_inscricao, &nome_pfn_inscricao),
            receita_principal: receita(&codigo_receita_principal, &nome_receita),
            prescrito,
            valor_inscrito,
            valor_consolidado,
            valor_remanescente,
            nome_devedor: texto(&nome_devedor),
            tipo_devedor: texto(&tipo_devedor),
            nome_unidade: texto(&nome_unidade),
            sistema_origem: sistema(&codigo_sistema_origem, &descricao_sistema_origem),
        })
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;

    use super::*;

    fn decimais(inteiros: i64, centavos: u32) -> Decimal {
        Decimal::new(inteiros * 100 + i64::from(centavos), 2)
    }

    #[test]
    fn detalhe_da_amostra_oficial() {
        let detalhe = InscricaoDetalhe::nova(
            "90 6 17 000964-19".to_owned(),
            "9061700096419".to_owned(),
            Some("46206 099222/99-01".to_owned()),
            Some("ATIVA NAO PRIORIZADA PARA AJUIZAMENTO".to_owned()),
            Some("16/12/2003".to_owned()),
            Some("1000".to_owned()),
            Some("PRIMEIRA REGIAO".to_owned()),
            Some("1000".to_owned()),
            Some("PRIMEIRA REGIAO".to_owned()),
            Some("3623".to_owned()),
            Some("DIV.ATIVA-CLT".to_owned()),
            Some("NAO".to_owned()),
            Some("805,06".to_owned()),
            Some("2.403,59".to_owned()),
            Some("805,06".to_owned()),
            None,
            None,
            None,
            Some("M".to_owned()),
            Some("Manual".to_owned()),
        )
        .expect("detalhe da amostra");

        assert_eq!(
            detalhe.processo_administrativo.as_deref(),
            Some("46206 099222/99-01")
        );
        assert_eq!(detalhe.prescrito, Some(false));
        assert_eq!(detalhe.valor_inscrito, Some(decimais(805, 6)));
        assert_eq!(detalhe.valor_consolidado, Some(decimais(2403, 59)));
        assert_eq!(detalhe.valor_remanescente, Some(decimais(805, 6)));
        let unidade = detalhe.unidade_responsavel.expect("unidade responsável");
        assert_eq!(unidade.numero, "1000");
        assert_eq!(unidade.nome.as_deref(), Some("PRIMEIRA REGIAO"));
        let receita = detalhe.receita_principal.expect("receita principal");
        assert_eq!(receita.codigo, "3623");
        assert_eq!(receita.nome.as_deref(), Some("DIV.ATIVA-CLT"));
        let sistema = detalhe.sistema_origem.expect("sistema de origem");
        assert_eq!(sistema.codigo, "M");
        assert_eq!(sistema.descricao.as_deref(), Some("Manual"));
    }

    #[test]
    fn prescricao_sim_nao_ausente_e_invalida() {
        fn detalhe_com(indicador: Option<String>) -> Result<InscricaoDetalhe, ErroConsultaDivida> {
            InscricaoDetalhe::nova(
                "90 6 17 000964-19".to_owned(),
                "9061700096419".to_owned(),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                indicador,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            )
        }

        assert_eq!(
            detalhe_com(Some("SIM".into())).expect("SIM").prescrito,
            Some(true)
        );
        assert_eq!(
            detalhe_com(Some("NAO".into())).expect("NAO").prescrito,
            Some(false)
        );
        assert_eq!(detalhe_com(None).expect("ausente").prescrito, None);
        assert_eq!(
            detalhe_com(Some("  ".into())).expect("branco").prescrito,
            None
        );
        assert!(detalhe_com(Some("MAYBE".into())).is_err());
    }

    #[test]
    fn moeda_invalida_no_detalhe() {
        let detalhe = InscricaoDetalhe::nova(
            "90 6 17 000964-19".to_owned(),
            "9061700096419".to_owned(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some("??".into()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        assert!(matches!(
            detalhe,
            Err(ErroConsultaDivida::RespostaInvalida(_))
        ));
    }
}
