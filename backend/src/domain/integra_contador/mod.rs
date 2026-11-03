//! Fase 5 — Integra Contador (SERPRO).
//!
//! Camadas do domínio:
//! - `envelope`: envelope comum, `Identificacao`, `PedidoDados`, verbos
//!   (SPEC §5.1–§5.3);
//! - `servicos`: catálogo/allowlist e validação fail-closed (SPEC §5.4,
//!   D4/P4);
//! - `regras_procuracao`: regras servicos-x-procuracoes (SPEC §6);
//! - `assinatura`: termo de autorização e XMLDSig (SPEC §7.1, ADR-0004);
//! - `erros`: árvore de erros única, com `ErroAssinatura` definido
//!   exclusivamente em `assinatura.rs`.

pub mod assinatura;
pub mod envelope;
pub mod erros;
pub mod regras_procuracao;
pub mod servicos;

pub use assinatura::{AssinadorXml, ErroAssinatura, TermoAutorizacao, XmlAssinado};
pub use envelope::{
    Identificacao, IntegraContadorRequest, IntegraContadorResponse, MensagemSerpro, PedidoDados,
    TIPO_CNPJ, TIPO_CPF, VerboIntegraContador,
};
pub use erros::{ErroIntegraContador, ErroProcuracao};
pub use regras_procuracao::{
    RegraProcuracao, ValidacaoProcuracao, bloqueado_allowlist, regra_procuracao_para_servico,
};
pub use servicos::{
    ErroCatalogo, EstadoCatalogo, ServicoReferencia, buscar_servico, validar_servico,
};
