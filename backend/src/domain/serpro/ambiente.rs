use std::fmt;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AmbienteSerpro {
    Trial,
    Producao,
}

impl FromStr for AmbienteSerpro {
    type Err = AmbienteSerproError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "TRIAL" => Ok(Self::Trial),
            "PRODUCAO" => Ok(Self::Producao),
            _ => Err(AmbienteSerproError::Invalido(s.to_owned())),
        }
    }
}

impl AmbienteSerpro {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Trial => "TRIAL",
            Self::Producao => "PRODUCAO",
        }
    }

    pub fn from_bool(modo_trial: bool) -> Self {
        if modo_trial {
            Self::Trial
        } else {
            Self::Producao
        }
    }
}

impl fmt::Display for AmbienteSerpro {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AmbienteSerproError {
    #[error("ambiente SERPRO inválido: {0}")]
    Invalido(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trial_as_str() {
        assert_eq!(AmbienteSerpro::Trial.as_str(), "TRIAL");
    }

    #[test]
    fn producao_as_str() {
        assert_eq!(AmbienteSerpro::Producao.as_str(), "PRODUCAO");
    }

    #[test]
    fn from_bool_true() {
        assert_eq!(AmbienteSerpro::from_bool(true), AmbienteSerpro::Trial);
    }

    #[test]
    fn from_bool_false() {
        assert_eq!(AmbienteSerpro::from_bool(false), AmbienteSerpro::Producao);
    }

    #[test]
    fn from_str_valido() {
        assert_eq!(
            AmbienteSerpro::from_str("TRIAL").unwrap(),
            AmbienteSerpro::Trial
        );
        assert_eq!(
            AmbienteSerpro::from_str("PRODUCAO").unwrap(),
            AmbienteSerpro::Producao
        );
    }

    #[test]
    fn from_str_invalido() {
        assert!(matches!(
            AmbienteSerpro::from_str("INVALIDO"),
            Err(AmbienteSerproError::Invalido(_))
        ));
    }

    #[test]
    fn display() {
        assert_eq!(AmbienteSerpro::Trial.to_string(), "TRIAL");
        assert_eq!(AmbienteSerpro::Producao.to_string(), "PRODUCAO");
    }
}
