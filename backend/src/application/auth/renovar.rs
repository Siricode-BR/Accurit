use std::sync::Arc;

use chrono::{Duration, Utc};
use serde_json::json;
use uuid::Uuid;

use crate::domain::auth::Sessao;

use super::error::AuthError;
use super::models::{
    PoliticaAuth, RenovarCommand, SessaoEmitida, UsuarioPublico, truncar_user_agent,
};
use super::ports::{
    AuditoriaRepositorio, EventoAuditoria, RefreshTokenService, SessaoRepositorio, TokenService,
};

pub struct RenovarSessao {
    sessoes: Arc<dyn SessaoRepositorio>,
    auditoria: Arc<dyn AuditoriaRepositorio>,
    tokens: Arc<dyn TokenService>,
    refresh: Arc<dyn RefreshTokenService>,
    politica: PoliticaAuth,
}

impl RenovarSessao {
    pub fn novo(
        sessoes: Arc<dyn SessaoRepositorio>,
        auditoria: Arc<dyn AuditoriaRepositorio>,
        tokens: Arc<dyn TokenService>,
        refresh: Arc<dyn RefreshTokenService>,
        politica: PoliticaAuth,
    ) -> Self {
        Self {
            sessoes,
            auditoria,
            tokens,
            refresh,
            politica,
        }
    }

    pub fn executar(&self, comando: RenovarCommand) -> Result<SessaoEmitida, AuthError> {
        let refresh_hash = self.refresh.hash(&comando.refresh_token);
        let Some(encontrada) = self.sessoes.buscar_por_hash(&refresh_hash)? else {
            return Err(AuthError::SessaoExpirada);
        };

        let sessao = &encontrada.sessao;
        let usuario = &encontrada.usuario;
        let agora = Utc::now();

        if sessao.revogada_em.is_some() {
            self.revogar_e_auditar_reuso(sessao.familia_id, sessao.escritorio_id, usuario.id);
            return Err(AuthError::SessaoExpirada);
        }

        if sessao.audiencia != comando.audiencia {
            return Err(AuthError::AudienciaIncompativel);
        }

        if sessao.expira_em <= agora {
            let _ = self.sessoes.revogar(sessao.id)?;
            return Err(AuthError::SessaoExpirada);
        }

        if !usuario.ativo {
            self.sessoes.revogar_familia(sessao.familia_id)?;
            self.auditar(
                sessao.escritorio_id,
                Some(usuario.id),
                "AUTH_SESSAO_REVOGADA",
                json!({ "motivo": "USUARIO_INATIVO" }),
                comando.ip,
            );
            return Err(AuthError::SessaoExpirada);
        }

        if !comando.audiencia.aceita(usuario.papel) {
            return Err(AuthError::AudienciaIncompativel);
        }

        if !self.sessoes.revogar(sessao.id)? {
            self.revogar_e_auditar_reuso(sessao.familia_id, sessao.escritorio_id, usuario.id);
            return Err(AuthError::SessaoExpirada);
        }

        let token = self.tokens.emitir(usuario, comando.audiencia)?;
        let novo_refresh = self.refresh.gerar();
        let novo_hash = self.refresh.hash(&novo_refresh);
        let ttl = self.politica.ttl_refresh(comando.audiencia);
        let agora = Utc::now();

        self.sessoes.criar(&Sessao {
            id: Uuid::now_v7(),
            escritorio_id: sessao.escritorio_id,
            usuario_id: usuario.id,
            audiencia: comando.audiencia,
            familia_id: sessao.familia_id,
            refresh_hash: novo_hash,
            criado_em: agora,
            expira_em: agora + Duration::seconds(ttl as i64),
            revogada_em: None,
            ip: comando.ip.map(|ip| ip.to_string()),
            user_agent: truncar_user_agent(comando.user_agent),
        })?;

        Ok(SessaoEmitida {
            access_token: token.token,
            expires_in: token.expires_in,
            refresh_token: novo_refresh,
            refresh_ttl_secs: ttl,
            usuario: UsuarioPublico::from(usuario),
        })
    }

    fn revogar_e_auditar_reuso(&self, familia_id: Uuid, escritorio_id: Uuid, usuario_id: Uuid) {
        if let Err(erro) = self.sessoes.revogar_familia(familia_id) {
            tracing::error!(erro = %erro, "falha ao revogar família de sessões reutilizada");
        }
        self.auditar(
            escritorio_id,
            Some(usuario_id),
            "AUTH_REFRESH_REUSO",
            json!({ "familia_id": familia_id }),
            None,
        );
    }

    fn auditar(
        &self,
        escritorio_id: Uuid,
        usuario_id: Option<Uuid>,
        acao: &'static str,
        valor_novo: serde_json::Value,
        ip: Option<std::net::IpAddr>,
    ) {
        if let Err(erro) = self.auditoria.registrar(&EventoAuditoria {
            escritorio_id,
            usuario_id,
            acao,
            entidade: "sessoes",
            entidade_id: None,
            valor_novo,
            ip,
        }) {
            tracing::warn!(erro = %erro, "falha ao registrar auditoria de sessão");
        }
    }
}
