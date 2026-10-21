use std::sync::Arc;

use super::error::AuthError;
use super::models::{UsuarioAtualCommand, UsuarioPublico};
use super::ports::UsuarioRepositorio;

pub struct ObterUsuarioAtual {
    usuarios: Arc<dyn UsuarioRepositorio>,
}

impl ObterUsuarioAtual {
    pub fn novo(usuarios: Arc<dyn UsuarioRepositorio>) -> Self {
        Self { usuarios }
    }

    pub fn executar(&self, comando: UsuarioAtualCommand) -> Result<UsuarioPublico, AuthError> {
        let Some(usuario) = self
            .usuarios
            .buscar_por_id(comando.escritorio_id, comando.usuario_id)?
        else {
            return Err(AuthError::SessaoExpirada);
        };

        if !usuario.ativo {
            return Err(AuthError::SessaoExpirada);
        }

        if !comando.audiencia.aceita(usuario.papel) {
            return Err(AuthError::AudienciaIncompativel);
        }

        Ok(UsuarioPublico::from(&usuario))
    }
}
