use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::{Audiencia, Papel};

#[derive(Debug, Clone)]
pub struct Escritorio {
    pub id: Uuid,
    pub slug: String,
}

#[derive(Debug, Clone)]
pub struct Usuario {
    pub id: Uuid,
    pub escritorio_id: Uuid,
    pub email: String,
    pub senha_hash: String,
    pub papel: Papel,
    pub ativo: bool,
    pub bloqueado_ate: Option<DateTime<Utc>>,
}

impl Usuario {
    pub fn esta_bloqueado(&self, agora: DateTime<Utc>) -> bool {
        self.bloqueado_ate
            .is_some_and(|bloqueado_ate| bloqueado_ate > agora)
    }
}

#[derive(Debug, Clone)]
pub struct Sessao {
    pub id: Uuid,
    pub escritorio_id: Uuid,
    pub usuario_id: Uuid,
    pub audiencia: Audiencia,
    pub familia_id: Uuid,
    pub refresh_hash: String,
    pub criado_em: DateTime<Utc>,
    pub expira_em: DateTime<Utc>,
    pub revogada_em: Option<DateTime<Utc>>,
    pub ip: Option<String>,
    pub user_agent: Option<String>,
}

impl Sessao {
    pub fn esta_ativa(&self, agora: DateTime<Utc>) -> bool {
        self.revogada_em.is_none() && self.expira_em > agora
    }
}

#[derive(Debug, Clone)]
pub struct SessaoComUsuario {
    pub sessao: Sessao,
    pub usuario: Usuario,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn usuario(bloqueado_ate: Option<DateTime<Utc>>) -> Usuario {
        Usuario {
            id: Uuid::now_v7(),
            escritorio_id: Uuid::now_v7(),
            email: "teste@exemplo.test".into(),
            senha_hash: "hash".into(),
            papel: Papel::Admin,
            ativo: true,
            bloqueado_ate,
        }
    }

    #[test]
    fn detecta_bloqueio_vigente() {
        let agora = Utc::now();
        assert!(usuario(Some(agora + Duration::minutes(1))).esta_bloqueado(agora));
        assert!(!usuario(Some(agora - Duration::minutes(1))).esta_bloqueado(agora));
        assert!(!usuario(None).esta_bloqueado(agora));
    }

    fn sessao(revogada_em: Option<DateTime<Utc>>, expira_em: DateTime<Utc>) -> Sessao {
        Sessao {
            id: Uuid::now_v7(),
            escritorio_id: Uuid::now_v7(),
            usuario_id: Uuid::now_v7(),
            audiencia: Audiencia::Admin,
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
    fn detecta_sessao_ativa() {
        let agora = Utc::now();
        assert!(sessao(None, agora + Duration::minutes(1)).esta_ativa(agora));
        assert!(!sessao(Some(agora), agora + Duration::minutes(1)).esta_ativa(agora));
        assert!(!sessao(None, agora - Duration::minutes(1)).esta_ativa(agora));
    }
}
