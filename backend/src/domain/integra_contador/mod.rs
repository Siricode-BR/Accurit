//! Fase 5 — Integra Contador (SERPRO).
//! Envelope comum, verbos, catálogo de serviços, regras de procuração e assinatura XML.

pub mod envelope;
pub mod erros;
pub mod procuração;
pub mod servicos;
pub mod assinatura;

pub use envelope::{
    Identificacao, PedidoDados, IntegraContadorRequest, IntegraContadorResponse,
    IntegraContadorEnvelope, VerboIntegraContador,
};
pub use erros::{ErroIntegraContador, ErroProcuracao, ErroAssinatura};
pub use procuração::{RegraProcuracao, CodigoServicoProcuracao, ValidacaoProcuracao};
pub use servicos::{CatalogoServico, ServicoReferencia, validar_servico};
pub use assinatura::{AssinadorXml, TermoAutorizacao, XmlAssinado};