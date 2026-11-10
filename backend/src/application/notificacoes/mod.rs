//! Fase C — Casos de uso de notificações.

pub mod use_cases;

pub use use_cases::{
    BuscarPreferenciaUseCase, ContarNotificacoesNaoLidasUseCase, CriarNotificacaoUseCase,
    CriarOuAtualizarPreferenciaUseCase, ListarNotificacoesUseCase,
    ListarTemplatesNotificacaoUseCase, MarcarNotificacaoLidaUseCase,
    MarcarTodasNotificacoesLidasUseCase,
};
