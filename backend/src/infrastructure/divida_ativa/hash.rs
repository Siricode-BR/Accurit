//! Hash SHA-256 com pepper de configuração para o histórico
//! `consultas_fiscais`: a chave de consulta nunca fica em claro nem no
//! banco nem em logs (SPEC §9.11). `hash` devolve `None` quando o
//! pepper não está configurado — comportamento documentado em
//! GO-LIVE-SERPRO (auditoria continua, sem hash).

use std::sync::Arc;

use secrecy::{ExposeSecret, SecretString};
use sha2::{Digest, Sha256};

use crate::domain::divida_ativa::ports::HashDeChaveConsulta;

#[derive(Clone)]
pub struct HashChaveSha256 {
    pepper: Option<SecretString>,
}

impl HashChaveSha256 {
    pub const fn novo(pepper: Option<SecretString>) -> Self {
        Self { pepper }
    }

    /// Serpro: hash com pepper vazio é invalidado — há sempre que haver
    /// uma sal de composição quando o pepper é conhecido.
    pub fn obrigatorio(ambiente_pepper: Option<SecretString>) -> Arc<dyn HashDeChaveConsulta> {
        Arc::new(Self::novo(ambiente_pepper))
    }
}

impl HashDeChaveConsulta for HashChaveSha256 {
    fn hash(&self, chave: &str) -> Option<String> {
        let Some(pepper) = &self.pepper else {
            return None;
        };
        let mut hasher = Sha256::new();
        hasher.update(pepper.expose_secret().as_bytes());
        hasher.update(chave.as_bytes());
        Some(hex::encode(hasher.finalize()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn com_pepper(pepper: &str) -> HashChaveSha256 {
        HashChaveSha256::novo(Some(SecretString::from(pepper.to_owned())))
    }

    #[test]
    fn hash_deterministico_com_pepper() {
        let hasher = com_pepper("sal-da-casa");
        let primeiro = hasher.hash("09781911768").expect("hash");
        let segundo = hasher.hash("09781911768").expect("hash");
        assert_eq!(primeiro, segundo);
        assert_eq!(primeiro.len(), 64);
    }

    #[test]
    fn pepper_diferente_muda_o_hash() {
        let primeiro = com_pepper("sal-1").hash("09781911768").expect("hash");
        let segundo = com_pepper("sal-2").hash("09781911768").expect("hash");
        assert_ne!(primeiro, segundo);
    }

    #[test]
    fn sem_pepper_devolve_none() {
        let hasher = HashChaveSha256::novo(None);
        assert_eq!(hasher.hash("09781911768"), None);
    }

    #[test]
    fn nunca_devolve_documento_em_claro() {
        let hash = com_pepper("sal").hash("09781911768").expect("hash");
        assert!(!hash.contains("09781911768"));
    }
}
