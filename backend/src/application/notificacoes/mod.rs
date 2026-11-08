//! Fase C — Casos de uso de notificações.

pub mod use_cases;

pub use use_cases::{
    CriarNotificacaoUseCase, ListarNotificacoesUseCase, MarcarNotificacaoLidaUseCase,
    MarcarTodasNotificacoesLidasUseCase, ContarNotificacoesNaoLidasUseCase,
    CriarOuAtualizarPreferenciaUseCase, BuscarPreferenciaUseCase,
    ListarTemplatesNotificacaoUseCase,
};
