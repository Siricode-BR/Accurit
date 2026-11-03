//! Fase 5 — casos de uso do Integra Contador.

pub mod ports;
pub mod use_cases;

pub use ports::{
    AutenticadorProcurador, CatalogoEscritoriosIntegra, ClienteIntegraContador,
    EntregaIntegraContador, RepositorioIdempotencia, ReservaIdempotencia, ValidadorProcuracao,
};
pub use use_cases::{
    ComandoIntegraContador, ExecutarIntegraContadorUseCase, PRODUTO_INTEGRA_CONTADOR,
};
