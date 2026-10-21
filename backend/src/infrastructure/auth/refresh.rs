use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::RngCore;
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};

use crate::application::auth::RefreshTokenService;

const TAMANHO_TOKEN_BYTES: usize = 32;

#[derive(Debug, Clone, Copy, Default)]
pub struct RefreshSha256;

impl RefreshTokenService for RefreshSha256 {
    fn gerar(&self) -> String {
        let mut bytes = [0_u8; TAMANHO_TOKEN_BYTES];
        OsRng.fill_bytes(&mut bytes);
        URL_SAFE_NO_PAD.encode(bytes)
    }

    fn hash(&self, valor: &str) -> String {
        let digest = Sha256::digest(valor.as_bytes());
        digest.iter().map(|byte| format!("{byte:02x}")).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gera_tokens_aleatorios_com_256_bits() {
        let servico = RefreshSha256;
        let primeiro = servico.gerar();
        let segundo = servico.gerar();
        assert_eq!(primeiro.len(), 43);
        assert_ne!(primeiro, segundo);
    }

    #[test]
    fn hash_tem_64_caracteres_hexadecimais_e_e_estavel() {
        let servico = RefreshSha256;
        let hash = servico.hash("token-de-teste");
        assert_eq!(hash.len(), 64);
        assert!(hash.bytes().all(|byte| byte.is_ascii_hexdigit()));
        assert_eq!(hash, servico.hash("token-de-teste"));
    }
}
