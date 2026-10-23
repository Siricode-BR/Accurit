use std::sync::Arc;

use async_trait::async_trait;
use secrecy::ExposeSecret;

use crate::domain::serpro::ambiente::AmbienteSerpro;
use crate::domain::serpro::endpoints::{EndpointSerpro, EstrategiaAutenticacaoSerpro};
use crate::domain::serpro::erros::ErroEndpoint;
use crate::domain::serpro::ports::ResolverEndpointsSerpro;
use crate::domain::serpro::produto::ProdutoSerpro;
use crate::infrastructure::config::SerproConfig;

/// Resolve endpoint por produto+ambiente a partir da configuração, com
/// erros explícitos — sem fallback silencioso entre ambientes (ADR-0007):
/// config ausente é erro, nunca "usar o outro ambiente e torcer".
#[derive(Debug, Clone)]
pub struct SerproEndpointResolverImpl {
    config: SerproConfig,
}

impl SerproEndpointResolverImpl {
    pub fn novo(config: SerproConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl ResolverEndpointsSerpro for SerproEndpointResolverImpl {
    async fn resolver(
        &self,
        produto: ProdutoSerpro,
        ambiente: AmbienteSerpro,
    ) -> Result<EndpointSerpro, ErroEndpoint> {
        match (produto, ambiente) {
            (ProdutoSerpro::IntegraContador, AmbienteSerpro::Trial) => {
                resolver_trial_integra(&self.config)
            }
            (ProdutoSerpro::IntegraContador, AmbienteSerpro::Producao) => {
                resolver_producao_integra(&self.config)
            }
            (ProdutoSerpro::DividaAtiva, AmbienteSerpro::Trial) => {
                resolver_trial_divida(&self.config)
            }
            (ProdutoSerpro::DividaAtiva, AmbienteSerpro::Producao) => {
                // Decisão D5: a URL de produção da Dívida Ativa não consta
                // referência/documentação: service fica implicitamente
                // bloqueado, mesmo que alguém configure a variável.
                Err(ErroEndpoint::DividaAtivaProducaoBloqueada)
            }
        }
    }
}

fn resolver_trial_integra(config: &SerproConfig) -> Result<EndpointSerpro, ErroEndpoint> {
    let token = config
        .trial_bearer_token
        .as_ref()
        .filter(|valor| !valor.expose_secret().trim().is_empty())
        .ok_or(ErroEndpoint::Indefinido(String::from(
            "SERPRO_TRIAL_BEARER_TOKEN não presente para Integra Contador trial",
        )))?;

    Ok(EndpointSerpro {
        produto: ProdutoSerpro::IntegraContador,
        ambiente: AmbienteSerpro::Trial,
        url_base: config.integra_trial_url.clone(),
        autenticacao: EstrategiaAutenticacaoSerpro::BearerFixo(token.clone()),
    })
}

fn resolver_trial_divida(config: &SerproConfig) -> Result<EndpointSerpro, ErroEndpoint> {
    Ok(EndpointSerpro {
        produto: ProdutoSerpro::DividaAtiva,
        ambiente: AmbienteSerpro::Trial,
        url_base: config.divida_ativa_trial_url.clone(),
        // A Dívida Ativa exige a identificação do contribuinte no
        // request (decisão D2), não credencial OAuth — a fase de
        // integração cuida do detalhe do corpo.
        autenticacao: EstrategiaAutenticacaoSerpro::SemCredencial,
    })
}

fn resolver_producao_integra(config: &SerproConfig) -> Result<EndpointSerpro, ErroEndpoint> {
    let consumer_key = config.consumer_key.as_ref();
    let consumer_secret = config.consumer_secret.as_ref();

    let (key, secret) = match (consumer_key, consumer_secret) {
        (Some(key), Some(secret)) => (key, secret),
        _ => {
            return Err(ErroEndpoint::Indefinido(String::from(
                "SERPRO_CONSUMER_KEY/SECRET ausentes para Integra Contador produção",
            )));
        }
    };

    Ok(EndpointSerpro {
        produto: ProdutoSerpro::IntegraContador,
        ambiente: AmbienteSerpro::Producao,
        url_base: config.integra_prod_url.clone(),
        autenticacao: EstrategiaAutenticacaoSerpro::Oauth2Producao {
            token_url: config.token_url.clone(),
            consumer_key: Arc::new(key.clone()),
            consumer_secret: Arc::new(secret.clone()),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::config::Config;
    use base64::Engine;
    use base64::engine::general_purpose::STANDARD;

    fn pares(extras: &[(&str, &str)]) -> Vec<(String, String)> {
        let mut pares: Vec<(String, String)> = vec![
            ("APP_ENV".into(), "development".into()),
            (
                "DATABASE_URL".into(),
                "postgres://app:senha@localhost:5432/accurit".into(),
            ),
            ("JWT_SECRET".into(), "x".repeat(32)),
            ("CERTIFICATE_MASTER_KEY".into(), STANDARD.encode([7u8; 32])),
            (
                "SERPRO_TOKEN_URL".into(),
                "https://serpro.test/token".into(),
            ),
            (
                "SERPRO_SAPI_AUTH_URL".into(),
                "https://serpro.test/auth".into(),
            ),
            (
                "SERPRO_INTEGRA_TRIAL_URL".into(),
                "https://serpro.test/integra".into(),
            ),
            (
                "SERPRO_INTEGRA_PROD_URL".into(),
                "https://serpro.test/integra-prod".into(),
            ),
            (
                "SERPRO_DIVIDA_ATIVA_TRIAL_URL".into(),
                "https://serpro.test/divida".into(),
            ),
        ];
        for (chave, valor) in extras {
            pares.push(((*chave).into(), (*valor).into()));
        }
        pares
    }

    #[tokio::test]
    async fn integra_trial_exige_bearer_configurado() {
        let config = Config::from_pairs(pares(&[])).expect("config válida");
        let resolver = SerproEndpointResolverImpl::novo(config.serpro);

        let erro = resolver
            .resolver(ProdutoSerpro::IntegraContador, AmbienteSerpro::Trial)
            .await
            .unwrap_err();

        assert!(matches!(erro, ErroEndpoint::Indefinido(_)));
    }

    #[tokio::test]
    async fn integra_trial_resolve_com_token() {
        let config = Config::from_pairs(pares(&[
            ("SERPRO_TRIAL_BEARER_TOKEN", "Bearer demo-token"),
            ("SERPRO_CONSUMER_KEY", "ck"),
            ("SERPRO_CONSUMER_SECRET", "cs"),
        ]))
        .expect("config válida");
        let resolver = SerproEndpointResolverImpl::novo(config.serpro);

        let endpoint = resolver
            .resolver(ProdutoSerpro::IntegraContador, AmbienteSerpro::Trial)
            .await
            .unwrap();

        assert_eq!(endpoint.url_base.as_str(), "https://serpro.test/integra");
        match endpoint.autenticacao {
            EstrategiaAutenticacaoSerpro::BearerFixo(token) => {
                assert_eq!(token.expose_secret(), "Bearer demo-token");
            }
            _ => panic!("esperava BearerFixo no trial"),
        }
    }

    #[tokio::test]
    async fn divida_trial_nao_exige_credencial() {
        let config = Config::from_pairs(pares(&[])).expect("config válida");
        let resolver = SerproEndpointResolverImpl::novo(config.serpro);

        let endpoint = resolver
            .resolver(ProdutoSerpro::DividaAtiva, AmbienteSerpro::Trial)
            .await
            .unwrap();

        assert_eq!(endpoint.url_base.as_str(), "https://serpro.test/divida");
        assert!(matches!(
            endpoint.autenticacao,
            EstrategiaAutenticacaoSerpro::SemCredencial
        ));
    }

    #[tokio::test]
    async fn integra_producao_exige_consumer_configurado() {
        let config = Config::from_pairs(pares(&[("SERPRO_TRIAL_BEARER_TOKEN", "Bearer demo")]))
            .expect("config válida");
        let resolver = SerproEndpointResolverImpl::novo(config.serpro);

        let erro = resolver
            .resolver(ProdutoSerpro::IntegraContador, AmbienteSerpro::Producao)
            .await
            .unwrap_err();

        assert!(matches!(erro, ErroEndpoint::Indefinido(_)));
    }

    #[tokio::test]
    async fn integra_producao_resolve_com_consumer() {
        let config = Config::from_pairs(pares(&[
            ("SERPRO_TRIAL_BEARER_TOKEN", "Bearer demo"),
            ("SERPRO_CONSUMER_KEY", "ck-prod"),
            ("SERPRO_CONSUMER_SECRET", "cs-prod"),
        ]))
        .expect("config válida");
        let resolver = SerproEndpointResolverImpl::novo(config.serpro);

        let endpoint = resolver
            .resolver(ProdutoSerpro::IntegraContador, AmbienteSerpro::Producao)
            .await
            .unwrap();

        assert_eq!(
            endpoint.url_base.as_str(),
            "https://serpro.test/integra-prod"
        );
        match endpoint.autenticacao {
            EstrategiaAutenticacaoSerpro::Oauth2Producao {
                consumer_key,
                consumer_secret,
                ..
            } => {
                assert_eq!(consumer_key.expose_secret(), "ck-prod");
                assert_eq!(consumer_secret.expose_secret(), "cs-prod");
            }
            _ => panic!("esperava Oauth2Producao"),
        }
    }

    #[tokio::test]
    async fn divida_producao_bloqueada_independente_de_config() {
        // Decisão D5: mesmo configurando URL, produção da Dívida Ativa fica
        // bloqueada — SEM fallback silencioso e sem exceção.
        let config = Config::from_pairs(pares(&[
            ("SERPRO_CONSUMER_KEY", "ck-prod"),
            ("SERPRO_CONSUMER_SECRET", "cs-prod"),
            (
                "SERPRO_DIVIDA_ATIVA_PROD_URL",
                "https://serpro.test/divida-prod",
            ),
        ]))
        .expect("config válida");
        let resolver = SerproEndpointResolverImpl::novo(config.serpro);

        let erro = resolver
            .resolver(ProdutoSerpro::DividaAtiva, AmbienteSerpro::Producao)
            .await
            .unwrap_err();

        assert!(matches!(erro, ErroEndpoint::DividaAtivaProducaoBloqueada));
    }
}
