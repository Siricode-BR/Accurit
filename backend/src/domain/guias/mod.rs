//! Fase D — Emissão de guias e documentos.
//! Domínio de emissão de guias DAS/DARF via Integra Contador, com idempotência
//! e auditoria.

pub mod entidades;
pub mod enums;
pub mod erros;
pub mod ports;

pub use entidades::{
    GuiaEmissao, NovoGuiaEmissao,
};
pub use enums::{TipoGuia, StatusGuia};
pub use erros::ErroGuia;
pub use ports::{
    RepositorioGuiasEmissao, ProvedorEmissaoGuia,
};
