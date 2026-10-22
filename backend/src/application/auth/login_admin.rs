use std::sync::Arc;

use chrono::{Duration, Utc};
use serde_json::json;
use uuid::Uuid;

use crate::domain::auth::{Audiencia, SessaoAdmin};

use super::error::AuthError;
use super::models::{
    AdministradorPublico, LoginAdminCommand, PoliticaAuth, SessaoAdminEmitida, normalizar_email,
    truncar_user_agent,
};
use super::ports::{
    AdministradorRepositorio, AuditoriaRepositorio, EventoAuditoria, RefreshTokenService,
    SenhaService, SessaoAdminRepositorio, TokenService,
};

/// Login do backoffice SaaS (ADR-0010): consulta `administradores` e cria
/// sessão em `sessoes_admin`. Nenhum papel de `usuarios` autentica aqui.
pub struct LoginAdministrador {
    administradores: Arc<dyn AdministradorRepositorio>,
    sessoes_admin: Arc<dyn SessaoAdminRepositorio>,
    auditoria: Arc<dyn AuditoriaRepositorio>,
    senha: Arc<dyn SenhaService>,
    tokens: Arc<dyn TokenService>,
    refresh: Arc<dyn RefreshTokenService>,
    politica: PoliticaAuth,
}

impl LoginAdministrador {
    #[allow(clippy::too_many_arguments)]
    pub fn novo(
        administradores: Arc<dyn AdministradorRepositorio>,
        sessoes_admin: Arc<dyn SessaoAdminRepositorio>,
        auditoria: Arc<dyn AuditoriaRepositorio>,
        senha: Arc<dyn SenhaService>,
        tokens: Arc<dyn TokenService>,
        refresh: Arc<dyn RefreshTokenService>,
        politica: PoliticaAuth,
    ) -> Self {
        Self {
            administradores,
            sessoes_admin,
            auditoria,
            senha,
            tokens,
            refresh,
            politica,
        }
    }

    pub fn executar(&self, comando: LoginAdminCommand) -> Result<SessaoAdminEmitida, AuthError> {
        let agora = Utc::now();
        let email = normalizar_email(&comando.email);

        let Some(administrador) = self.administradores.buscar_por_email(&email)? else {
            let _ = self
                .senha
                .verificar(&comando.senha, self.senha.hash_ficticio());
            self.auditar_falha(None, &email, &comando, "ADMIN_INEXISTENTE");
            return Err(AuthError::CredenciaisInvalidas);
        };

        if !administrador.ativo {
            let _ = self
                .senha
                .verificar(&comando.senha, &administrador.senha_hash);
            self.auditar_falha(Some(administrador.id), &email, &comando, "ADMIN_INATIVO");
            return Err(AuthError::CredenciaisInvalidas);
        }

        if administrador.esta_bloqueado(agora) {
            let _ = self
                .senha
                .verificar(&comando.senha, &administrador.senha_hash);
            self.auditar_falha(Some(administrador.id), &email, &comando, "ADMIN_BLOQUEADO");
            return Err(AuthError::CredenciaisInvalidas);
        }

        if !self
            .senha
            .verificar(&comando.senha, &administrador.senha_hash)
        {
            self.administradores.registrar_falha(
                administrador.id,
                self.politica.max_failed_attempts,
                self.politica.lockout_secs,
            )?;
            self.auditar_falha(Some(administrador.id), &email, &comando, "SENHA_INVALIDA");
            return Err(AuthError::CredenciaisInvalidas);
        }

        self.administradores.registrar_sucesso(administrador.id)?;

        let audiencia = Audiencia::Admin;
        let token = self.tokens.emitir_admin(&administrador, audiencia)?;
        let refresh_token = self.refresh.gerar();
        let refresh_hash = self.refresh.hash(&refresh_token);
        let refresh_ttl_secs = self.politica.ttl_refresh(audiencia);
        let agora = Utc::now();

        let sessao = SessaoAdmin {
            id: Uuid::now_v7(),
            administrador_id: administrador.id,
            familia_id: Uuid::now_v7(),
            refresh_hash,
            criado_em: agora,
            expira_em: agora + Duration::seconds(refresh_ttl_secs as i64),
            revogada_em: None,
            ip: comando.ip.map(|ip| ip.to_string()),
            user_agent: truncar_user_agent(comando.user_agent.clone()),
        };
        self.sessoes_admin.criar(&sessao)?;

        if let Err(erro) = self.auditoria.registrar(&EventoAuditoria {
            escritorio_id: None,
            usuario_id: None,
            acao: "AUTH_LOGIN_SUCESSO",
            entidade: "sessoes_admin",
            entidade_id: Some(sessao.id),
            valor_novo: json!({ "administrador_id": administrador.id }),
            ip: comando.ip,
        }) {
            tracing::warn!(erro = %erro, "falha ao registrar auditoria de login");
        }

        Ok(SessaoAdminEmitida {
            access_token: token.token,
            expires_in: token.expires_in,
            refresh_token,
            refresh_ttl_secs,
            administrador: AdministradorPublico {
                id: administrador.id,
                email: administrador.email,
            },
        })
    }

    fn auditar_falha(
        &self,
        administrador_id: Option<Uuid>,
        email: &str,
        comando: &LoginAdminCommand,
        motivo: &'static str,
    ) {
        let email_hash = self.refresh.hash(email);
        if let Err(erro) = self.auditoria.registrar(&EventoAuditoria {
            escritorio_id: None,
            usuario_id: None,
            acao: "AUTH_LOGIN_FALHA",
            entidade: "administradores",
            entidade_id: administrador_id,
            valor_novo: json!({
                "motivo": motivo,
                "email_hash": email_hash,
            }),
            ip: comando.ip,
        }) {
            tracing::warn!(erro = %erro, "falha ao registrar auditoria de login");
        }
    }
}
