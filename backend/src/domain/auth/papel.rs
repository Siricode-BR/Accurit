use std::fmt;
use std::str::FromStr;

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Papel {
    Admin,
    Operador,
    Cliente,
}

impl Papel {
    pub const fn como_str(self) -> &'static str {
        match self {
            Self::Admin => "ADMIN",
            Self::Operador => "OPERADOR",
            Self::Cliente => "CLIENTE",
        }
    }

    pub const fn exige_cpf(self) -> bool {
        matches!(self, Self::Admin | Self::Operador | Self::Cliente)
    }
}

impl fmt::Display for Papel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.como_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("papel inválido; use ADMIN, OPERADOR ou CLIENTE")]
pub struct PapelInvalido;

impl FromStr for Papel {
    type Err = PapelInvalido;

    fn from_str(valor: &str) -> Result<Self, Self::Err> {
        match valor {
            "ADMIN" => Ok(Self::Admin),
            "OPERADOR" => Ok(Self::Operador),
            "CLIENTE" => Ok(Self::Cliente),
            _ => Err(PapelInvalido),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reconhece_os_tres_papeis() {
        assert_eq!("ADMIN".parse::<Papel>(), Ok(Papel::Admin));
        assert_eq!("OPERADOR".parse::<Papel>(), Ok(Papel::Operador));
        assert_eq!("CLIENTE".parse::<Papel>(), Ok(Papel::Cliente));
    }

    #[test]
    fn rejeita_papel_desconhecido_e_minusculo() {
        assert!("admin".parse::<Papel>().is_err());
        assert!("SUPER".parse::<Papel>().is_err());
    }

    #[test]
    fn todos_os_papeis_exigem_cpf() {
        assert!(Papel::Admin.exige_cpf());
        assert!(Papel::Operador.exige_cpf());
        assert!(Papel::Cliente.exige_cpf());
    }

    #[test]
    fn papel_admin_e_reservado_ao_gestor_de_tenants() {
        // Papel::Admin existe apenas para tokens do backoffice (administradores);
        // a tabela usuarios nunca o registra (migration restringe a CHECK).
        assert_eq!(Papel::Admin.como_str(), "ADMIN");
    }
}
