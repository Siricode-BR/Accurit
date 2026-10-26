//! Infraestrutura da Fase 4 (consultivo de Dívida Ativa): transporte
//! HTTP com tokens OAuth2, rate limit por escritório e repositório
//! Diesel do histórico consultas_fiscais.

pub mod cliente_http;
pub mod rate;
pub mod repositorio;
pub mod tokens;
