use std::fmt;
use std::str::FromStr;

use thiserror::Error;

use super::Papel;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Audiencia {
    Admin,
    Client,
}

impl Audiencia {
    pub const fn como_str(self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::Client => "client",
        }
    }

    pub const fn como_str_banco(self) -> &'static str {
        match self {
            Self::Admin => "ADMIN",
            Self::Client => "CLIENT",
        }
    }

    pub const fn aceita(self, papel: Papel) -> bool {
        match self {
            Self::Admin => matches!(papel, Papel::Admin | Papel::Operador),
            Self::Client => matches!(papel, Papel::Cliente),
        }
    }
}

impl fmt::Display for Audiencia {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.como_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("audiência inválida; use admin ou client")]
pub struct AudienciaInvalida;

impl FromStr for Audiencia {
    type Err = AudienciaInvalida;

    fn from_str(valor: &str) -> Result<Self, Self::Err> {
        match valor {
            "admin" | "ADMIN" => Ok(Self::Admin),
            "client" | "CLIENT" => Ok(Self::Client),
            _ => Err(AudienciaInvalida),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admin_aceita_apenas_papeis_do_escritorio() {
        assert!(Audiencia::Admin.aceita(Papel::Admin));
        assert!(Audiencia::Admin.aceita(Papel::Operador));
        assert!(!Audiencia::Admin.aceita(Papel::Cliente));
    }

    #[test]
    fn client_aceita_apenas_cliente() {
        assert!(Audiencia::Client.aceita(Papel::Cliente));
        assert!(!Audiencia::Client.aceita(Papel::Admin));
        assert!(!Audiencia::Client.aceita(Papel::Operador));
    }

    #[test]
    fn converte_entre_representacoes() {
        assert_eq!("admin".parse::<Audiencia>(), Ok(Audiencia::Admin));
        assert_eq!("CLIENT".parse::<Audiencia>(), Ok(Audiencia::Client));
        assert_eq!(Audiencia::Admin.como_str_banco(), "ADMIN");
        assert!("super".parse::<Audiencia>().is_err());
    }
}
