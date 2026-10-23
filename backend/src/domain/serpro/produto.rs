use std::fmt;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProdutoSerpro {
    IntegraContador,
    DividaAtiva,
}

impl ProdutoSerpro {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::IntegraContador => "INTEGRA_CONTADOR",
            Self::DividaAtiva => "DIVIDA_ATIVA",
        }
    }
}

impl FromStr for ProdutoSerpro {
    type Err = ProdutoSerproError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "INTEGRA_CONTADOR" => Ok(Self::IntegraContador),
            "DIVIDA_ATIVA" => Ok(Self::DividaAtiva),
            _ => Err(ProdutoSerproError::Invalido(s.to_owned())),
        }
    }
}

impl fmt::Display for ProdutoSerpro {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ProdutoSerproError {
    #[error("produto SERPRO inválido: {0}")]
    Invalido(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integra_contador_as_str() {
        assert_eq!(ProdutoSerpro::IntegraContador.as_str(), "INTEGRA_CONTADOR");
    }

    #[test]
    fn divida_ativa_as_str() {
        assert_eq!(ProdutoSerpro::DividaAtiva.as_str(), "DIVIDA_ATIVA");
    }

    #[test]
    fn from_str_valido() {
        assert_eq!(
            ProdutoSerpro::from_str("INTEGRA_CONTADOR").unwrap(),
            ProdutoSerpro::IntegraContador
        );
        assert_eq!(
            ProdutoSerpro::from_str("DIVIDA_ATIVA").unwrap(),
            ProdutoSerpro::DividaAtiva
        );
    }

    #[test]
    fn from_str_invalido() {
        assert!(matches!(
            ProdutoSerpro::from_str("INVALIDO"),
            Err(ProdutoSerproError::Invalido(_))
        ));
    }

    #[test]
    fn display() {
        assert_eq!(
            ProdutoSerpro::IntegraContador.to_string(),
            "INTEGRA_CONTADOR"
        );
        assert_eq!(ProdutoSerpro::DividaAtiva.to_string(), "DIVIDA_ATIVA");
    }
}
