//! Agregado do devedor (consulta por CPF/CNPJ — SPEC §4.4).
//!
//! `ResultadoDevedor` codifica os estados distintos exigidos em §4.5:
//! "sem pendências" ≠ erro de integração ≠ falha de autorização. Os
//! valores monetários são `Decimal` já saneados; textos exibidos
//! preservam o que a SERPRO retorna, sem interpretação local.

use rust_decimal::Decimal;

use super::moeda::parse_moeda_ptbr_estrito;
use crate::domain::clientes::CpfCnpj;

#[derive(Debug, Clone, PartialEq)]
pub struct DevedorConsulta {
    /// Chave consultada (CPF/CNPJ em dígitos, com DV local válido).
    pub chave: CpfCnpj,
    pub resultado: ResultadoDevedor,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResultadoDevedor {
    /// SERPRO respondeu com inscrições do devedor.
    Pendencias { inscricoes: Vec<InscricaoDevedor> },
    /// Resposta vazia inequívoca (SPEC §4.4 passo 7): nunca inventar dados.
    SemPendencias,
}

impl ResultadoDevedor {
    pub const fn sem_pendencias(&self) -> bool {
        matches!(self, Self::SemPendencias)
    }
}

/// Item retornado por `/devedor/{cpfCnpj}` (amostra oficial: objeto plano,
/// SPEC §4.3). Moeda e documento são sanitizados; o resto aparece como
/// retornado (sem interpretação).
#[derive(Debug, Clone, PartialEq)]
pub struct InscricaoDevedor {
    pub numero_inscricao: Option<String>,
    pub numero_processo: Option<String>,
    pub situacao_inscricao: Option<String>,
    pub situacao_descricao: Option<String>,
    pub nome_devedor: Option<String>,
    pub tipo_devedor: Option<String>,
    /// `valorTotalConsolidadoMoeda` convertido para `Decimal`.
    pub valor_total_consolidado: Option<Decimal>,
    /// `cpfCnpj` (dígitos) com DV; DV inválido/ausente → `None`.
    pub cpf_cnpj: Option<CpfCnpj>,
    pub codigo_sida: Option<String>,
    pub nome_unidade: Option<String>,
    pub codigo_comprot: Option<String>,
    pub codigo_uorg: Option<String>,
}

impl InscricaoDevedor {
    /// Constrói um item saneando os campos derivados; moeda inválida
    /// retorna erro de resposta inválida (o vec vira estado
    /// `RESPOSTA_INVALIDA` no use case, em vez de string monstruosa).
    #[allow(clippy::result_large_err)]
    #[allow(clippy::too_many_arguments)]
    pub fn nova(
        numero_inscricao: Option<String>,
        numero_processo: Option<String>,
        situacao_inscricao: Option<String>,
        situacao_descricao: Option<String>,
        nome_devedor: Option<String>,
        tipo_devedor: Option<String>,
        valor_total_consolidado_moeda: Option<String>,
        cpf_cnpj_digitos: Option<String>,
        codigo_sida: Option<String>,
        nome_unidade: Option<String>,
        codigo_comprot: Option<String>,
        codigo_uorg: Option<String>,
    ) -> Result<Self, super::erros::ErroConsultaDivida> {
        let trime = |texto_opt: &Option<String>| -> Option<String> {
            texto_opt
                .as_ref()
                .map(|valor: &String| valor.trim())
                .filter(|texto| !texto.is_empty())
                .map(ToOwned::to_owned)
        };

        let valor_total_consolidado = match valor_total_consolidado_moeda {
            None => None,
            Some(moeda) => match parse_moeda_ptbr_estrito(&moeda) {
                Ok(valor) => Some(valor),
                Err(_) => {
                    return Err(super::erros::ErroConsultaDivida::RespostaInvalida(format!(
                        "valor monetário inválido: {moeda}"
                    )));
                }
            },
        };

        Ok(Self {
            numero_inscricao: trime(&numero_inscricao),
            numero_processo: trime(&numero_processo),
            situacao_inscricao: trime(&situacao_inscricao),
            situacao_descricao: trime(&situacao_descricao),
            nome_devedor: trime(&nome_devedor),
            tipo_devedor: trime(&tipo_devedor),
            valor_total_consolidado,
            cpf_cnpj: cpf_cnpj_digitos
                .as_deref()
                .map(str::trim)
                .filter(|digitos| !digitos.is_empty())
                .and_then(|digitos| CpfCnpj::novo(digitos).ok()),
            codigo_sida: trime(&codigo_sida),
            nome_unidade: trime(&nome_unidade),
            codigo_comprot: trime(&codigo_comprot),
            codigo_uorg: trime(&codigo_uorg),
        })
    }

    /// "Objeto plano" com todos os campos nulos/brancos não é inscrição:
    /// é corpo vazio (SPEC §4.4 passo 7).
    pub fn e_vazio(&self) -> bool {
        self.numero_inscricao.is_none()
            && self.numero_processo.is_none()
            && self.situacao_inscricao.is_none()
            && self.situacao_descricao.is_none()
            && self.nome_devedor.is_none()
            && self.tipo_devedor.is_none()
            && self.valor_total_consolidado.is_none()
            && self.cpf_cnpj.is_none()
            && self.codigo_sida.is_none()
            && self.nome_unidade.is_none()
            && self.codigo_comprot.is_none()
            && self.codigo_uorg.is_none()
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
    fn item_construido_sanitiza_moeda_e_documento() {
        let item = InscricaoDevedor::nova(
            Some("10 5 03 003880-14".to_owned()),
            Some("10166 677771/2014-12".to_owned()),
            Some("121105".to_owned()),
            Some("ATIVA NAO PRIORIZADA PARA AJUIZAMENTO".to_owned()),
            Some("DEVEDOR DA SILVA".to_owned()),
            Some("PRINCIPAL".to_owned()),
            Some("4.457,56".to_owned()),
            Some("529.982.247-25".to_owned()),
            Some("1000".to_owned()),
            Some("PROCURADORIA REGIONAL DA FAZENDA NACIONAL DA 1A REGIAO".to_owned()),
            Some("1101951".to_owned()),
            Some("0005786".to_owned()),
        )
        .expect("item válido");

        assert_eq!(item.numero_inscricao.as_deref(), Some("10 5 03 003880-14"));
        assert_eq!(item.valor_total_consolidado, Some(decimais(4457, 56)));
        let documento = item.cpf_cnpj.expect("documento válido");
        assert_eq!(documento.como_str(), "52998224725");
    }

    #[test]
    fn moeda_invalida_vira_resposta_invalida() {
        let resultado = InscricaoDevedor::nova(
            None,
            None,
            None,
            None,
            None,
            None,
            Some("abc".to_owned()),
            None,
            None,
            None,
            None,
            None,
        );

        assert!(matches!(
            resultado,
            Err(super::super::erros::ErroConsultaDivida::RespostaInvalida(_))
        ));
    }

    #[test]
    fn item_vazio_e_detectado() {
        let item = InscricaoDevedor::nova(
            None, None, None, None, None, None, None, None, None, None, None, None,
        )
        .expect("item vazio válido");

        assert!(item.e_vazio());
    }

    #[test]
    fn resultado_sem_pendencias() {
        assert!(ResultadoDevedor::SemPendencias.sem_pendencias());
        assert!(!ResultadoDevedor::Pendencias { inscricoes: vec![] }.sem_pendencias());
    }
}
