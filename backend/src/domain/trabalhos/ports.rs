//! Portas do domínio de trabalhos agendados e consultas em lote.
//! Implementadas pela infrastructure (Diesel, job scheduler, etc.).

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::trabalhos::entidades::{
    ConsultaLote, ExecucaoTrabalho, ItemConsultaLote, NovoConsultaLote, NovoItemConsultaLote,
    NovoTrabalhoAgendado, TrabalhoAgendado,
};
use crate::domain::trabalhos::enums::{StatusExecucao, StatusLote};
use crate::domain::trabalhos::erros::ErroTrabalho;

/// Repositório de trabalhos agendados.
#[async_trait]
pub trait RepositorioTrabalhosAgendados: Send + Sync {
    async fn criar(
        &self,
        trabalho: &NovoTrabalhoAgendado,
    ) -> Result<TrabalhoAgendado, ErroTrabalho>;
    async fn buscar_por_id(&self, id: Uuid) -> Result<Option<TrabalhoAgendado>, ErroTrabalho>;
    async fn listar_por_escritorio(
        &self,
        escritorio_id: Uuid,
    ) -> Result<Vec<TrabalhoAgendado>, ErroTrabalho>;
    async fn listar_devidos(
        &self,
        agora: DateTime<Utc>,
    ) -> Result<Vec<TrabalhoAgendado>, ErroTrabalho>;
    async fn atualizar_proxima_execucao(
        &self,
        id: Uuid,
        proxima: Option<DateTime<Utc>>,
        ultima: Option<DateTime<Utc>>,
    ) -> Result<(), ErroTrabalho>;
    async fn habilitar(&self, id: Uuid) -> Result<(), ErroTrabalho>;
    async fn desabilitar(&self, id: Uuid) -> Result<(), ErroTrabalho>;
}

/// Repositório de execuções de trabalho.
#[async_trait]
pub trait RepositorioExecucoesTrabalho: Send + Sync {
    async fn criar(&self, execucao: &ExecucaoTrabalho) -> Result<Uuid, ErroTrabalho>;
    async fn buscar_por_id(&self, id: Uuid) -> Result<Option<ExecucaoTrabalho>, ErroTrabalho>;
    async fn listar_por_trabalho(
        &self,
        trabalho_agendado_id: Uuid,
    ) -> Result<Vec<ExecucaoTrabalho>, ErroTrabalho>;
    async fn listar_por_escritorio(
        &self,
        escritorio_id: Uuid,
    ) -> Result<Vec<ExecucaoTrabalho>, ErroTrabalho>;
    async fn atualizar_status(
        &self,
        id: Uuid,
        status: StatusExecucao,
        finalizado_em: Option<DateTime<Utc>>,
        erro_mensagem: Option<String>,
        registros_processados: i32,
        registros_com_erro: i32,
    ) -> Result<(), ErroTrabalho>;
}

/// Executor de trabalho: roda uma tarefa fiscal agendada.
#[async_trait]
pub trait ExecutorTrabalho: Send + Sync {
    async fn executar(&self, trabalho: &TrabalhoAgendado) -> Result<(i32, i32), ErroTrabalho>;
}

/// Scheduler de trabalhos: calcula a próxima execução com base no cron.
pub trait SchedulerTrabalho: Send + Sync {
    fn proxima_execucao(&self, cron: &str, depois_de: DateTime<Utc>) -> Option<DateTime<Utc>>;
}

/// Repositório de consultas em lote.
#[async_trait]
pub trait RepositorioConsultasLote: Send + Sync {
    async fn criar(&self, lote: &NovoConsultaLote) -> Result<ConsultaLote, ErroTrabalho>;
    async fn buscar_por_id(&self, id: Uuid) -> Result<Option<ConsultaLote>, ErroTrabalho>;
    async fn listar_por_escritorio(
        &self,
        escritorio_id: Uuid,
    ) -> Result<Vec<ConsultaLote>, ErroTrabalho>;
    #[allow(clippy::too_many_arguments)]
    async fn atualizar_status(
        &self,
        id: Uuid,
        status: StatusLote,
        iniciado_em: Option<DateTime<Utc>>,
        finalizado_em: Option<DateTime<Utc>>,
        total_itens: i32,
        itens_concluidos: i32,
        itens_com_erro: i32,
    ) -> Result<(), ErroTrabalho>;
}

/// Repositório de itens de consulta em lote.
#[async_trait]
pub trait RepositorioItensConsultaLote: Send + Sync {
    async fn criar(
        &self,
        consulta_lote_id: Uuid,
        itens: &[NovoItemConsultaLote],
    ) -> Result<Vec<ItemConsultaLote>, ErroTrabalho>;
    async fn listar_por_lote(
        &self,
        consulta_lote_id: Uuid,
    ) -> Result<Vec<ItemConsultaLote>, ErroTrabalho>;
    async fn atualizar_item(
        &self,
        id: Uuid,
        status: String,
        resultado_redigido: Option<serde_json::Value>,
        erro: Option<String>,
        processado_em: Option<DateTime<Utc>>,
    ) -> Result<(), ErroTrabalho>;
}
