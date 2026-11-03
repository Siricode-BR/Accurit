use std::sync::Arc;

use async_trait::async_trait;
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, ETAG, EXPIRES, RETRY_AFTER};
use secrecy::ExposeSecret;
use url::Url;

use crate::application::integra_contador::ports::{ClienteIntegraContador, EntregaIntegraContador};
use crate::domain::integra_contador::envelope::{
    IntegraContadorRequest, IntegraContadorResponse, VerboIntegraContador,
};
use crate::domain::integra_contador::erros::ErroIntegraContador;
use crate::domain::serpro::ambiente::AmbienteSerpro;
use crate::domain::serpro::endpoints::{EndpointSerpro, EstrategiaAutenticacaoSerpro};
use crate::domain::serpro::ports::ResolverEndpointsSerpro;
use crate::domain::serpro::produto::ProdutoSerpro;

pub struct ClienteIntegraContadorImpl {
    http_client: reqwest::Client,
    endpoints: Arc<dyn ResolverEndpointsSerpro>,
    operator_header_enabled: bool,
}

impl ClienteIntegraContadorImpl {
    pub fn novo(
        http_client: reqwest::Client,
        endpoints: Arc<dyn ResolverEndpointsSerpro>,
        operator_header_enabled: bool,
    ) -> Self {
        Self {
            http_client,
            endpoints,
            operator_header_enabled,
        }
    }

    fn headers(
        &self,
        token: &str,
        cpf_operador: Option<&str>,
    ) -> Result<reqwest::header::HeaderMap, ErroIntegraContador> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            ACCEPT,
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&format!("Bearer {token}"))
                .map_err(|_| ErroIntegraContador::ConfiguracaoIncompleta)?,
        );

        if self.operator_header_enabled {
            let cpf = cpf_operador
                .filter(|cpf| cpf.len() == 11 && cpf.bytes().all(|byte| byte.is_ascii_digit()))
                .ok_or(ErroIntegraContador::ConfiguracaoIncompleta)?;
            headers.insert(
                "x-cpf-usuario",
                reqwest::header::HeaderValue::from_str(cpf)
                    .map_err(|_| ErroIntegraContador::ConfiguracaoIncompleta)?,
            );
        }
        Ok(headers)
    }
}

#[async_trait]
impl ClienteIntegraContador for ClienteIntegraContadorImpl {
    async fn executar(
        &self,
        verbo: VerboIntegraContador,
        request: &IntegraContadorRequest,
        ambiente: AmbienteSerpro,
        cpf_operador: Option<&str>,
        request_id: &str,
    ) -> Result<EntregaIntegraContador, ErroIntegraContador> {
        if ambiente == AmbienteSerpro::Producao {
            return Err(ErroIntegraContador::ProducaoBloqueada);
        }

        let endpoint = self
            .endpoints
            .resolver(ProdutoSerpro::IntegraContador, ambiente)
            .await
            .map_err(|_| ErroIntegraContador::ConfiguracaoIncompleta)?;
        let token = match &endpoint.autenticacao {
            EstrategiaAutenticacaoSerpro::BearerFixo(token) => token.expose_secret(),
            EstrategiaAutenticacaoSerpro::Oauth2Producao { .. } => {
                return Err(ErroIntegraContador::ProducaoBloqueada);
            }
        };
        let url = url_do_verbo(&endpoint, verbo)?;
        let headers = self.headers(token, cpf_operador)?;

        let resposta = self
            .http_client
            .post(url)
            .headers(headers)
            .json(request)
            .send()
            .await
            .map_err(|erro| {
                tracing::warn!(
                    request_id,
                    timeout = erro.is_timeout(),
                    "falha de transporte na chamada Integra Contador"
                );
                if erro.is_timeout() && verbo.exige_confirmacao() {
                    ErroIntegraContador::Indeterminado
                } else {
                    ErroIntegraContador::Indisponivel
                }
            })?;

        let status = resposta.status().as_u16();
        let retry_after = resposta
            .headers()
            .get(RETRY_AFTER)
            .and_then(|valor| valor.to_str().ok())
            .and_then(|valor| valor.parse::<u64>().ok());
        let etag = resposta
            .headers()
            .get(ETAG)
            .and_then(|valor| valor.to_str().ok())
            .map(str::to_owned);
        let expires = resposta
            .headers()
            .get(EXPIRES)
            .and_then(|valor| valor.to_str().ok())
            .map(str::to_owned);
        let referencia_resposta = resposta
            .headers()
            .get("x-request-id")
            .and_then(|valor| valor.to_str().ok())
            .map(str::to_owned);
        let corpo = resposta
            .bytes()
            .await
            .map_err(|_| ErroIntegraContador::RespostaInvalida)?;
        let envelope = interpretar_envelope(status, &corpo)?;

        Ok(EntregaIntegraContador {
            http_status: status,
            estado: estado_http(status),
            resposta: envelope,
            referencia_resposta,
            tempo_espera_segundos: retry_after,
            etag,
            expires,
        })
    }
}

