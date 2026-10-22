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
            // A audiência `admin` pertence ao backoffice SaaS (ADR-0010): nenhum papel de
            // `usuarios` autentica nela — o gestor de tenants usa `administradores`.
            Self::Admin => false,
            // A plataforma do escritório aceita o titular (CLIENTE) e sua equipe (OPERADOR).
            Self::Client => matches!(papel, Papel::Operador | Papel::Cliente),
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
    fn nenhuma_audiencia_aceita_papel_admin_de_usuarios() {
        // O papel ADMIN tornou-se exclusivo do backoffice (tabela separada).
        assert!(!Audiencia::Admin.aceita(Papel::Admin));
        assert!(!Audiencia::Client.aceita(Papel::Admin));
    }

    #[test]
    fn client_aceita_papeis_da_plataforma_do_escritorio() {
        assert!(Audiencia::Client.aceita(Papel::Cliente));
        assert!(Audiencia::Client.aceita(Papel::Operador));
        assert!(!Audiencia::Admin.aceita(Papel::Operador));
        assert!(!Audiencia::Admin.aceita(Papel::Cliente));
    }

    #[test]
    fn converte_entre_representacoes() {
        assert_eq!("admin".parse::<Audiencia>(), Ok(Audiencia::Admin));
        assert_eq!("CLIENT".parse::<Audiencia>(), Ok(Audiencia::Client));
        assert_eq!(Audiencia::Admin.como_str_banco(), "ADMIN");
        assert!("super".parse::<Audiencia>().is_err());
    }
}
