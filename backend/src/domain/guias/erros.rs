//! Erros do domínio de emissão de guias.

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ErroGuia {
    #[error("guia não encontrada")]
    GuiaNaoEncontrada,
    #[error("guia já existe para esta chave de idempotência")]
    JaExisteIdempotencia,
    #[error("não é possível baixar guia com status {0}")]
    NaoPodeBaixar(String),
    #[error("provedor de emissão indisponível: {0}")]
    ProvedorIndisponivel(String),
    #[error("falha de persistência: {0}")]
    Persistencia(String),
    #[error("erro interno: {0}")]
    Interno(String),
}

impl ErroGuia {
    pub const fn codigo(&self) -> &'static str {
        match self {
            Self::GuiaNaoEncontrada => "GUIA_NAO_ENCONTRADA",
            Self::JaExisteIdempotencia => "GUIA_JA_EXISTE_IDEMPOTENCIA",
            Self::NaoPodeBaixar(_) => "NAO_PODE_BAIXAR",
            Self::ProvedorIndisponivel(_) => "PROVEDOR_INDISPONIVEL",
            Self::Persistencia(_) => "PERSISTENCIA",
            Self::Interno(_) => "ERRO_INTERNO",
        }
    }
}
