//! Domínio da Dívida Ativa (Fase 4): natureza **exclusivamente consultiva**
//! (SPEC §4.1). Consulta por devedor (CPF/CNPJ) e por inscrição, com
//! estados de resultado distintos para "sem pendências", erro de
//! integração e falha de autorização (SPEC §4.5).
//!
//! Sem Axum, Diesel, reqwest ou qualquer I/O: este módulo define apenas
//! tipos, VOs, enums, erros e regras de cálculo (dinheiro pt-BR, masking
//! de chaves de consulta).

pub mod devedor;
pub mod enums;
pub mod erros;
pub mod inscricao;
pub mod moeda;
pub mod vo;

pub use devedor::{DevedorConsulta, InscricaoDevedor, ResultadoDevedor};
pub use enums::{EstadoConsulta, FonteConsulta};
pub use erros::{ContextoConsulta, ErroConsultaDivida, ErroSerpro};
pub use inscricao::{InscricaoDetalhe, ReceitaPrincipal, SistemaOrigem, UnidadeServico};
pub use moeda::parse_moeda_ptbr_estrito;
pub use vo::{NumeroInscricao, TipoChave, hash_consulta, horario_consulta, mascarar_chave};
