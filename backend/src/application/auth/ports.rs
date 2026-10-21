use std::net::IpAddr;

use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;

use crate::domain::auth::{Audiencia, Escritorio, Papel, Sessao, SessaoComUsuario, Usuario};

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RepositorioError {
    #[error("banco de dados indisponível")]
    Indisponivel,
    #[error("falha ao acessar dados de autenticação")]
    Dados,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("falha ao processar senha")]
pub struct SenhaError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum TokenError {
    #[error("token expirado")]
    Expirado,
    #[error("audiência incompatível")]
    Audiencia,
    #[error("token inválido")]
    Invalido,
}

#[derive(Debug, Clone)]
pub struct TokenEmitido {
    pub token: String,
    pub expires_in: u64,
}

#[derive(Debug, Clone)]
pub struct ClaimsAccess {
    pub usuario_id: Uuid,
    pub escritorio_id: Uuid,
    pub papel: Papel,
    pub audiencia: Audiencia,
    pub jti: Uuid,
}

#[derive(Debug, Clone)]
pub struct EventoAuditoria {
    pub escritorio_id: Uuid,
    pub usuario_id: Option<Uuid>,
    pub acao: &'static str,
    pub entidade: &'static str,
    pub entidade_id: Option<Uuid>,
    pub valor_novo: Value,
    pub ip: Option<IpAddr>,
}

pub trait UsuarioRepositorio: Send + Sync {
    fn buscar_escritorio_por_slug(
        &self,
        slug: &str,
    ) -> Result<Option<Escritorio>, RepositorioError>;

    fn buscar_por_escritorio_e_email(
        &self,
        escritorio_id: Uuid,
        email: &str,
    ) -> Result<Option<Usuario>, RepositorioError>;

    fn buscar_por_id(
        &self,
        escritorio_id: Uuid,
        usuario_id: Uuid,
    ) -> Result<Option<Usuario>, RepositorioError>;

    fn registrar_falha(
        &self,
        usuario_id: Uuid,
        max_tentativas: u32,
        bloqueio_secs: u64,
    ) -> Result<(), RepositorioError>;

    fn registrar_sucesso(&self, usuario_id: Uuid) -> Result<(), RepositorioError>;
}

pub trait SessaoRepositorio: Send + Sync {
    fn criar(&self, sessao: &Sessao) -> Result<(), RepositorioError>;

    fn buscar_por_hash(
        &self,
        refresh_hash: &str,
    ) -> Result<Option<SessaoComUsuario>, RepositorioError>;

    fn revogar(&self, sessao_id: Uuid) -> Result<bool, RepositorioError>;

    fn revogar_familia(&self, familia_id: Uuid) -> Result<u64, RepositorioError>;
}

pub trait AuditoriaRepositorio: Send + Sync {
    fn registrar(&self, evento: &EventoAuditoria) -> Result<(), RepositorioError>;
}

pub trait SenhaService: Send + Sync {
    fn verificar(&self, senha: &str, hash: &str) -> bool;

    fn hash_ficticio(&self) -> &str;

    fn gerar_hash(&self, senha: &str) -> Result<String, SenhaError>;
}

pub trait TokenService: Send + Sync {
    fn emitir(&self, usuario: &Usuario, audiencia: Audiencia) -> Result<TokenEmitido, TokenError>;

    fn validar(&self, token: &str, audiencia: Audiencia) -> Result<ClaimsAccess, TokenError>;
}

pub trait RefreshTokenService: Send + Sync {
    fn gerar(&self) -> String;

    fn hash(&self, valor: &str) -> String;
}
