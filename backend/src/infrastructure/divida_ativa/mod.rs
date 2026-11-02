//! Adapters de infraestrutura da Fase 4: HTTP SERPRO, rate limit,
//! persistência Diesel e hash com pepper (SPEC §9.11).

pub mod cliente_http;
pub mod hash;
pub mod rate;
pub mod repositorio;
