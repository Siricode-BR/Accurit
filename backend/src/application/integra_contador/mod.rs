pub mod ports;
pub mod use_cases;

pub use ports::{
    AuditoriaIntegraContador, AutenticaProcuradorCliente, AutenticarProcuradorResultado,
    CatalogoEscritoriosIntegra, CertificadoA1Provider, ClienteIntegraContador,
    ContextoIntegraContador, EntregaIntegraContador, EventoAuditoriaIntegra, EventoChamadaIntegra,
    RepositorioIdempotencia, ReservaIdempotencia, TokensSerproRepositorio, ValidadorProcuracao,
};
pub use use_cases::{
    AutenticarProcuradorUseCase, ComandoAutenticarProcurador, ComandoIntegraContador,
    ExecutarIntegraContadorUseCase, ResultadoAutenticarProcurador, ResultadoIntegraContador,
};

pub const PRODUTO_INTEGRA_CONTADOR: &str = "INTEGRA_CONTADOR";
