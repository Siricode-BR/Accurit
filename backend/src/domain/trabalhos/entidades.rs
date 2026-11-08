//! Entidades de domínio de trabalhos agendados e consultas em lote.
//! São objetos puros de negócio, sem dependência de Diesel ou Axum.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::trabalhos::enums::{StatusExecucao, StatusLote, TipoTrabalho};

/// Trabalho agendado: definição de uma tarefa fiscal recorrente para um escritório.
#[derive(Debug, Clone)]
pub struct TrabalhoAgendado {
    pub id: Uuid,
    pub escritorio_id: Uuid,
    pub nome: String,
    pub tipo: TipoTrabalho,
    pub produto: String,
    pub ambiente: String,
    pub cron_expression: String,
    pub habilitado: bool,
    pub proxima_execucao: Option<DateTime<Utc>>,
    pub ultima_execucao: Option<DateTime<Utc>>,
    pub criado_em: DateTime<Utc>,
    pub atualizado_em: DateTime<Utc>,
}

/// Dados para criação de um novo trabalho agendado.
#[derive(Debug, Clone)]
pub struct NovoTrabalhoAgendado {
    pub escritorio_id: Uuid,
    pub nome: String,
    pub tipo: TipoTrabalho,
    pub produto: String,
    pub ambiente: String,
    pub cron_expression: String,
}

/// Execução de um trabalho agendado: uma instância de execução no tempo.
#[derive(Debug, Clone)]
pub struct ExecucaoTrabalho {
    pub id: Uuid,
    pub trabalho_agendado_id: Uuid,
    pub escritorio_id: Uuid,
    pub status: StatusExecucao,
    pub iniciado_em: DateTime<Utc>,
    pub finalizado_em: Option<DateTime<Utc>>,
    pub erro_mensagem: Option<String>,
    pub registros_processados: i32,
    pub registros_com_erro: i32,
    pub criado_em: DateTime<Utc>,
}

/// Consulta em lote: grupo de consultas fiscais executadas de uma só vez.
#[derive(Debug, Clone)]
pub struct ConsultaLote {
    pub id: Uuid,
    pub escritorio_id: Uuid,
    pub nome: String,
    pub produto: String,
    pub servico: String,
    pub verbo: Option<String>,
    pub status: StatusLote,
    pub total_itens: i32,
    pub itens_concluidos: i32,
    pub itens_com_erro: i32,
    pub criado_em: DateTime<Utc>,
    pub iniciado_em: Option<DateTime<Utc>>,
    pub finalizado_em: Option<DateTime<Utc>>,
}

/// Dados para criação de uma nova consulta em lote.
#[derive(Debug, Clone)]
pub struct NovoConsultaLote {
    pub escritorio_id: Uuid,
    pub nome: String,
    pub produto: String,
    pub servico: String,
    pub verbo: Option<String>,
}

/// Item individual de uma consulta em lote.
#[derive(Debug, Clone)]
pub struct ItemConsultaLote {
    pub id: Uuid,
    pub consulta_lote_id: Uuid,
    pub documento: String,
    pub tipo: String,
    pub status: String,
    pub resultado_redigido: Option<serde_json::Value>,
    pub erro: Option<String>,
    pub criado_em: DateTime<Utc>,
    pub processado_em: Option<DateTime<Utc>>,
}

/// Dados para criação de um novo item de lote.
#[derive(Debug, Clone)]
pub struct NovoItemConsultaLote {
    pub documento: String,
    pub tipo: String,
}
