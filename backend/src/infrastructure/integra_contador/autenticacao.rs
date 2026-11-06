use std::sync::Arc;

use async_trait::async_trait;
use base64::Engine as _;
use chrono::{NaiveDateTime, Utc};
use secrecy::ExposeSecret;
use url::Url;

use crate::application::integra_contador::ports::{
    AutenticaProcuradorCliente, AutenticarProcuradorResultado,
};
use crate::domain::integra_contador::{
    envelope::IntegraContadorRequest,
    erros::ErroIntegraContador,
};
use crate::domain::serpro::ambiente::AmbienteSerpro;
use crate::domain::serpro::endpoints::{EndpointSerpro, EstrategiaAutenticacaoSerpro};
use crate::domain::serpro::ports::ResolverEndpointsSerpro;
use crate::domain::serpro::produto::ProdutoSerpro;

/// Implementação da porta `AutenticaProcuradorCliente`: envia o XML do
/// Termo de Autorização (base64) ao endpoint `/Apoiar` do Integra Contador
/// (`AUTENTICAPROCURADOR` / `ENVIOXMLASSINADO81`) e interpreta o token
/// `autenticar_procurador_token` retornado pelo SERPRO.
pub struct AutenticaProcuradorClienteImpl {
    http_client: reqwest::Client,
    endpoints: Arc<dyn ResolverEndpointsSerpro>,
}

impl AutenticaProcuradorClienteImpl {
    pub fn novo(http_client: reqwest::Client, endpoints: Arc<dyn ResolverEndpointsSerpro>) -> Self {
        Self {
            http_client,
            endpoints,
        }
    }

    fn bearer(&self, endpoint: &EndpointSerpro) -> Result<String, ErroIntegraContador> {
        match &endpoint.autenticacao {
            // O contratante da API (Software House) detém o Bearer Token
            // principal; o token por procurador vem depois do /Apoiar.
            EstrategiaAutenticacaoSerpro::BearerFixo(token) => {
                Ok(format!("Bearer {}", token.expose_secret()))
            }
            EstrategiaAutenticacaoSerpro::Oauth2Producao { .. } => {
                Err(ErroIntegraContador::ProducaoBloqueada)
            }
        }
    }

    fn url_apoiar(&self, endpoint: &EndpointSerpro) -> Result<Url, ErroIntegraContador> {
        let caminho = format!(
            "{}/Apoiar",
            endpoint.url_base.as_str().trim_end_matches('/')
        );
        Url::parse(&caminho).map_err(|_| ErroIntegraContador::ConfiguracaoIncompleta)
    }
}

/// Objeto `Autentica` retornado dentro de `dados` (string JSON escapada).
#[derive(serde::Deserialize)]
struct AutenticaDados {
    autenticar_procurador_token: String,
    data_hora_expiracao: Option<String>,
}

/// O SERPRO devolve `data_hora_expiracao` no fuso de Brasília
/// (America/Sao_Paulo, UTC-3); converte para UTC corretamente.
fn expiracao_utc(valor: &str) -> chrono::DateTime<Utc> {
    if let Ok(naive) = NaiveDateTime::parse_from_str(&valor.replace(' ', "T"), "%Y-%m-%dT%H:%M:%S") {
        // Brasília é UTC-3: o horário local equivale ao UTC somado de 3 h.
        let utc = naive + chrono::TimeDelta::try_hours(3).expect("3 horas cabe em TimeDelta");
        return chrono::DateTime::<Utc>::from_naive_utc_and_offset(utc, Utc);
    }
    // Formato inesperado: token válido por até 24 h (meia-noite de Brasília
    // do dia seguinte); expira em 24 h como cota superior conservadora.
    Utc::now() + chrono::Duration::hours(24)
}

