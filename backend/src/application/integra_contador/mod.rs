pub mod ports;
pub mod use_cases;

pub use ports::{
    AuditoriaIntegraContador, AutenticaProcuradorCliente, AutenticarProcuradorResultado,
    CatalogoEscritoriosIntegra, CertificadoA1Provider, ClienteIntegraContador,
    ContextoIntegraContador, ContratantePlataforma, EntregaIntegraContador, EventoAuditoriaIntegra,
    EventoChamadaIntegra, LinhaCatalogoServico, ProvedorCatalogoIntegra, RegistroProcuracao,
    RepositorioIdempotencia, RepositorioProcuracoes, ServicoResolvido, TokensSerproRepositorio,
    ValidadorProcuracao,
};
pub use use_cases::{
    AutenticarProcuradorUseCase, ComandoAutenticarProcurador, ComandoIntegraContador,
    ComandoListarProcuracao, ComandoValidarProcuracao, EstadoProcuracaoCliente,
    EstadoProcuracaoServico, ExecutarIntegraContadorUseCase, ListarProcuracoesUseCase,
    ResultadoAutenticarProcurador, ResultadoIntegraContador, ResultadoValidarProcuracao,
    ValidarProcuracoesUseCase,
};

pub const PRODUTO_INTEGRA_CONTADOR: &str = "INTEGRA_CONTADOR";
