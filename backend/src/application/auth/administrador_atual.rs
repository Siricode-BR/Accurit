use std::sync::Arc;

use super::error::AuthError;
use super::models::{AdministradorAtualCommand, AdministradorPublico};
use super::ports::AdministradorRepositorio;

pub struct ObterAdministradorAtual {
    administradores: Arc<dyn AdministradorRepositorio>,
}

impl ObterAdministradorAtual {
    pub fn novo(administradores: Arc<dyn AdministradorRepositorio>) -> Self {
        Self { administradores }
    }

    pub fn executar(
        &self,
        comando: AdministradorAtualCommand,
    ) -> Result<AdministradorPublico, AuthError> {
        let Some(administrador) = self
            .administradores
            .buscar_por_id(comando.administrador_id)?
        else {
            return Err(AuthError::SessaoExpirada);
        };

        if !administrador.ativo {
            return Err(AuthError::SessaoExpirada);
        }

        Ok(AdministradorPublico {
            id: administrador.id,
            email: administrador.email,
        })
    }
}
