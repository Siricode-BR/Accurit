//! Fase G — Domínio de medição de uso SERpro.
//! Entidades puras de negócio, sem dependência de Diesel ou Axum.

pub mod entidades;
pub mod enums;
pub mod erros;
pub mod ports;

pub use entidades::{
    AgregadoConsumoSerpro, AssinaturaEscritorio, BloqueioSerpro, ChamadaSerpro,
    ComandoRegistrarBloqueio, ComandoRegistrarChamada, PlanoEscritorio, ResultadoRegistroChamada,
};
pub use enums::{MotivoBloqueio, OrigemChamada, PeriodoAgregado, ResultadoChamada};
pub use erros::ErroMedicao;
pub use ports::ServicoMedicaoSerpro;
