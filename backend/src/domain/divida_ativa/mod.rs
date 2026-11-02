//! Fase 4 — Dívida Ativa da União (somente leitura, ADR-0002).

pub mod devedor;
pub mod enums;
pub mod erros;
pub mod inscricao;
pub mod moeda;
pub mod ports;
pub mod vo;

pub use devedor::{
    ComandoConsultaDevedor, ComandoConsultaInscricao, EstadoConsulta, FonteConsulta,
    InscricaoDetalhe, InscricaoResumo, RegistroConsultaFiscal, ResultadoConsultaDevedor,
    mascarar_documento, parse_moeda_ptbr_estrito,
};
pub use erros::{ErroConsultaDivida, ErroDividaAtiva};
pub use moeda::parse_moeda_ptbr_estrito as parse_moeda_divida_ativa;
pub use vo::{NumeroInscricao, TipoChave, hash_consulta, mascarar_chave};
pub type ConsultaDevedor = ComandoConsultaDevedor;
pub type RegistroConsulta = RegistroConsultaFiscal;
