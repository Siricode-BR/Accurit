use std::sync::Arc;

use chrono::{Duration, Utc};
use serde_json::json;
use uuid::Uuid;

use crate::domain::auth::{Audiencia, SessaoAdmin};

use super::error::AuthError;
use super::models::{
    AdministradorPublico, PoliticaAuth, RenovarAdminCommand, SessaoAdminEmitida, truncar_user_agent,
};
use super::ports::{
    AdministradorRepositorio, AuditoriaRepositorio, EventoAuditoria, RefreshTokenService,
    SessaoAdminRepositorio, TokenService,
};

/// Rotação de refresh do backoffice (`sessoes_admin`), espelhando `RenovarSessao`.
pub struct RenovarSessaoAdmin {
    sessoes_admin: Arc<dyn SessaoAdminRepositorio>,
    administradores: Arc<dyn AdministradorRepositorio>,
    auditoria: Arc<dyn AuditoriaRepositorio>,
    tokens: Arc<dyn TokenService>,
    refresh: Arc<dyn RefreshTokenService>,
    politica: PoliticaAuth,
}

impl RenovarSessaoAdmin {
    #[allow(clippy::too_many_arguments)]
    pub fn novo(
        sessoes_admin: Arc<dyn SessaoAdminRepositorio>,
        administradores: Arc<dyn AdministradorRepositorio>,
        auditoria: Arc<dyn AuditoriaRepositorio>,
        tokens: Arc<dyn TokenService>,
        refresh: Arc<dyn RefreshTokenService>,
        politica: PoliticaAuth,
    ) -> Self {
        Self {
            sessoes_admin,
            administradores,
            auditoria,
            tokens,
            refresh,
            politica,
        }
    }

    pub fn executar(&self, comando: RenovarAdminCommand) -> Result<SessaoAdminEmitida, AuthError> {
        let refresh_hash = self.refresh.hash(&comando.refresh_token);
        let Some(encontrada) = self.sessoes_admin.buscar_por_hash(&refresh_hash)? else {
            return Err(AuthError::SessaoExpirada);
        };

        let sessao = &encontrada.sessao;
        let administrador = &encontrada.administrador;
        let agora = Utc::now();

        if sessao.revogada_em.is_some() {
            self.revogar_e_auditar_reuso(sessao.familia_id);
            return Err(AuthError::SessaoExpirada);
        }

        if sessao.expira_em <= agora {
            let _ = self.sessoes_admin.revogar(sessao.id)?;
            return Err(AuthError::SessaoExpirada);
        }

        if !administrador.ativo {
            self.sessoes_admin.revogar_familia(sessao.familia_id)?;
            self.auditar(
                "AUTH_SESSAO_REVOGADA",
                json!({ "motivo": "ADMIN_INATIVO", "familia_id": sessao.familia_id }),
            );
            return Err(AuthError::SessaoExpirada);
        }

        // Confirma no banco que o gestor continua ativo mesmo se mudou desde o login.
        let Some(atual) = self
            .administradores
            .buscar_por_id(sessao.administrador_id)?
        else {
            self.revogar_e_auditar_reuso(sessao.familia_id);
            return Err(AuthError::SessaoExpirada);
        };
        if !atual.ativo {
            self.sessoes_admin.revogar_familia(sessao.familia_id)?;
            self.auditar(
                "AUTH_SESSAO_REVOGADA",
                json!({ "motivo": "ADMIN_INATIVO", "familia_id": sessao.familia_id }),
            );
            return Err(AuthError::SessaoExpirada);
        }

        if !self.sessoes_admin.revogar(sessao.id)? {
            self.revogar_e_auditar_reuso(sessao.familia_id);
            return Err(AuthError::SessaoExpirada);
        }

        let audiencia = Audiencia::Admin;
        let token = self.tokens.emitir_admin(&atual, audiencia)?;
        let novo_refresh = self.refresh.gerar();
        let novo_hash = self.refresh.hash(&novo_refresh);
        let ttl = self.politica.ttl_refresh(audiencia);
        let agora = Utc::now();

        self.sessoes_admin.criar(&SessaoAdmin {
            id: Uuid::now_v7(),
            administrador_id: sessao.administrador_id,
            familia_id: sessao.familia_id,
            refresh_hash: novo_hash,
            criado_em: agora,
            expira_em: agora + Duration::seconds(ttl as i64),
            revogada_em: None,
            ip: comando.ip.map(|ip| ip.to_string()),
            user_agent: truncar_user_agent(comando.user_agent),
        })?;

        Ok(SessaoAdminEmitida {
            access_token: token.token,
            expires_in: token.expires_in,
            refresh_token: novo_refresh,
            refresh_ttl_secs: ttl,
            administrador: AdministradorPublico {
                id: atual.id,
                email: atual.email,
            },
        })
    }

    fn revogar_e_auditar_reuso(&self, familia_id: Uuid) {
        if let Err(erro) = self.sessoes_admin.revogar_familia(familia_id) {
            tracing::error!(erro = %erro, "falha ao revogar família de sessões do backoffice");
        }
        self.auditar("AUTH_REFRESH_REUSO", json!({ "familia_id": familia_id }));
    }

    fn auditar(&self, acao: &'static str, valor_novo: serde_json::Value) {
        if let Err(erro) = self.auditoria.registrar(&EventoAuditoria {
            escritorio_id: None,
            usuario_id: None,
            acao,
            entidade: "sessoes_admin",
            entidade_id: None,
            valor_novo,
            ip: None,
        }) {
            tracing::warn!(erro = %erro, "falha ao registrar auditoria de sessão");
        }
    }
}
