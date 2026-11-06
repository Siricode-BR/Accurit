//! Implementação concreta do Integra Contador (infraestrutura).
//!
//! Este módulo contém as implementações dos ports da aplicação para o
//! Integra Contador, conectando o domínio ao infraestrutura SERPRO:
//! cliente HTTP, criptografia A1, resolução de endpoints, etc.

pub mod autenticacao;
pub mod cliente;
pub mod xml_assinador;

pub use autenticacao::AutenticaProcuradorClienteImpl;
pub use cliente::ClienteIntegraContadorImpl;
pub use xml_assinador::AssinadorXmlImpl;
