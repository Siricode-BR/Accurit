//! Fase G — Casos de uso de medição de uso SERpro.

pub mod use_cases;

pub use use_cases::{
    CriarAtualizarAssinaturaUseCase, CriarPlanoEscritorioUseCase, ListarConsumoEscritorioUseCase,
    ListarPlanosUseCase, ServicoMedicaoSerproImpl,
};
pub use use_cases::{
    AgregadoConsumoDto, ChamadaRecenteDto, ConsumoEscritorioDto,
};
