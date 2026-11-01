use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ErroDividaAtiva {
    #[error("chave de consulta inválida")]
    ChaveInvalida,
    #[error("falha ao autenticar no SERPRO: {0}")]
    Autenticacao(String),
    #[error("serviço indisponível")]
    Indisponivel,
    #[error("pendência não encontrada")]
    NaoEncontrado,
    #[error("erro interno de integração: {0}")]
    ErroInterno(String),
}
