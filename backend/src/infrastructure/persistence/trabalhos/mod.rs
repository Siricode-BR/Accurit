//! Fase B — Repositórios Diesel para trabalhos agendados, execuções,
//! consultas em lote e itens de lote. Sempre isolados por `escritorio_id`.

pub mod repositorio_consultas_lote;
pub mod repositorio_execucoes_trabalho;
pub mod repositorio_itens_consulta_lote;
pub mod repositorio_trabalhos_agendados;

pub use repositorio_consultas_lote::ConsultasLoteDiesel;
pub use repositorio_execucoes_trabalho::ExecucoesTrabalhoDiesel;
pub use repositorio_itens_consulta_lote::ItensConsultaLoteDiesel;
pub use repositorio_trabalhos_agendados::TrabalhosAgendadosDiesel;
