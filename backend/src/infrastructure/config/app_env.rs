use std::fmt;
use std::str::FromStr;

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppEnv {
    Development,
    Staging,
    Production,
}

impl AppEnv {
    pub const fn como_str(self) -> &'static str {
        match self {
            Self::Development => "development",
            Self::Staging => "staging",
            Self::Production => "production",
        }
    }

    pub const fn e_producao(self) -> bool {
        matches!(self, Self::Production)
    }

    pub const fn usa_log_json(self) -> bool {
        !matches!(self, Self::Development)
    }
}

impl fmt::Display for AppEnv {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.como_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("APP_ENV inválido; use development, staging ou production")]
pub struct AppEnvInvalido;

impl FromStr for AppEnv {
    type Err = AppEnvInvalido;

    fn from_str(valor: &str) -> Result<Self, Self::Err> {
        match valor {
            "development" => Ok(Self::Development),
            "staging" => Ok(Self::Staging),
            "production" => Ok(Self::Production),
            _ => Err(AppEnvInvalido),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reconhece_ambientes_suportados() {
        assert_eq!("development".parse::<AppEnv>(), Ok(AppEnv::Development));
        assert_eq!("staging".parse::<AppEnv>(), Ok(AppEnv::Staging));
        assert_eq!("production".parse::<AppEnv>(), Ok(AppEnv::Production));
    }

    #[test]
    fn rejeita_ambiente_desconhecido() {
        assert!("prod".parse::<AppEnv>().is_err());
        assert!("Development".parse::<AppEnv>().is_err());
    }
}
