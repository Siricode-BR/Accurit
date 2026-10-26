//! Transporte HTTP da consulta de Dívida Ativa (SPEC §3.3/§4.4):
//! `GET {base}/devedor/{chave}` ou `GET {base}/inscricao/{chave}`,
//! headers `Accept`, `X-Request-Id` e `x-cpf-usuario` (quando
//! habilitado por configuração), timeouts da configuração e o
//! mapeamento de status → estados (401 → renovação única OAuth2,
//! 404 → NAO_ENCONTRADO, 429 → LIMITE_EXCEDIDO com Retry-After,
//! 5xx → INDISPONIVEL, 504 → INDETERMINADO).

use std::sync::Arc;

use async_trait::async_trait;
use reqwest::StatusCode;
use secrecy::ExposeSecret;
use serde_json::Value;

use crate::application::divida_ativa::ports::{
    ConsultaDividaAtivaHttp, ErroHttpSerpro, RespostaHttpSerpro, ServicoDividaSerpro,
    SolicitacaoSerproHttp,
};
use crate::domain::divida_ativa::erros::{ErroConsultaDivida, ErroSerpro};
use crate::domain::serpro::endpoints::{EndpointSerpro, EstrategiaAutenticacaoSerpro};
use crate::infrastructure::divida_ativa::tokens::FornecedorDeTokens;

const ROTA_DEVEDOR: &str = "/devedor/";
const ROTA_INSCRICAO: &str = "/inscricao/";
const CABECALHO_OPERADOR: &str = "x-cpf-usuario";

#[derive(Clone)]
pub struct ClienteDividaAtivaHttp {
    http: reqwest::Client,
    tokens: Arc<dyn FornecedorDeTokens>,
}

impl ClienteDividaAtivaHttp {
    /// Timeouts vêm da configuração
    /// (`SERPRO_HTTP_CONNECT_TIMEOUT_SECS`/`SERPRO_HTTP_REQUEST_TIMEOUT_SECS`).
    pub fn novo(http: reqwest::Client, tokens: Arc<dyn FornecedorDeTokens>) -> Self {
        Self { http, tokens }
    }

    /// URL final: base configurada já inclui o prefixo `/api/v1`
    /// (quick-start); aqui só acrescentamos a rota e a chave
    /// (somente dígitos, sem escapamento necessário).
    fn url_da_consulta(
        endpoint: &EndpointSerpro,
        servico: ServicoDividaSerpro,
        chave: &str,
    ) -> reqwest::Url {
        let mut url: reqwest::Url = endpoint.url_base.clone();
        let base = url.path().trim_end_matches('/').to_owned();
        let rota = match servico {
            ServicoDividaSerpro::Devedor => ROTA_DEVEDOR,
            ServicoDividaSerpro::Inscricao => ROTA_INSCRICAO,
        };
        url.set_path(&format!("{base}{rota}{chave}"));
        url
    }

    async fn token_da_estrategia(&self, endpoint: &EndpointSerpro) -> Result<String, ErroSerpro> {
        match &endpoint.autenticacao {
            EstrategiaAutenticacaoSerpro::BearerFixo(token) => Ok(token.expose_secret().to_owned()),
            EstrategiaAutenticacaoSerpro::Oauth2Producao {
                token_url,
                consumer_key,
                consumer_secret,
            } => self
                .tokens
                .token_vigente(
                    token_url,
                    consumer_key.expose_secret().as_bytes(),
                    consumer_secret.expose_secret().as_bytes(),
                )
                .await
                .map(|token| token.expose_secret().to_owned()),
        }
    }

    /// Nova concessão exclusiva da renovação em 401: sem cache
    /// intermediário e sem segunda tentativa.
    async fn renovar_token(&self, endpoint: &EndpointSerpro) -> Result<String, ErroSerpro> {
        match &endpoint.autenticacao {
            EstrategiaAutenticacaoSerpro::BearerFixo(_) => Err(ErroSerpro::NaoAutorizado),
            EstrategiaAutenticacaoSerpro::Oauth2Producao {
                token_url,
                consumer_key,
                consumer_secret,
            } => self
                .tokens
                .conceder_novo_token(
                    token_url,
                    consumer_key.expose_secret().as_bytes(),
                    consumer_secret.expose_secret().as_bytes(),
                )
                .await
                .map(|token| token.expose_secret().to_owned()),
        }
    }

