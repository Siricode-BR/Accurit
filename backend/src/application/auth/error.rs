use thiserror::Error;

use super::ports::{RepositorioError, SenhaError, TokenError};

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AuthError {
    #[error("credenciais inválidas")]
    CredenciaisInvalidas,
    #[error("sessão expirada")]
    SessaoExpirada,
    #[error("audiência incompatível")]
    AudienciaIncompativel,
    #[error(transparent)]
    Repositorio(#[from] RepositorioError),
    #[error(transparent)]
    Senha(#[from] SenhaError),
    #[error(transparent)]
    Token(#[from] TokenError),
}
