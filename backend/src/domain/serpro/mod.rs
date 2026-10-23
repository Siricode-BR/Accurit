pub mod ambiente;
pub mod credencial;
pub mod endpoints;
pub mod erros;
pub mod ports;
pub mod produto;

pub use ambiente::{AmbienteSerpro, AmbienteSerproError};
pub use credencial::{CertificadoA1, CredencialSerpro};
pub use endpoints::{EndpointError, EndpointSerpro, EstrategiaAutenticacaoSerpro};
pub use erros::{ErroAmbiente, ErroCredencial, ErroEndpoint};
pub use ports::{
    AlterarModoResultado, ResolverEndpointsSerpro, SerproAmbienteResolver, SerproCredentialProvider,
};
pub use produto::{ProdutoSerpro, ProdutoSerproError};
