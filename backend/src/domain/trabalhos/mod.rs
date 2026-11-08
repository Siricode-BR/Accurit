//! Fase B — Job runner, consultas em lote e painel de rotina fiscal.
//! Domínio de agendamento e execução de tarefas fiscais recorrentes,
//! sempre isolado por `escritorio_id`.

pub mod entidades;
pub mod enums;
pub mod erros;
pub mod ports;

pub use entidades::{
    ConsultaLote, ExecucaoTrabalho, ItemConsultaLote, NovoConsultaLote, NovoItemConsultaLote,
    NovoTrabalhoAgendado, TrabalhoAgendado,
};
pub use enums::{StatusExecucao, StatusLote, TipoTrabalho};
pub use erros::ErroTrabalho;
pub use ports::{
    ExecutorTrabalho, RepositorioExecucoesTrabalho, RepositorioTrabalhosAgendados,
    SchedulerTrabalho,
};
