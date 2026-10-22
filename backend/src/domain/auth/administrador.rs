use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Gestor de tenants do backoffice SaaS (ADR-0010, D2): vive fora de `usuarios`,
/// não pertence a nenhum escritório e autentica apenas na audiência `admin`.
#[derive(Debug, Clone)]
pub struct Administrador {
    pub id: Uuid,
    pub email: String,
    pub senha_hash: String,
    pub ativo: bool,
    pub bloqueado_ate: Option<DateTime<Utc>>,
}

impl Administrador {
    pub fn esta_bloqueado(&self, agora: DateTime<Utc>) -> bool {
        self.bloqueado_ate
            .is_some_and(|bloqueado_ate| bloqueado_ate > agora)
    }
}

/// Sessão de refresh do backoffice (`sessoes_admin`), sem `escritorio_id`.
#[derive(Debug, Clone)]
pub struct SessaoAdmin {
    pub id: Uuid,
    pub administrador_id: Uuid,
    pub familia_id: Uuid,
    pub refresh_hash: String,
    pub criado_em: DateTime<Utc>,
    pub expira_em: DateTime<Utc>,
    pub revogada_em: Option<DateTime<Utc>>,
    pub ip: Option<String>,
    pub user_agent: Option<String>,
}

impl SessaoAdmin {
    pub fn esta_ativa(&self, agora: DateTime<Utc>) -> bool {
        self.revogada_em.is_none() && self.expira_em > agora
    }
}

#[derive(Debug, Clone)]
pub struct SessaoAdminComAdministrador {
    pub sessao: SessaoAdmin,
    pub administrador: Administrador,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn administrador(bloqueado_ate: Option<DateTime<Utc>>) -> Administrador {
        Administrador {
            id: Uuid::now_v7(),
            email: "gestor@accurit.test".into(),
            senha_hash: "hash".into(),
            ativo: true,
            bloqueado_ate,
        }
    }

    #[test]
    fn detecta_bloqueio_vigente() {
        let agora = Utc::now();
        assert!(administrador(Some(agora + Duration::minutes(1))).esta_bloqueado(agora));
        assert!(!administrador(Some(agora - Duration::minutes(1))).esta_bloqueado(agora));
        assert!(!administrador(None).esta_bloqueado(agora));
    }

    fn sessao(revogada_em: Option<DateTime<Utc>>, expira_em: DateTime<Utc>) -> SessaoAdmin {
        SessaoAdmin {
            id: Uuid::now_v7(),
            administrador_id: Uuid::now_v7(),
            familia_id: Uuid::now_v7(),
            refresh_hash: "hash".into(),
            criado_em: Utc::now(),
            expira_em,
            revogada_em,
            ip: None,
            user_agent: None,
        }
    }

    #[test]
    fn detecta_sessao_admin_ativa() {
        let agora = Utc::now();
        assert!(sessao(None, agora + Duration::minutes(1)).esta_ativa(agora));
        assert!(!sessao(Some(agora), agora + Duration::minutes(1)).esta_ativa(agora));
        assert!(!sessao(None, agora - Duration::minutes(1)).esta_ativa(agora));
    }
}
