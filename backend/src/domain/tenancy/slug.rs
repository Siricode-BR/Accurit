use std::fmt;
use std::str::FromStr;

use thiserror::Error;

const TAMANHO_MINIMO: usize = 3;
const TAMANHO_MAXIMO: usize = 40;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Slug(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum SlugInvalido {
    #[error("slug deve ter entre 3 e 40 caracteres")]
    Tamanho,
    #[error("slug deve conter apenas a-z, 0-9 e hífen")]
    Caractere,
}

impl Slug {
    pub fn novo(valor: impl Into<String>) -> Result<Self, SlugInvalido> {
        let valor = valor.into();
        Self::validar(&valor)?;
        Ok(Self(valor))
    }

    fn validar(valor: &str) -> Result<(), SlugInvalido> {
        let tamanho = valor.chars().count();
        if !(TAMANHO_MINIMO..=TAMANHO_MAXIMO).contains(&tamanho) {
            return Err(SlugInvalido::Tamanho);
        }

        if !valor
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        {
            return Err(SlugInvalido::Caractere);
        }

        Ok(())
    }

    pub fn como_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Slug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for Slug {
    type Err = SlugInvalido;

    fn from_str(valor: &str) -> Result<Self, Self::Err> {
        Self::novo(valor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aceita_slug_valido() {
        assert!(Slug::novo("escritorio-01").is_ok());
        assert_eq!("abc".parse::<Slug>().unwrap().como_str(), "abc");
    }

    #[test]
    fn rejeita_tamanho_fora_do_limite() {
        assert_eq!(Slug::novo("ab"), Err(SlugInvalido::Tamanho));
        assert_eq!(Slug::novo("a".repeat(41)), Err(SlugInvalido::Tamanho));
    }

    #[test]
    fn rejeita_caracteres_invalidos() {
        assert_eq!(Slug::novo("Escritorio"), Err(SlugInvalido::Caractere));
        assert_eq!(Slug::novo("com espaco"), Err(SlugInvalido::Caractere));
        assert_eq!(Slug::novo("acentuação"), Err(SlugInvalido::Caractere));
    }
}