    fn abrir_requisicao(
        &self,
        url: &reqwest::Url,
        token: &str,
        solicitacao: &SolicitacaoSerproHttp,
    ) -> reqwest::RequestBuilder {
        let mut builder = self
            .http
            .get(url.clone())
            .header(reqwest::header::ACCEPT, "application/json")
            .header(reqwest::header::AUTHORIZATION, format!("Bearer {token}"))
            .header("x-request-id", solicitacao.request_id.to_string());
        if let Some(cpf) = &solicitacao.cpf_usuario {
            builder = builder.header(CABECALHO_OPERADOR, cpf);
        }
        builder
    }

    async fn mapear_resposta(resposta: reqwest::Response) -> Result<Value, ErroHttpSerpro> {
        let status = resposta.status();
        let retry_after = resposta
            .headers()
            .get(reqwest::header::RETRY_AFTER)
            .and_then(|valor| valor.to_str().ok())
            .and_then(|valor| valor.trim().parse::<u64>().ok());

        if status == StatusCode::OK {
            return match resposta.json::<Value>().await {
                Ok(corpo) => Ok(corpo),
                Err(_) => Err(resposta_invalida()),
            };
        }

        let erro = match status.as_u16() {
            404 => ErroSerpro::NaoEncontrado,
            401 | 403 => ErroSerpro::NaoAutorizado,
            429 => ErroSerpro::LimiteExcedido { retry_after },
            504 => ErroSerpro::Indeterminado,
            500..=599 => ErroSerpro::Indisponivel(status.as_u16()),
            _ => ErroSerpro::Indeterminado,
        };
        Err(ErroHttpSerpro {
            erro: ErroConsultaDivida::Serpro(erro),
            status_upstream: Some(i32::from(status.as_u16())),
        })
    }
}

fn resposta_invalida() -> ErroHttpSerpro {
    ErroHttpSerpro {
        erro: ErroConsultaDivida::RespostaInvalida("corpo não é JSON do contrato".to_owned()),
        status_upstream: Some(200),
    }
}

fn erro_de_transporte() -> ErroHttpSerpro {
    ErroHttpSerpro {
        erro: ErroConsultaDivida::Serpro(ErroSerpro::TimeoutOuRede),
        status_upstream: None,
    }
}

fn erro_serpro(erro: ErroSerpro, status: Option<i32>) -> ErroHttpSerpro {
    ErroHttpSerpro {
        erro: ErroConsultaDivida::Serpro(erro),
        status_upstream: status,
    }
}

#[async_trait]
impl ConsultaDividaAtivaHttp for ClienteDividaAtivaHttp {
    async fn consultar(&self, solicitacao: SolicitacaoSerproHttp) -> RespostaHttpSerpro {
        let url = Self::url_da_consulta(
            &solicitacao.endpoint,
            solicitacao.servico,
            &solicitacao.chave_normalizada,
        );

        let mut token = match self.token_da_estrategia(&solicitacao.endpoint).await {
            Ok(token) => token,
            Err(erro) => return Err(erro_serpro(erro, None)),
        };

        // Tentativa + UMA renovação em 401 (SPEC §3.1.1); renovação
        // só existe na estratégia OAuth2 de produção.
        for tentativa in 0..2u8 {
            match self
                .abrir_requisicao(&url, &token, &solicitacao)
                .send()
                .await
            {
                Ok(resposta) => {
                    if resposta.status() == StatusCode::UNAUTHORIZED && tentativa == 0 {
                        match self.renovar_token(&solicitacao.endpoint).await {
                            Ok(token_novo) => token = token_novo,
                            Err(erro) => {
                                return Err(erro_serpro(erro, Some(401)));
                            }
                        }
                        continue;
                    }
                    return Self::mapear_resposta(resposta).await;
                }
                Err(erro_reqwest) => {
                    let mensagem = erro_reqwest.to_string();
                    tracing::warn!(
                        request_id = solicitacao.request_id.to_string(),
                        "falha de envio ao gateway SERPRO: {mensagem}"
                    );
                    return Err(erro_de_transporte());
                }
            }
        }
        Err(erro_de_transporte())
    }
}
