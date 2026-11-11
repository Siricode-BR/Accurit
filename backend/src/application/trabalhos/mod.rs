//! Fase B — Casos de uso de trabalhos agendados e consultas em lote.

pub mod runner;
pub mod scheduler;
pub mod use_cases;

pub use runner::{ConfiguracaoRunner, DependenciasRunner, JobRunner, ProcessadorLote};
pub use scheduler::SchedulerTrabalhoCron;
pub use use_cases::{
    BuscarConsultaLoteUseCase, BuscarTrabalhoAgendadoUseCase, CriarConsultaLoteUseCase,
    CriarTrabalhoAgendadoUseCase, ExecutorTrabalhoImpl, HabilitarDesabilitarTrabalhoUseCase,
    ListarConsultasLoteUseCase, ListarExecucoesTrabalhoUseCase, ListarItensConsultaLoteUseCase,
    ListarTrabalhosAgendadosUseCase, ReexecutarConsultaLoteUseCase,
};
