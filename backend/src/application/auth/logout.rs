use std::sync::Arc;

use serde_json::json;

use super::error::AuthError;
use super::models::LogoutCommand;
use super::ports::{AuditoriaRepositorio, EventoAuditoria, RefreshTokenService, SessaoRepositorio};

pub struct EncerrarSessao {
    sessoes: Arc<dyn SessaoRepositorio>,
    auditoria: Arc<dyn AuditoriaRepositorio>,
    refresh: Arc<dyn RefreshTokenService>,
}

impl EncerrarSessao {
    pub fn novo(
        sessoes: Arc<dyn SessaoRepositorio>,
        auditoria: Arc<dyn AuditoriaRepositorio>,
        refresh: Arc<dyn RefreshTokenService>,
    ) -> Self {
        Self {
            sessoes,
            auditoria,
            refresh,
        }
    }

    pub fn executar(&self, comando: LogoutCommand) -> Result<(), AuthError> {
        let refresh_hash = self.refresh.hash(&comando.refresh_token);
        let Some(encontrada) = self.sessoes.buscar_por_hash(&refresh_hash)? else {
            return Ok(());
        };

        if encontrada.sessao.audiencia != comando.audiencia {
            return Ok(());
        }

        self.sessoes.revogar_familia(encontrada.sessao.familia_id)?;

        if let Err(erro) = self.auditoria.registrar(&EventoAuditoria {
            escritorio_id: Some(encontrada.sessao.escritorio_id),
            usuario_id: Some(encontrada.usuario.id),
            acao: "AUTH_LOGOUT",
            entidade: "sessoes",
            entidade_id: Some(encontrada.sessao.id),
            valor_novo: json!({ "audiencia": comando.audiencia.como_str() }),
            ip: comando.ip,
        }) {
            tracing::warn!(erro = %erro, "falha ao registrar auditoria de logout");
        }

        Ok(())
    }
}
