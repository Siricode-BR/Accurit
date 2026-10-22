use std::sync::Arc;

use serde_json::json;

use super::error::AuthError;
use super::models::LogoutAdminCommand;
use super::ports::{
    AuditoriaRepositorio, EventoAuditoria, RefreshTokenService, SessaoAdminRepositorio,
};

pub struct EncerrarSessaoAdmin {
    sessoes_admin: Arc<dyn SessaoAdminRepositorio>,
    auditoria: Arc<dyn AuditoriaRepositorio>,
    refresh: Arc<dyn RefreshTokenService>,
}

impl EncerrarSessaoAdmin {
    pub fn novo(
        sessoes_admin: Arc<dyn SessaoAdminRepositorio>,
        auditoria: Arc<dyn AuditoriaRepositorio>,
        refresh: Arc<dyn RefreshTokenService>,
    ) -> Self {
        Self {
            sessoes_admin,
            auditoria,
            refresh,
        }
    }

    pub fn executar(&self, comando: LogoutAdminCommand) -> Result<(), AuthError> {
        let refresh_hash = self.refresh.hash(&comando.refresh_token);
        let Some(encontrada) = self.sessoes_admin.buscar_por_hash(&refresh_hash)? else {
            return Ok(());
        };

        self.sessoes_admin
            .revogar_familia(encontrada.sessao.familia_id)?;

        if let Err(erro) = self.auditoria.registrar(&EventoAuditoria {
            escritorio_id: None,
            usuario_id: None,
            acao: "AUTH_LOGOUT",
            entidade: "sessoes_admin",
            entidade_id: Some(encontrada.sessao.id),
            valor_novo: json!({
                "administrador_id": encontrada.administrador.id,
                "audiencia": "admin",
            }),
            ip: comando.ip,
        }) {
            tracing::warn!(erro = %erro, "falha ao registrar auditoria de logout");
        }

        Ok(())
    }
}
