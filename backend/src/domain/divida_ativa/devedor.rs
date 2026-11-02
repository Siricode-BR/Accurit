use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use thiserror::Error;
use uuid::Uuid;

use crate::domain::clientes::CpfCnpj;
use crate::domain::divida_ativa::erros::ErroDividaAtiva;
use crate::domain::serpro::ambiente::AmbienteSerpro;

/// Fonte da chave usada na consulta (auditoria `fonte_consulta`,
/// SPEC §9.11). O fallback pelo Código de Contribuinte (D2) é o
/// segundo valor; fica por trás de flag até haver contrato real.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FonteConsulta {
    CpfCnpj,
    CodigoContribuinte,
}

impl FonteConsulta {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CpfCnpj => "CPF_CNPJ",
            Self::CodigoContribuinte => "CODIGO_CONTRIBUINTE",
        }
    }
}

impl fmt::Display for FonteConsulta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("fonte de consulta inválida: {0}")]
pub struct FonteConsultaInvalida(String);

impl FromStr for FonteConsulta {
    type Err = FonteConsultaInvalida;

    fn from_str(valor: &str) -> Result<Self, Self::Err> {
        match valor {
            "CPF_CNPJ" => Ok(Self::CpfCnpj),
            "CODIGO_CONTRIBUINTE" => Ok(Self::CodigoContribuinte),
            outro => Err(FonteConsultaInvalida(outro.to_owned())),
        }
    }
}

/// Estado agregado da resposta informada pela SERPRO. 200 com dados é
/// `COM_PENDENCIAS`; 200 sem dados é `SEM_PENDENCIAS` — sucesso, não
/// erro (P2, aguardando confirmação com a SERPRO real).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EstadoConsulta {
    ComPendencias,
    SemPendencias,
}

impl EstadoConsulta {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ComPendencias => "COM_PENDENCIAS",
            Self::SemPendencias => "SEM_PENDENCIAS",
        }
    }
}

impl fmt::Display for EstadoConsulta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Inscrição resumida, resultado do `GET {base}/devedor/{cpfCnpj}`
/// (quick-start SERPRO). Campos ausentes ficam `None` — resposta
/// tolerante, como o contrato exige.
#[derive(Debug, Clone)]
pub struct InscricaoResumo {
    pub numero_inscricao: String,
    pub numero_processo: Option<String>,
    pub situacao_inscricao: Option<String>,
    pub situacao_descricao: String,
    pub nome_devedor: Option<String>,
    pub tipo_devedor: String,
    pub valor_total_consolidado: Decimal,
    pub cpf_cnpj: Option<String>,
    pub nome_unidade: Option<String>,
}

/// Resultado agregado da consulta de devedor. Sempre carrega ambiente
/// do snapshot, estado, fonte, máscara da chave, horário e nº de
/// tentativas — o `client` não reconstrói nada disso.
#[derive(Debug, Clone)]
pub struct ResultadoConsultaDevedor {
    pub ambiente: AmbienteSerpro,
    pub estado: EstadoConsulta,
    pub fonte: FonteConsulta,
    pub chave_mascarada: String,
    pub horario_consulta: DateTime<Utc>,
    pub tentativas: u32,
    pub inscricoes: Vec<InscricaoResumo>,
}

