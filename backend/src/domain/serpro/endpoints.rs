use std::fmt;
use std::sync::Arc;

use thiserror::Error;

use secrecy::SecretString;
use url::Url;

use crate::domain::serpro::ambiente::AmbienteSerpro;
use crate::domain::serpro::produto::ProdutoSerpro;

#[derive(Debug, Clone)]
pub struct EndpointSerpro {
    pub produto: ProdutoSerpro,
    pub ambiente: AmbienteSerpro,
    pub url_base: Url,
    pub autenticacao: EstrategiaAutenticacaoSerpro,
}

impl fmt::Display for EndpointSerpro {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format!("{} ({})", self.produto, self.ambiente))
    }
}

#[derive(Debug, Clone, Error)]
pub enum EndpointError {
    #[error("URL de configuração ausente")]
    UrlAusente(String),
    #[error("token de trial ausente")]
    TokenTrialAusente,
    #[error("credencial de produção ausente")]
    CredencialProducaoAusente,
    #[error("URL não configurada")]
    UrlInvalida { variavel: String, motivo: String },
}

impl EndpointError {
    /// Código estável para desenvelope nas chamadas (fase 4 não deve mapear
    /// nomes de variáveis de ambiente para o cliente).
    pub const fn codigo(&self) -> &'static str {
        match self {
            Self::UrlAusente(_) => "SERPRO_ENDPOINT_INDEFINIDO",
            Self::TokenTrialAusente => "SERPRO_TRIAL_BEARER_TOKEN_AUSENTE",
            Self::CredencialProducaoAusente => "SERPRO_CONSUMER_KEY_MISSING",
            Self::UrlInvalida { .. } => "SERPRO_ENDPOINT_INVAL",
        }
    }
}

#[derive(Debug, Clone)]
pub enum EstrategiaAutenticacaoSerpro {
    /// Integra Contador trial: token bearer fixo da configuração
    /// (`SERPRO_TRIAL_BEARER_TOKEN`, token de demo da SERPRO).
    BearerFixo(SecretString),
    /// Integra Contador produção: consumer key/secret via fluxo OAuth2/SAPI.
    Oauth2Producao {
        token_url: Url,
        consumer_key: Arc<SecretString>,
        consumer_secret: Arc<SecretString>,
    },
    /// Dívida Ativa trial: não usa credencial OAuth — a identificação do
    /// contribuinte vai no corpo do request (decisão D2); fase 4 cuida disso.
    SemCredencial,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_endpoint_inclui_produto_e_ambiente() {
        let endpoint = EndpointSerpro {
            produto: ProdutoSerpro::IntegraContador,
            ambiente: AmbienteSerpro::Trial,
            url_base: Url::parse("https://exemplo.test/integra").unwrap(),
            autenticacao: EstrategiaAutenticacaoSerpro::BearerFixo(SecretString::from("demo")),
        };
        assert_eq!(endpoint.to_string(), "INTEGRA_CONTADOR (TRIAL)");
    }

    #[test]
    fn display_endpoint_com_ambiente_producao() {
        let endpoint = EndpointSerpro {
            produto: ProdutoSerpro::DividaAtiva,
            ambiente: AmbienteSerpro::Producao,
            url_base: Url::parse("https://exemplo.test/divida-prod").unwrap(),
            autenticacao: EstrategiaAutenticacaoSerpro::Oauth2Producao {
                token_url: Url::parse("https://exemplo.test/token").unwrap(),
                consumer_key: Arc::new(SecretString::from("ck")),
                consumer_secret: Arc::new(SecretString::from("cs")),
            },
        };
        assert_eq!(endpoint.to_string(), "DIVIDA_ATIVA (PRODUCAO)");
    }
}
