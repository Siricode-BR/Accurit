//! DTOs de resposta e mapeamento de erros do uso da Dívida Ativa.
//! Sempre derivados dos VOs de domínio — nunca do JSON bruto da SERPRO.

use axum::http::StatusCode;
use chrono::{DateTime, SecondsFormat, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use uuid::Uuid;

use crate::domain::divida_ativa::{
    ErroDividaAtiva, EstadoConsulta, InscricaoDetalhe, InscricaoResumo, ResultadoConsultaDevedor,
};
use crate::presentation::error::ApiError;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InscricaoResumoDto {
    pub numero_inscricao: String,
    pub numero_processo: Option<String>,
    pub situacao_descricao: String,
    pub nome_devedor: Option<String>,
    pub tipo_devedor: String,
    pub valor_total_consolidado: ValorDinheiro,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValorDinheiro {
    /// Valor decimal (nunca `f32`/`f64`): string pt-BR padronizada.
    pub valor: String,
    remarca: &'static str,
}

impl From<Decimal> for ValorDinheiro {
    fn from(valor: Decimal) -> Self {
        Self {
            valor: formatar_ptbr(valor),
            remarca: "BRL",
        }
    }
}

fn formatar_ptbr(valor: Decimal) -> String {
    let texto = valor.round_dp(2).to_string();
    let (inteira, decimal) = if let Some((inteira, decimal)) = texto.split_once('.') {
        (inteira.to_owned(), Some(decimal.to_owned()))
    } else {
        (texto, None)
    };

    let inteira_agrupada = {
        let digitos: Vec<char> = inteira.chars().collect();
        let mut agrupada = String::new();
        for (indice, caractere) in digitos.iter().enumerate() {
            if indice > 0 && (digitos.len() - indice).is_multiple_of(3) {
                agrupada.push('.');
            }
            agrupada.push(*caractere);
        }
        agrupada
    };

    let decimal_texto = match decimal {
        Some(decimal) => {
            let mut decimal_pad = decimal;
            if decimal_pad.len() == 1 {
                decimal_pad.push('0');
            }
            decimal_pad
        }
        None => "00".to_owned(),
    };

    format!("{inteira_agrupada},{decimal_texto}")
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RespostaDevedor {
    pub ambiente: &'static str,
    pub estado: &'static str,
    pub fonte_consulta: &'static str,
    pub chave_mascarada: String,
    pub horario_consulta: String,
    pub tentativas: u32,
    pub aviso: &'static str,
    pub inscricoes: Vec<InscricaoResumoDto>,
}

/// Aviso permanente: consulta informativa, sem ações transacionais.
pub const AVISO_CONSULTIVO: &str =
    "Consulta informativa. Não realiza negociação, parcelamento nem emissão de DARF.";

impl RespostaDevedor {
    pub fn de(consulta: &ResultadoConsultaDevedor) -> Self {
        Self {
            ambiente: consulta.ambiente.as_str(),
            estado: consulta.estado.como_str(),
            fonte_consulta: consulta.fonte.as_str(),
            chave_mascarada: consulta.chave_mascarada.clone(),
            horario_consulta: horario(&consulta.horario_consulta),
            tentativas: consulta.tentativas,
            aviso: AVISO_CONSULTIVO,
            inscricoes: consulta
                .inscricoes
                .iter()
                .map(inscricao_resumo_dto)
                .collect(),
        }
    }
}

fn horario(data_horario: &DateTime<Utc>) -> String {
    data_horario.to_rfc3339_opts(SecondsFormat::Secs, true)
}

fn inscricao_resumo_dto(inscricao: &InscricaoResumo) -> InscricaoResumoDto {
    InscricaoResumoDto {
        numero_inscricao: inscricao.numero_inscricao.clone(),
        numero_processo: inscricao.numero_processo.clone(),
        situacao_descricao: inscricao.situacao_descricao.clone(),
        nome_devedor: inscricao.nome_devedor.clone(),
        tipo_devedor: inscricao.tipo_devedor.clone(),
        valor_total_consolidado: inscricao.valor_total_consolidado.into(),
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RespostaInscricao {
    pub ambiente: &'static str,
    pub estado: &'static str,
    pub fonte_consulta: Option<&'static str>,
    pub chave_mascarada: String,
    pub horario_consulta: String,
    pub tentativas: u32,
    pub aviso: &'static str,
    pub numero_inscricao: String,
    pub processo_administrativo: Option<String>,
    pub situacao: String,
    pub data_inscricao: Option<String>,
    pub orgao_origem: Option<String>,
    pub codigo_natureza_receita: Option<String>,
    pub nome_receita: Option<String>,
    pub valor_total_inscrito: Option<ValorDinheiro>,
    pub valor_total_consolidado: Option<ValorDinheiro>,
    pub valor_remanescente: Option<ValorDinheiro>,
    pub indicador_prescricao: Option<bool>,
    pub nome_unidade: Option<String>,
}

impl EstadoConsulta {
    pub const fn como_str(self) -> &'static str {
        self.as_str()
    }
}

impl RespostaInscricao {
    pub fn de(detalhe: &InscricaoDetalhe) -> Self {
        Self {
            ambiente: detalhe.ambiente.as_str(),
            estado: detalhe.estado.como_str(),
            fonte_consulta: None,
            chave_mascarada: detalhe.chave_mascarada.clone(),
            horario_consulta: horario(&detalhe.horario_consulta),
            tentativas: detalhe.tentativas,
            aviso: AVISO_CONSULTIVO,
            numero_inscricao: detalhe.numero_inscricao.clone(),
            processo_administrativo: detalhe.processo_administrativo.clone(),
            situacao: detalhe.situacao.clone(),
            data_inscricao: detalhe.data_inscricao.clone(),
            orgao_origem: detalhe.orgao_origem.clone(),
            codigo_natureza_receita: detalhe.codigo_natureza_receita.clone(),
            nome_receita: detalhe.nome_receita.clone(),
            valor_total_inscrito: detalhe.valor_total_inscrito.map(ValorDinheiro::from),
            valor_total_consolidado: detalhe.valor_total_consolidado.map(ValorDinheiro::from),
            valor_remanescente: detalhe.valor_remanescente.map(ValorDinheiro::from),
            indicador_prescricao: detalhe.indicador_prescricao,
            nome_unidade: detalhe.nome_unidade.clone(),
        }
    }
}

/// Histórico paginado (`GET /divida-ativa/historico`).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EntradaHistoricoDto {
    pub id: Uuid,
    pub ambiente: String,
    pub servico: String,
    pub estado: String,
    pub fonte_consulta: Option<String>,
    pub chave_mascarada: Option<String>,
    pub status_code: Option<i32>,
    pub criado_em: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RespostaHistorico {
    pub pagina: u32,
    pub itens_pagina: u32,
    pub itens: Vec<EntradaHistoricoDto>,
}

/// Mapeamento erro de domínio → envelope estável (SPEC §10.3).
pub fn mapear_erro_divida(erro: ErroDividaAtiva) -> ApiError {
    match erro {
        ErroDividaAtiva::ChaveInvalida => {
            ApiError::validacao("CPF/CNPJ ou número de inscrição inválido")
        }
        ErroDividaAtiva::LimiteLocalAtingido => ApiError::muitas_requisicoes().com_retry_after("1"),
        ErroDividaAtiva::LimiteSerproAtingido => ApiError::nova(
            StatusCode::TOO_MANY_REQUESTS,
            "SERPRO_RATE_LIMITED",
            "Consultas em excesso na SERPRO; tente novamente mais tarde",
        ),
        ErroDividaAtiva::TrialNaoConfigurado => ApiError::nova(
            StatusCode::SERVICE_UNAVAILABLE,
            "SERPRO_TRIAL_NOT_CONFIGURED",
            "Ambiente trial da SERPRO não configurado para este escritório",
        ),
        ErroDividaAtiva::ProducaoNaoConfigurada => ApiError::nova(
            StatusCode::SERVICE_UNAVAILABLE,
            "SERPRO_PROD_NOT_CONFIGURED",
            "Produção da Dívida Ativa indisponível até decisão da SERPRO (D5)",
        ),
        ErroDividaAtiva::NaoAutorizado => ApiError::nova(
            StatusCode::BAD_GATEWAY,
            "SERPRO_UNAUTHORIZED",
            "Autenticação na SERPRO falhou",
        ),
        ErroDividaAtiva::ProxyNegado => ApiError::nova(
            StatusCode::BAD_GATEWAY,
            "SERPRO_PROXY_DENIED",
            "Acesso negado pelo proxy SERPRO",
        ),
        ErroDividaAtiva::ServicoIndisponivel => ApiError::nova(
            StatusCode::SERVICE_UNAVAILABLE,
            "SERPRO_UNAVAILABLE",
            "Serviço SERPRO indisponível; tente novamente em instantes",
        ),
        ErroDividaAtiva::Indeterminado => ApiError::nova(
            StatusCode::GATEWAY_TIMEOUT,
            "SERPRO_INDETERMINATE",
            "A SERPRO não confirmou o resultado da consulta; não repita automaticamente",
        ),
        ErroDividaAtiva::RespostaInvalida | ErroDividaAtiva::RespostaInvalidaDetalhada(_) => {
            ApiError::nova(
                StatusCode::BAD_GATEWAY,
                "SERPRO_BAD_RESPONSE",
                "Resposta da SERPRO fora do contrato",
            )
        }
        ErroDividaAtiva::NaoEncontrado => ApiError::nova(
            StatusCode::NOT_FOUND,
            "SERPRO_NOT_FOUND",
            "Pendência não encontrada na Dívida Ativa",
        ),
        ErroDividaAtiva::ErroInterno(_) => ApiError::erro_interno(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formata_valores_ptbr() {
        assert_eq!(formatar_ptbr(Decimal::new(240_359, 2)), "2.403,59");
        assert_eq!(formatar_ptbr(Decimal::new(80_506, 2)), "805,06");
        assert_eq!(formatar_ptbr(Decimal::new(1200, 0)), "1.200,00");
        assert_eq!(formatar_ptbr(Decimal::new(5, 0)), "5,00");
    }

    #[test]
    fn horario_rfc3339() {
        let agora = DateTime::<Utc>::from_timestamp(0, 0).expect("época");
        assert_eq!(horario(&agora), "1970-01-01T00:00:00Z");
    }
}
