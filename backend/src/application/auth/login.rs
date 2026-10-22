use std::sync::Arc;

use chrono::{Duration, Utc};
use serde_json::json;
use uuid::Uuid;

use crate::domain::auth::Sessao;

use super::error::AuthError;
use super::models::{
    LoginCommand, PoliticaAuth, SessaoEmitida, UsuarioPublico, normalizar_email, normalizar_slug,
    truncar_user_agent,
};
use super::ports::{
    AuditoriaRepositorio, EventoAuditoria, RefreshTokenService, SenhaService, SessaoRepositorio,
    TokenService, UsuarioRepositorio,
};

pub struct LoginUsuario {
    usuarios: Arc<dyn UsuarioRepositorio>,
    sessoes: Arc<dyn SessaoRepositorio>,
    auditoria: Arc<dyn AuditoriaRepositorio>,
    senha: Arc<dyn SenhaService>,
    tokens: Arc<dyn TokenService>,
    refresh: Arc<dyn RefreshTokenService>,
    politica: PoliticaAuth,
}

impl LoginUsuario {
    #[allow(clippy::too_many_arguments)]
    pub fn novo(
        usuarios: Arc<dyn UsuarioRepositorio>,
        sessoes: Arc<dyn SessaoRepositorio>,
        auditoria: Arc<dyn AuditoriaRepositorio>,
        senha: Arc<dyn SenhaService>,
        tokens: Arc<dyn TokenService>,
        refresh: Arc<dyn RefreshTokenService>,
        politica: PoliticaAuth,
    ) -> Self {
        Self {
            usuarios,
            sessoes,
            auditoria,
            senha,
            tokens,
            refresh,
            politica,
        }
    }

    pub fn executar(&self, comando: LoginCommand) -> Result<SessaoEmitida, AuthError> {
        let agora = Utc::now();
        let slug = normalizar_slug(&comando.escritorio);
        let email = normalizar_email(&comando.email);

        let Some(escritorio) = self.usuarios.buscar_escritorio_por_slug(&slug)? else {
            let _ = self
                .senha
                .verificar(&comando.senha, self.senha.hash_ficticio());
            tracing::warn!(
                audiencia = %comando.audiencia,
                "tentativa de login para escritório inexistente"
            );
            return Err(AuthError::CredenciaisInvalidas);
        };

        let Some(usuario) = self
            .usuarios
            .buscar_por_escritorio_e_email(escritorio.id, &email)?
        else {
            let _ = self
                .senha
                .verificar(&comando.senha, self.senha.hash_ficticio());
            self.auditar_falha(escritorio.id, None, &email, &comando, "USUARIO_INEXISTENTE");
            return Err(AuthError::CredenciaisInvalidas);
        };

        if !usuario.ativo {
            let _ = self.senha.verificar(&comando.senha, &usuario.senha_hash);
            self.auditar_falha(
                escritorio.id,
                Some(usuario.id),
                &email,
                &comando,
                "USUARIO_INATIVO",
            );
            return Err(AuthError::CredenciaisInvalidas);
        }

        if usuario.esta_bloqueado(agora) {
            let _ = self.senha.verificar(&comando.senha, &usuario.senha_hash);
            self.auditar_falha(
                escritorio.id,
                Some(usuario.id),
                &email,
                &comando,
                "USUARIO_BLOQUEADO",
            );
            return Err(AuthError::CredenciaisInvalidas);
        }

        if !self.senha.verificar(&comando.senha, &usuario.senha_hash) {
            self.usuarios.registrar_falha(
                usuario.id,
                self.politica.max_failed_attempts,
                self.politica.lockout_secs,
            )?;
            self.auditar_falha(
                escritorio.id,
                Some(usuario.id),
                &email,
                &comando,
                "SENHA_INVALIDA",
            );
            return Err(AuthError::CredenciaisInvalidas);
        }

        if !comando.audiencia.aceita(usuario.papel) {
            self.auditar_falha(
                escritorio.id,
                Some(usuario.id),
                &email,
                &comando,
                "PAPEL_INCOMPATIVEL",
            );
            return Err(AuthError::CredenciaisInvalidas);
        }

        self.usuarios.registrar_sucesso(usuario.id)?;

        let token = self.tokens.emitir(&usuario, comando.audiencia)?;
        let refresh_token = self.refresh.gerar();
        let refresh_hash = self.refresh.hash(&refresh_token);
        let refresh_ttl_secs = self.politica.ttl_refresh(comando.audiencia);
        let agora = Utc::now();

        let sessao = Sessao {
            id: Uuid::now_v7(),
            escritorio_id: escritorio.id,
            usuario_id: usuario.id,
            audiencia: comando.audiencia,
            familia_id: Uuid::now_v7(),
            refresh_hash,
            criado_em: agora,
            expira_em: agora + Duration::seconds(refresh_ttl_secs as i64),
            revogada_em: None,
            ip: comando.ip.map(|ip| ip.to_string()),
            user_agent: truncar_user_agent(comando.user_agent.clone()),
        };
        self.sessoes.criar(&sessao)?;

        if let Err(erro) = self.auditoria.registrar(&EventoAuditoria {
            escritorio_id: Some(escritorio.id),
            usuario_id: Some(usuario.id),
            acao: "AUTH_LOGIN_SUCESSO",
            entidade: "sessoes",
            entidade_id: Some(sessao.id),
            valor_novo: json!({ "audiencia": comando.audiencia.como_str() }),
            ip: comando.ip,
        }) {
            tracing::warn!(erro = %erro, "falha ao registrar auditoria de login");
        }

        Ok(SessaoEmitida {
            access_token: token.token,
            expires_in: token.expires_in,
            refresh_token,
            refresh_ttl_secs,
            usuario: UsuarioPublico::from(&usuario),
        })
    }

    fn auditar_falha(
        &self,
        escritorio_id: Uuid,
        usuario_id: Option<Uuid>,
        email: &str,
        comando: &LoginCommand,
        motivo: &'static str,
    ) {
        let email_hash = self.refresh.hash(email);
        if let Err(erro) = self.auditoria.registrar(&EventoAuditoria {
            escritorio_id: Some(escritorio_id),
            usuario_id,
            acao: "AUTH_LOGIN_FALHA",
            entidade: "usuarios",
            entidade_id: usuario_id,
            valor_novo: json!({
                "audiencia": comando.audiencia.como_str(),
                "motivo": motivo,
                "email_hash": email_hash,
            }),
            ip: comando.ip,
        }) {
            tracing::warn!(erro = %erro, "falha ao registrar auditoria de login");
        }
    }
}
