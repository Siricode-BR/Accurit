use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum FingerprintInvalido {
    #[error("fingerprint deve ter exatamente 64 caracteres hexadecimais")]
    Formato,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FingerprintSha256(String);

impl FingerprintSha256 {
    pub fn novo(valor: impl AsRef<str>) -> Result<Self, FingerprintInvalido> {
        let valor = valor.as_ref();
        if valor.len() != 64 || !valor.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(FingerprintInvalido::Formato);
        }
        Ok(Self(valor.to_ascii_lowercase()))
    }

    #[cfg(test)]
    pub fn novo_para_teste(valor: String) -> Self {
        Self(valor)
    }

    pub fn validar(&self) -> Result<(), FingerprintInvalido> {
        if self.0.len() != 64 || !self.0.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(FingerprintInvalido::Formato);
        }
        Ok(())
    }

    pub fn como_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for FingerprintSha256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for FingerprintSha256 {
    type Err = FingerprintInvalido;

    fn from_str(valor: &str) -> Result<Self, Self::Err> {
        Self::novo(valor)
    }
}

impl From<FingerprintInvalido> for FingerprintSha256 {
    fn from(_: FingerprintInvalido) -> Self {
        FingerprintSha256("0".repeat(64))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aceita_fingerprint_valido() {
        let fp = FingerprintSha256::novo("a".repeat(64)).expect("válido");
        assert_eq!(fp.como_str(), "a".repeat(64));
    }

    #[test]
    fn aceita_fingerprint_maiusculo() {
        let fp = FingerprintSha256::novo("A".repeat(64)).expect("válido");
        assert_eq!(fp.como_str(), "a".repeat(64));
    }

    #[test]
    fn rejeita_tamanho_errado() {
        assert_eq!(
            FingerprintSha256::novo("a".repeat(63)).unwrap_err(),
            FingerprintInvalido::Formato
        );
        assert_eq!(
            FingerprintSha256::novo("a".repeat(65)).unwrap_err(),
            FingerprintInvalido::Formato
        );
    }

    #[test]
    fn rejeita_caractere_invalido() {
        assert_eq!(
            FingerprintSha256::novo("g".repeat(64)).unwrap_err(),
            FingerprintInvalido::Formato
        );
        assert_eq!(
            FingerprintSha256::novo("a".repeat(63) + "z").unwrap_err(),
            FingerprintInvalido::Formato
        );
    }
}
