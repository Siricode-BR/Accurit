//! Gerenciador de tokens OAuth2 client_credentials do SERPRO (SPEC
//! §3.1.1/§8): cache por escritório de credencial, expiração com
//! margem de segurança e renovação **uma única vez** em 401. O
//! token nunca aparece em logs ou envelope de erro.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use secrecy::SecretString;
use serde::Deserialize;
use url::Url;

use crate::domain::divida_ativa::erros::ErroSerpro;

/// Resposta de `POST /token` do gateway SERPRO.
#[derive(Debug, Clone, Deserialize)]
pub struct OAuthTokenResponse {
    pub access_token: String,
    #[serde(default)]
    pub token_type: Option<String>,
    /// Validade da concessão em segundos; ausente → expiração
    /// conservadora local (TTL_PADRAO_SECS).
    pub expires_in: Option<u64>,
    #[serde(default)]
    pub scope: Option<String>,
}

const MARGEM_EXPIRACAO_SECS: u64 = 60;
const TTL_PADRAO_SECS: u64 = 240;

/// Casamento de credencial persistida: chave de cache = hash curto de
/// (token_url, consumer_key). Nunca guarda a secret em texto — só o
/// token as ativo em `SecretString`.
#[async_trait]
pub trait FornecedorDeTokens: Send + Sync {
    /// Token vigente: cache se renovável ou nova concessão.
    async fn token_vigente(
        &self,
        token_url: &Url,
        consumer_key: &[u8],
        consumer_secret: &[u8],
    ) -> Result<SecretString, ErroSerpro>;

    /// Nova concessão (usada pela renovação única em 401).
    async fn conceder_novo_token(
        &self,
        token_url: &Url,
        consumer_key: &[u8],
        consumer_secret: &[u8],
    ) -> Result<SecretString, ErroSerpro>;
}

#[derive(Clone)]
pub struct FornecedorDeTokensHttp {
    http: reqwest::Client,
    cache: Arc<Mutex<HashMap<String, (SecretString, Instant)>>>,
}

impl FornecedorDeTokensHttp {
    pub fn novo(http: reqwest::Client) -> Self {
        Self {
            http,
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

/// Palavra de cache: SHA-256 hex de "url\u{1}|key" — a secret nunca vira
/// chave de dicionário legível.
fn chave_de_cache(token_url: &Url, consumer_key: &[u8]) -> String {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(token_url.as_str().as_bytes());
    hasher.update([0x01]);
    hasher.update(consumer_key);
    let resumo = hasher.finalize();
    hex::encode(resumo)
}

fn basic_de(consumer_key: &[u8], consumer_secret: &[u8]) -> String {
    format!(
        "Basic {}",
        STANDARD.encode([consumer_key, b":", consumer_secret].concat())
    )
}

#[async_trait]
impl FornecedorDeTokens for FornecedorDeTokensHttp {
    async fn token_vigente(
        &self,
        token_url: &Url,
        consumer_key: &[u8],
        consumer_secret: &[u8],
    ) -> Result<SecretString, ErroSerpro> {
        let chave = chave_de_cache(token_url, consumer_key);
        if let Some((token, expira_em)) = self
            .cache
            .lock()
            .expect("cache de tokens")
            .get(&chave)
            .map(|(token, expira_em)| (token.clone(), *expira_em))
        {
            if Instant::now() < expira_em {
                return Ok(token);
            }
            self.cache.lock().expect("cache de tokens").remove(&chave);
        }
        self.conceder_novo_token(token_url, consumer_key, consumer_secret)
            .await
    }

    async fn conceder_novo_token(
        &self,
        token_url: &Url,
        consumer_key: &[u8],
        consumer_secret: &[u8],
    ) -> Result<SecretString, ErroSerpro> {
        let resposta = self
            .http
            .post(token_url.clone())
            .header(
                reqwest::header::AUTHORIZATION,
                basic_de(consumer_key, consumer_secret),
            )
            .header(
                reqwest::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            )
            .body("grant_type=client_credentials")
            .send()
            .await;

        let resposta = match resposta {
            Ok(resposta) => resposta,
            // Falha de transporte ao buscar token → serviço
            // indisponível (renovação)
            Err(_) => return Err(ErroSerpro::Indisponivel(0)),
        };

        let status = resposta.status().as_u16();
        if status == 401 || status == 403 {
            return Err(ErroSerpro::NaoAutorizado);
        }
        if !(200..300).contains(&status) {
            return Err(ErroSerpro::Indisponivel(status));
        }

        let concedido: OAuthTokenResponse = match resposta.json().await {
            Ok(concedido) => concedido,
            Err(_) => return Err(ErroSerpro::Indeterminado),
        };
        let ttl_secs = concedido
            .expires_in
            .filter(|valor| *valor > 0)
            .unwrap_or(TTL_PADRAO_SECS);
        let expira_em =
            Instant::now() + Duration::from_secs(ttl_secs.saturating_sub(MARGEM_EXPIRACAO_SECS));

        let chave = chave_de_cache(token_url, consumer_key);
        let token = SecretString::from(concedido.access_token);
        self.cache
            .lock()
            .expect("cache de tokens")
            .insert(chave, (token.clone(), expira_em));

        Ok(token)
    }
}

/// Renovação única em 401 (SPEC §3.1.1):Invalida cache e concede
/// token novo; uma segunda falha de autorização vira `NaoAutorizado`
/// na chamada — sem loops.
pub struct RenovacaoUnica;

impl RenovacaoUnica {
    pub fn pode_renovar(tentativa: u8) -> bool {
        tentativa == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chave_de_cache_e_deterministica_e_oculta_a_key() {
        let url = Url::parse("https://serpro.test/token").expect("url");
        let a = chave_de_cache(&url, b"key1");
        let b = chave_de_cache(&url, b"key1");
        assert_eq!(a, b);
        assert_eq!(a.len(), 64);
        assert_ne!(a, chave_de_cache(&url, b"key2"));
        assert!(!a.contains("key1"));
    }

    #[test]
    fn basic_de_codifica_key_secret() {
        let basic = basic_de(b"ck", b"cs");
        assert!(basic.starts_with("Basic "));
        let decodificado = STANDARD
            .decode(basic.trim_start_matches("Basic "))
            .expect("base64");
        assert_eq!(decodificado, b"ck:cs");
    }

    #[test]
    fn renovacao_unica_permite_apenas_a_primeira() {
        assert!(RenovacaoUnica::pode_renovar(0));
        assert!(!RenovacaoUnica::pode_renovar(1));
    }
}