fn url_do_verbo(
    endpoint: &EndpointSerpro,
    verbo: VerboIntegraContador,
) -> Result<Url, ErroIntegraContador> {
    let caminho = format!(
        "{}/{}",
        endpoint.url_base.as_str().trim_end_matches('/'),
        verbo.como_str()
    );
    Url::parse(&caminho).map_err(|_| ErroIntegraContador::ConfiguracaoIncompleta)
}

fn interpretar_envelope(
    status: u16,
    corpo: &[u8],
) -> Result<Option<IntegraContadorResponse>, ErroIntegraContador> {
    if matches!(status, 204 | 304) || corpo.is_empty() {
        return if matches!(status, 202 | 204 | 304) || !(200..300).contains(&status) {
            Ok(None)
        } else {
            Err(ErroIntegraContador::RespostaInvalida)
        };
    }

    match serde_json::from_slice(corpo) {
        Ok(resposta) => Ok(Some(resposta)),
        Err(_) if (200..300).contains(&status) => Err(ErroIntegraContador::RespostaInvalida),
        Err(_) => Ok(None),
    }
}

fn estado_http(status: u16) -> String {
    let estado = match status {
        202 => "PROCESSANDO",
        204 => "PROCESSANDO",
        304 => "NAO_MODIFICADO",
        401 | 403 => "FALHA_AUTORIZACAO",
        404 => "SERVICO_NAO_ENCONTRADO",
        429 => "LIMITE_EXCEDIDO",
        504 => "INDETERMINADO",
        400 => "REJEICAO_NEGOCIO",
        500..=599 => "FALHA_TRANSPORTE",
        200..=299 => "CONCLUIDO",
        _ => "FALHA_NEGOCIO",
    };
    estado.to_owned()
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use secrecy::SecretString;

    use super::*;

    fn cliente(operator_header_enabled: bool) -> ClienteIntegraContadorImpl {
        struct Endpoints;

        #[async_trait]
        impl ResolverEndpointsSerpro for Endpoints {
            async fn resolver(
                &self,
                _produto: ProdutoSerpro,
                _ambiente: AmbienteSerpro,
            ) -> Result<EndpointSerpro, crate::domain::serpro::erros::ErroEndpoint> {
                unreachable!("testa apenas os helpers locais")
            }
        }

        ClienteIntegraContadorImpl::novo(
            reqwest::Client::new(),
            Arc::new(Endpoints),
            operator_header_enabled,
        )
    }

    #[test]
    fn envia_cpf_operador_apenas_quando_habilitado() {
        let enabled = cliente(true);
        let headers = enabled
            .headers("trial-token", Some("52998224725"))
            .expect("headers válidos");
        assert_eq!(headers["x-cpf-usuario"], "52998224725");

        let disabled = cliente(false);
        let headers = disabled
            .headers("trial-token", None)
            .expect("headers válidos");
        assert!(!headers.contains_key("x-cpf-usuario"));
        assert_eq!(headers[AUTHORIZATION], "Bearer trial-token");
    }

    #[test]
    fn rejeita_cpf_de_operador_invalido_quando_header_esta_habilitado() {
        assert!(matches!(
            cliente(true).headers("trial-token", Some("123")),
            Err(ErroIntegraContador::ConfiguracaoIncompleta)
        ));
    }

    #[test]
    fn resposta_sem_corpo_preserva_status_assincrono() {
        assert_eq!(interpretar_envelope(204, b"").expect("204 válido"), None);
        assert_eq!(interpretar_envelope(304, b"").expect("304 válido"), None);
        assert_eq!(estado_http(504), "INDETERMINADO");
    }

    #[test]
    fn nao_interpreta_corpo_de_erro_como_envelope() {
        assert_eq!(
            interpretar_envelope(429, b"erro").expect("erro é opaco"),
            None
        );
        assert_eq!(
            interpretar_envelope(200, b"erro"),
            Err(ErroIntegraContador::RespostaInvalida)
        );
    }

    #[test]
    fn token_nao_e_expresso_em_debug_ou_logs() {
        let token = SecretString::from("segredo");
        assert!(!format!("{token:?}").contains("segredo"));
    }
}
