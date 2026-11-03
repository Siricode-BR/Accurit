pub mod ports;
pub mod use_cases;

pub use ports::{
    AuditoriaIntegraContador, CatalogoEscritoriosIntegra, ClienteIntegraContador,
    ContextoIntegraContador, EntregaIntegraContador, EventoAuditoriaIntegra, EventoChamadaIntegra,
    RepositorioIdempotencia, ReservaIdempotencia, ValidadorProcuracao,
};
pub use use_cases::{
    ComandoIntegraContador, ExecutarIntegraContadorUseCase, ResultadoIntegraContador,
};

pub const PRODUTO_INTEGRA_CONTADOR: &str = "INTEGRA_CONTADOR";