/// Detalhe de uma inscrição, resultado do
/// `GET {base}/inscricao/{numero}` (quick-start SERPRO). Valores em
/// `Decimal` — `f32`/`f64` são proibidos.
#[derive(Debug, Clone)]
pub struct InscricaoDetalhe {
    pub ambiente: AmbienteSerpro,
    pub estado: EstadoConsulta,
    pub fonte: FonteConsulta,
    pub chave_mascarada: String,
    pub horario_consulta: DateTime<Utc>,
    pub tentativas: u32,
    pub numero_inscricao: String,
    pub processo_administrativo: Option<String>,
    pub situacao: String,
    pub data_inscricao: Option<String>,
    pub orgao_origem: Option<String>,
    pub codigo_natureza_receita: Option<String>,
    pub nome_receita: Option<String>,
    pub valor_total_inscrito: Option<Decimal>,
    pub valor_total_consolidado: Option<Decimal>,
    pub valor_remanescente: Option<Decimal>,
    pub indicador_prescricao: Option<bool>,
    pub nome_unidade: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ComandoConsultaDevedor {
    pub escritorio_id: Uuid,
    pub usuario_id: Uuid,
    pub chave: CpfCnpj,
}

#[derive(Debug)]
pub struct ComandoConsultaInscricao {
    pub escritorio_id: Uuid,
    pub usuario_id: Uuid,
    pub numero_inscricao: String,
}

impl ComandoConsultaInscricao {
    /// Números de inscrição são dígitos em bloco: apenas 6 a 20 dígitos
    /// são aceitos (ex.: `9061700096419`). Espaços do PDF são tolerados.
    pub fn novo(
        escritorio_id: Uuid,
        usuario_id: Uuid,
        valor: impl AsRef<str>,
    ) -> Result<Self, ErroDividaAtiva> {
        let digitos: String = valor
            .as_ref()
            .chars()
            .filter(|caractere| !caractere.is_whitespace())
            .collect();
        if digitos.is_empty()
            || digitos.len() < 6
            || digitos.len() > 20
            || !digitos.bytes().all(|byte| byte.is_ascii_digit())
        {
            return Err(ErroDividaAtiva::ChaveInvalida);
        }
        Ok(Self {
            escritorio_id,
            usuario_id,
            numero_inscricao: digitos,
        })
    }
}

/// Linha de auditoria de `consultas_fiscais` (SPEC §9.11). O documento
/// nunca entra em claro: só hash com pepper e máscara.
#[derive(Debug, Clone)]
pub struct RegistroConsultaFiscal {
    pub escritorio_id: Uuid,
    pub cliente_id: Option<Uuid>,
    pub produto: String,
    pub servico: String,
    pub verbo: String,
    pub ambiente: String,
    pub chave_consulta_hash: Option<String>,
    pub chave_consulta_mascarada: Option<String>,
    pub fonte_consulta: Option<String>,
    pub status_code: Option<i32>,
    pub estado: String,
    pub protocolo: Option<String>,
    pub request_redigido: Option<serde_json::Value>,
    pub response_redigido: Option<serde_json::Value>,
}

/// Parâmetros que a imagem define independentemente do documento.
#[derive(Debug, Clone)]
pub struct ContextoConsulta {
    pub escritorio_id: Uuid,
    pub ambiente: AmbienteSerpro,
}

/// Máscara exibível da chave de consulta: CPF guarda `***.***.890-**`
/// e CNPJ `**.***.***/0001-**` (400% seguro). Devolve `"?" ` quando nada
/// é reconhecivo, sem nunca revelar mais.
pub fn mascarar_documento(digitos: &str) -> String {
    match digitos.len() {
        11 => format!("***.***.{}-**", &digitos[6..9]),
        14 => format!("**.***.***/{}-**", &digitos[8..12]),
        _ => String::from("?"),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("valor monetário inválido: {0}")]
pub struct ValorMoedaInvalido(String);

/// Interpreta valores monetários pt-BR de forma **estrita**: `"2.403,59"` e
/// `"245,06"` são aceitos; `"2.403,59"` com o milhar fora da convenção
/// pt-BR (grupos de 3) é rejeitado, bem como o formato en-US `"2,403.56"`.
pub fn parse_moeda_ptbr_estrito(valor: &str) -> Result<Decimal, ValorMoedaInvalido> {
    let texto = valor.trim();
    if texto.is_empty() {
        return Err(ValorMoedaInvalido(valor.to_owned()));
    }

    let normalizado = if texto.contains(',') {
        // pt-BR: `.` é milhar (grupos de 3) e `,` é decimal.
        let partes: Vec<&str> = texto.split(',').collect();
        let Some((inteira, decimais)) = partes.split_first() else {
            return Err(ValorMoedaInvalido(valor.to_owned()));
        };
        if decimais.len() != 1 {
            return Err(ValorMoedaInvalido(valor.to_owned()));
        }
        let milhares = inteira.replace('.', "");
        if inteira.contains('.') {
            let grupos: Vec<&str> = inteira.split('.').collect();
            let valida = grupos[0].len() <= 3
                && !grupos[0].is_empty()
                && grupos[1..].iter().all(|grupo| grupo.len() == 3);
            if !valida {
                return Err(ValorMoedaInvalido(valor.to_owned()));
            }
        }
        if !decimais[0].bytes().all(|byte| byte.is_ascii_digit())
            || !milhares.bytes().all(|byte| byte.is_ascii_digit())
        {
            return Err(ValorMoedaInvalido(valor.to_owned()));
        }
        if decimais[0].is_empty() {
            return Err(ValorMoedaInvalido(valor.to_owned()));
        }
        format!("{milhares}.{}", decimais[0])
    } else {
        // Sem vírgula: inteiro, com ou sem milhares de 3 em 3.
        if texto.contains('.') {
            let grupos: Vec<&str> = texto.split('.').collect();
            let valida = grupos.len() > 1
                && grupos[0].len() <= 3
                && !grupos[0].is_empty()
                && grupos[1..].iter().all(|grupo| grupo.len() == 3);
            if !valida {
                return Err(ValorMoedaInvalido(valor.to_owned()));
            }
        }
        if !texto
            .bytes()
            .all(|byte| byte.is_ascii_digit() || byte == b'.')
        {
            return Err(ValorMoedaInvalido(valor.to_owned()));
        }
        texto.replace('.', "")
    };

    Decimal::from_str(&normalizado).map_err(|_| ValorMoedaInvalido(valor.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_moeda_ptbr_com_milhar_e_centavos() {
        let valor = parse_moeda_ptbr_estrito("2.403,59").expect("válido");
        assert_eq!(valor, Decimal::new(240_359, 2));
    }

    #[test]
    fn parse_moeda_ptbr_sem_milhar() {
        let valor = parse_moeda_ptbr_estrito("805,06").expect("válido");
        assert_eq!(valor, Decimal::new(80_506, 2));
    }

    #[test]
    fn parse_moeda_inteira_com_milhar() {
        let valor = parse_moeda_ptbr_estrito("1.200").expect("válido");
        assert_eq!(valor, Decimal::new(1200, 0));
    }

    #[test]
    fn rejeita_formato_en_us() {
        assert!(parse_moeda_ptbr_estrito("2,403.59").is_err());
    }

    #[test]
    fn rejeita_grupos_errados() {
        assert!(parse_moeda_ptbr_estrito("24.03,59").is_err());
        assert!(parse_moeda_ptbr_estrito("403.59.05").is_err());
    }

    #[test]
    fn rejeita_vazio_e_letras() {
        assert!(parse_moeda_ptbr_estrito("").is_err());
        assert!(parse_moeda_ptbr_estrito("R$ 2.403,59").is_err());
        assert!(parse_moeda_ptbr_estrito("805,06,").is_err());
    }

    #[test]
    fn mascarar_cpf_cnpj() {
        assert_eq!(mascarar_documento("09781911768"), "***.***.117-**");
        assert_eq!(mascarar_documento("11222333000181"), "**.***.***/0001-**");
        assert_eq!(mascarar_documento("123"), "?");
    }

    #[test]
    fn comando_inscricao_aceita_espacos_e_rejeita_basico() {
        let comando =
            ComandoConsultaInscricao::novo(Uuid::now_v7(), Uuid::now_v7(), "9061700096419")
                .expect("válido");
        assert_eq!(comando.numero_inscricao, "9061700096419");

        assert!(ComandoConsultaInscricao::novo(Uuid::now_v7(), Uuid::now_v7(), "abc").is_err());
        assert!(ComandoConsultaInscricao::novo(Uuid::now_v7(), Uuid::now_v7(), "12").is_err());
    }

    #[test]
    fn fonte_consulta_vai_em_str_e_volta() {
        assert_eq!(FonteConsulta::CpfCnpj.as_str(), "CPF_CNPJ");
        assert_eq!(
            FonteConsulta::from_str("CODIGO_CONTRIBUINTE"),
            Ok(FonteConsulta::CodigoContribuinte)
        );
        assert!(FonteConsulta::from_str("outro").is_err());
    }

    #[test]
    fn estado_consulta_str() {
        assert_eq!(EstadoConsulta::ComPendencias.as_str(), "COM_PENDENCIAS");
        assert_eq!(EstadoConsulta::SemPendencias.as_str(), "SEM_PENDENCIAS");
    }

    #[test]
    fn decimal_compat() {
        let usada = Decimal::new(240_359, 2);
        assert_eq!(usada, Decimal::from_i128_with_scale(240359, 2));
    }
}
