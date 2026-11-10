//! Fase F — Erros do domínio de relatórios.

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ErroRelatorio {
    #[error("relatório não encontrado")]
    RelatorioNaoEncontrado,
    #[error("link de compartilhamento não encontrado")]
    LinkNaoEncontrado,
    #[error("link de compartilhamento expirado")]
    LinkExpirado,
    #[error("link de compartilhamento revogado")]
    LinkRevogado,
    #[error("limite de acessos do link atingido")]
    LinkLimiteAtingido,
    #[error("tipo de relatório inválido: {0}")]
    TipoInvalido(String),
    #[error("formato de exportação inválido: {0}")]
    FormatoInvalido(String),
    #[error("falha de persistência: {0}")]
    Persistencia(String),
    #[error("erro interno: {0}")]
    Interno(String),
}

impl ErroRelatorio {
    pub const fn codigo(&self) -> &'static str {
        match self {
            Self::RelatorioNaoEncontrado => "RELATORIO_NAO_ENCONTRADO",
            Self::LinkNaoEncontrado => "LINK_NAO_ENCONTRADO",
            Self::LinkExpirado => "LINK_EXPIRADO",
            Self::LinkRevogado => "LINK_REVOGADO",
            Self::LinkLimiteAtingido => "LINK_LIMITE_ATINGIDO",
            Self::TipoInvalido(_) => "TIPO_INVALIDO",
            Self::FormatoInvalido(_) => "FORMATO_INVALIDO",
            Self::Persistencia(_) => "PERSISTENCIA",
            Self::Interno(_) => "ERRO_INTERNO",
        }
    }
}
