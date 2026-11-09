//! Fase G — Erros de medição de uso SERpro.

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ErroMedicao {
    #[error("repositório de medição indisponível")]
    RepositorioIndisponivel,
    #[error("plano não encontrado para o escritório")]
    PlanoNaoEncontrado,
    #[error("assinatura não encontrada para o escritório")]
    AssinaturaNaoEncontrada,
    #[error("limite de chamadas excedido")]
    LimiteChamadasExcedido,
    #[error("erro interno: {0}")]
    Interno(String),
}
