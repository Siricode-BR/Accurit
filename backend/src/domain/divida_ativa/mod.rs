pub mod devedor;
pub mod erros;
pub mod ports;

pub use devedor::{
    ConsultaDevedor, DevedorPayload, DevedorResponse, FonteConsulta, InscricaoDetalhe,
    InscricaoDetalheResponse, InscricaoResumo, ResultadoConsultaDevedor,
};
pub use erros::ErroDividaAtiva;
pub use ports::{ClienteDividaAtiva, ConsultasFiscaisRepositorio, RegistroConsulta, ResolvedorChaveAlternativa};