#[async_trait]
impl AutenticaProcuradorCliente for AutenticaProcuradorClienteImpl {
    async fn enviar_xml_assinado(
        &self,
        request: &IntegraContadorRequest,
        xml_base64: &str,
        ambiente: AmbienteSerpro,
        _request_id: &str,
    ) -> Result<AutenticarProcuradorResultado, ErroIntegraContador> {
        if ambiente == AmbienteSerpro::Producao {
            return Err(ErroIntegraContador::ProducaoBloqueada);
        }

        let endpoint = self
            .endpoints
            .resolver(ProdutoSerpro::IntegraContador, ambiente)
            .await
            .map_err(|_| ErroIntegraContador::ConfiguracaoIncompleta)?;
        let auth = self.bearer(&endpoint)?;
        let url = self.url_apoiar(&endpoint)?;

        // Valida o XML recebido: SEMPRE em base64 (string escapada dentro
        // de pedidoDados.dados), conforme contrato ENVIOXMLASSINADO81.
        if xml_base64.is_empty() {
            return Err(ErroIntegraContador::PayloadInvalido);
        }
        base64::engine::general_purpose::STANDARD
            .decode(xml_base64)
            .map_err(|_| ErroIntegraContador::PayloadInvalido)?;

        let mut requisicao = request.clone();
        requisicao.pedido_dados.id_sistema = "AUTENTICAPROCURADOR".to_owned();
        requisicao.pedido_dados.id_servico = "ENVIOXMLASSINADO81".to_owned();
        requisicao.pedido_dados.versao_sistema = "1.0".to_owned();
        requisicao.pedido_dados.dados = serde_json::to_string(&serde_json::json!({
            "xml": xml_base64
        }))
        .map_err(|_| ErroIntegraContador::PayloadInvalido)?;

        let resposta = self
            .http_client
            .post(url)
            .header(reqwest::header::ACCEPT, "application/json")
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .header(reqwest::header::AUTHORIZATION, auth)
            .json(&requisicao)
            .send()
            .await
            .map_err(|erro| {
                tracing::warn!(
                    timeout = erro.is_timeout(),
                    "falha de transporte na chamada AUTENTICAPROCURADOR"
                );
                ErroIntegraContador::Indisponivel
            })?;

        let status = resposta.status().as_u16();
        match status {
            200 => {}
            401 => return Err(ErroIntegraContador::NaoAutorizado),
            403 => return Err(ErroIntegraContador::AcessoNegado),
            429 => return Err(ErroIntegraContador::LimiteExcedido),
            400..=499 => return Err(ErroIntegraContador::ProcuracaoNegada),
            504 => return Err(ErroIntegraContador::Indeterminado),
            500..=599 => return Err(ErroIntegraContador::Indisponivel),
            _ => return Err(ErroIntegraContador::RespostaInvalida),
        }

        let envelope: crate::domain::integra_contador::envelope::IntegraContadorResponse =
            resposta
                .json()
                .await
                .map_err(|_| ErroIntegraContador::RespostaInvalida)?;

        // `dados` chega como string JSON escapada contendo o objeto Autentica.
        let dados_texto = envelope
            .dados
            .as_deref()
            .ok_or(ErroIntegraContador::RespostaInvalida)?;
        let autentica: AutenticaDados =
            serde_json::from_str(dados_texto).map_err(|_| ErroIntegraContador::RespostaInvalida)?;

        if autentica.autenticar_procurador_token.is_empty() {
            return Err(ErroIntegraContador::RespostaInvalida);
        }

        Ok(AutenticarProcuradorResultado {
            jwt_token: autentica.autenticar_procurador_token,
            expira_em: autentica
                .data_hora_expiracao
                .as_deref()
                .map(expiracao_utc)
                .unwrap_or_else(|| Utc::now() + chrono::Duration::hours(24)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exige_xml_base64_valido() {
        assert!(base64::engine::general_purpose::STANDARD
            .decode("aGVsbG8=")
            .is_ok());
        assert!(base64::engine::general_purpose::STANDARD.decode("!").is_err());
    }

    #[test]
    fn expiracao_resolve_para_24h_quando_formato_e_diferente() {
        let expira = expiracao_utc("nao-parseavel");
        assert!(expira > Utc::now());
        assert!(expira <= Utc::now() + chrono::Duration::minutes(24 * 60 + 5));
    }

    #[test]
    fn expiracao_parseia_formato_iso_local() {
        // Brasília 2026-11-07 23:59:59 => UTC 2026-11-08 02:59:59.
        let expira = expiracao_utc("2026-11-07 23:59:59");
        assert_eq!(&expira.to_rfc3339()[..16], "2026-11-08T02:59");
    }
}
