//! Erros do domínio de notificações.

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ErroNotificacao {
    #[error("notificação não encontrada")]
    NotificacaoNaoEncontrada,
    #[error("preferência de notificação não encontrada")]
    PreferenciaNaoEncontrada,
    #[error("template de notificação não encontrado")]
    TemplateNaoEncontrado,
    #[error("usuário não possui permissão para esta notificação")]
    SemPermissao,
    #[error("canal não configurado: {0}")]
    CanalNaoConfigurado(String),
    #[error("provedor de envio indisponível: {0}")]
    ProvedorIndisponivel(String),
    #[error("falha de persistência: {0}")]
    Persistencia(String),
    #[error("erro interno: {0}")]
    Interno(String),
}

impl ErroNotificacao {
    pub const fn codigo(&self) -> &'static str {
        match self {
            Self::NotificacaoNaoEncontrada => "NOTIFICACAO_NAO_ENCONTRADA",
            Self::PreferenciaNaoEncontrada => "PREFERENCIA_NAO_ENCONTRADA",
            Self::TemplateNaoEncontrado => "TEMPLATE_NAO_ENCONTRADO",
            Self::SemPermissao => "SEM_PERMISSAO",
            Self::CanalNaoConfigurado(_) => "CANAL_NAO_CONFIGURADO",
            Self::ProvedorIndisponivel(_) => "PROVEDOR_INDISPONIVEL",
            Self::Persistencia(_) => "PERSISTENCIA",
            Self::Interno(_) => "ERRO_INTERNO",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codigos_estaveis() {
        assert_eq!(
            ErroNotificacao::NotificacaoNaoEncontrada.codigo(),
            "NOTIFICACAO_NAO_ENCONTRADA"
        );
        assert_eq!(
            ErroNotificacao::PreferenciaNaoEncontrada.codigo(),
            "PREFERENCIA_NAO_ENCONTRADA"
        );
        assert_eq!(ErroNotificacao::SemPermissao.codigo(), "SEM_PERMISSAO");
    }
}
