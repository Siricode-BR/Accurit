//! Fase G — Casos de uso de medição de uso SERPRO.

pub mod use_cases;

pub use use_cases::{
    AgregadoConsumoDto, ChamadaRecenteDto, ConsumoAdminDto, ConsumoAdminUseCase,
    ConsumoEscritorioAdminDto, ConsumoEscritorioDto, RankingConsumoDto, SerieConsumoDto,
};
pub use use_cases::{
    CriarAtualizarAssinaturaUseCase, CriarPlanoEscritorioUseCase, ListarConsumoEscritorioUseCase,
    ListarPlanosUseCase, ServicoMedicaoSerproImpl,
};
