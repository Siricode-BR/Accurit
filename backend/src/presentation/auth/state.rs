use std::sync::Arc;

use thiserror::Error;

use crate::application::auth::{
    EncerrarSessao, LoginUsuario, ObterUsuarioAtual, PoliticaAuth, RenovarSessao, SenhaError,
    SenhaService, TokenService,
};
use crate::infrastructure::auth::{HashArgon2, JwtService, RefreshSha256, RepositorioDiesel};
use crate::infrastructure::config::Config;
use crate::infrastructure::persistence::pool::DbPool;

use super::cookie::CookieSettings;
use super::rate_limit::LimiteLogin;

#[derive(Debug, Error)]
pub enum AuthInitError {
    #[error("não foi possível configurar o serviço de senhas")]
    Senha(#[from] SenhaError),
}

#[derive(Clone)]
pub struct AuthState {
    pub login: Arc<LoginUsuario>,
    pub renovar: Arc<RenovarSessao>,
    pub logout: Arc<EncerrarSessao>,
    pub usuario_atual: Arc<ObterUsuarioAtual>,
    pub tokens: Arc<dyn TokenService>,
    pub cookie: CookieSettings,
    pub limite_login: LimiteLogin,
}

impl AuthState {
    pub fn novo(pool: DbPool, config: &Config) -> Result<Self, AuthInitError> {
        let repositorio = Arc::new(RepositorioDiesel::novo(pool));
        let usuarios = repositorio.clone();
        let sessoes = repositorio.clone();
        let auditoria = repositorio;

        let senha: Arc<dyn SenhaService> = Arc::new(HashArgon2::novo(&config.auth)?);
        let tokens: Arc<dyn TokenService> = Arc::new(JwtService::novo(
            config.jwt_secret.clone(),
            config.auth.access_ttl_secs,
        ));
        let refresh = Arc::new(RefreshSha256);

        let politica = PoliticaAuth {
            access_ttl_secs: config.auth.access_ttl_secs,
            refresh_ttl_admin_secs: config.auth.refresh_ttl_admin_secs,
            refresh_ttl_client_secs: config.auth.refresh_ttl_client_secs,
            max_failed_attempts: config.auth.max_failed_attempts,
            lockout_secs: config.auth.lockout_secs,
        };

        let login = Arc::new(LoginUsuario::novo(
            usuarios.clone(),
            sessoes.clone(),
            auditoria.clone(),
            senha,
            tokens.clone(),
            refresh.clone(),
            politica.clone(),
        ));
        let renovar = Arc::new(RenovarSessao::novo(
            sessoes.clone(),
            auditoria.clone(),
            tokens.clone(),
            refresh.clone(),
            politica,
        ));
        let logout = Arc::new(EncerrarSessao::novo(sessoes, auditoria, refresh));
        let usuario_atual = Arc::new(ObterUsuarioAtual::novo(usuarios));

        Ok(Self {
            login,
            renovar,
            logout,
            usuario_atual,
            tokens,
            cookie: CookieSettings {
                secure: config.auth.cookie_secure,
            },
            limite_login: LimiteLogin::novo(),
        })
    }
}
