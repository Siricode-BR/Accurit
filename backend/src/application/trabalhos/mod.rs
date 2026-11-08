//! Fase B — Casos de uso de trabalhos agendados e consultas em lote.

pub mod use_cases;

pub use use_cases::{
    BuscarConsultaLoteUseCase, BuscarTrabalhoAgendadoUseCase, CriarConsultaLoteUseCase,
    CriarTrabalhoAgendadoUseCase, ExecutorTrabalhoImpl, HabilitarDesabilitarTrabalhoUseCase,
    ListarConsultasLoteUseCase, ListarExecucoesTrabalhoUseCase, ListarItensConsultaLoteUseCase,
    ListarTrabalhosAgendadosUseCase,
};
