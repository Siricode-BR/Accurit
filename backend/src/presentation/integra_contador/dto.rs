use serde::{Deserialize, Serialize};

use crate::application::integra_contador::ResultadoIntegraContador;
use crate::domain::integra_contador::envelope::{PedidoDados, VerboIntegraContador};
use crate::domain::integra_contador::servicos::{EstadoCatalogo, ServicoIntegraContador};
use crate::domain::serpro::ambiente::AmbienteSerpro;
use crate::presentation::serpro::ambiente::AmbienteResolvido;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequisicaoExecucao {
    pub documento_contribuinte: String,
    pub verbo: VerboIntegraContador,
    pub pedido_dados: PedidoDados,
    #[serde(default)]
    pub confirmacao_explicita: bool,
    pub chave_idempotencia: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequisicaoConsultaProcuracoes {
    pub documento_contribuinte: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusIntegraContadorResponse {
    pub ambiente_efetivo: AmbienteSerpro,
    pub producao_habilitada: bool,
    pub assinatura_habilitada: bool,
    pub motivo_bloqueio_producao: &'static str,
    pub motivo_bloqueio_assinatura: &'static str,
    pub servicos: Vec<ServicoIntegraContadorResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServicoIntegraContadorResponse {
    pub id_sistema: &'static str,
    pub id_servico: &'static str,
    pub versao_sistema: Option<&'static str>,
    pub verbo: VerboIntegraContador,
    pub habilitado: bool,
    pub requer_confirmacao: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultadoIntegraContadorResponse {
    pub ambiente: AmbienteSerpro,
    pub verbo: VerboIntegraContador,
    pub id_sistema: String,
    pub id_servico: String,
    pub estado: String,
    pub status_serpro: u16,
    pub mensagem: &'static str,
    pub referencia_resposta: Option<String>,
    pub tempo_espera_segundos: Option<u64>,
    pub etag: Option<String>,
    pub expires: Option<String>,
    pub codigos_mensagem: Vec<String>,
}

impl StatusIntegraContadorResponse {
    pub fn novo(ambiente: AmbienteResolvido) -> Self {
        let servicos = crate::domain::integra_contador::servicos::catalogo_de_referencia()
            .into_iter()
            .map(ServicoIntegraContadorResponse::de)
            .collect();
        Self {
            ambiente_efetivo: ambiente.ambiente,
            producao_habilitada: false,
            assinatura_habilitada: false,
            motivo_bloqueio_producao: "Confirmação contratual SERPRO e aprovação D5 pendentes.",
            motivo_bloqueio_assinatura: "Validação de cadeia ICP-Brasil e LCR não está configurada.",
            servicos,
        }
    }
}

impl ServicoIntegraContadorResponse {
    fn de(servico: ServicoIntegraContador) -> Self {
        Self {
            id_sistema: servico.id_sistema,
            id_servico: servico.id_servico,
            versao_sistema: servico.versao_sistema,
            verbo: servico.verbo,
            habilitado: servico.estado == EstadoCatalogo::Habilitado,
            requer_confirmacao: servico.verbo.exige_confirmacao(),
        }
    }
}

impl From<ResultadoIntegraContador> for ResultadoIntegraContadorResponse {
    fn from(resultado: ResultadoIntegraContador) -> Self {
        Self {
            ambiente: resultado.ambiente,
            verbo: resultado.verbo,
            id_sistema: resultado.id_sistema,
            id_servico: resultado.id_servico,
            estado: resultado.estado,
            status_serpro: resultado.status,
            mensagem: resultado.mensagem,
            referencia_resposta: resultado.referencia_resposta,
            tempo_espera_segundos: resultado.tempo_espera_segundos,
            etag: resultado.etag,
            expires: resultado.expires,
            codigos_mensagem: resultado.codigos_mensagem,
        }
    }
}
