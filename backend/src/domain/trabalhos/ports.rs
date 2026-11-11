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
    /// Reivindica atomicamente (`FOR UPDATE SKIP LOCKED`) trabalhos devidos
    /// — `habilitado` e `proxima_execucao <= agora` — até `max`, respeitando
    /// o lock por escritório (um job por vez por escritório) e o lock por
    /// trabalho (nenhuma execução pendente/executando em aberto). Cria a
    /// execução (`executando`, com lease) e devolve o trabalho e o id da
    /// execução criada.
    async fn reivindicar_devidos(
        &self,
        agora: DateTime<Utc>,
        lease_expira_em: DateTime<Utc>,
        max: usize,
    ) -> Result<Vec<(TrabalhoAgendado, Uuid)>, ErroTrabalho>;
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
    /// Job runner (Fase B.1): reivindica atomicamente execuções pendentes
    /// (retry com backoff) ou "presas" (lease expirado), incrementando
    /// `tentativas` e renovando o lease.
    async fn reivindicar_pendentes(
        &self,
        agora: DateTime<Utc>,
        lease_expira_em: DateTime<Utc>,
        max: usize,
    ) -> Result<Vec<ExecucaoTrabalho>, ErroTrabalho>;
    /// Renova o lease/heartbeat de uma execução em andamento.
    async fn renovar_lease(
        &self,
        id: Uuid,
        lease_expira_em: DateTime<Utc>,
    ) -> Result<(), ErroTrabalho>;
    /// Agenda nova tentativa de uma execução transitória: volta a
    /// `pendente` com backoff exponencial (`proxima_tentativa_em`).
    async fn agendar_reexecucao(
        &self,
        id: Uuid,
        erro_mensagem: String,
        proxima_tentativa_em: Option<DateTime<Utc>>,
        registros_processados: i32,
        registros_com_erro: i32,
    ) -> Result<(), ErroTrabalho>;
    /// Recupera execuções "presas" por crash: `executando` com lease
    /// expirado volta a `pendente` para reivindicação.
    async fn recuperar_presos(&self, agora: DateTime<Utc>) -> Result<i64, ErroTrabalho>;
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
    /// Job runner (Fase B.1): reivindica atomicamente lotes pendentes até
    /// `max`, com lock por escritório (um lote por vez por escritório) e
    /// reivindicação de lotes "presos" (lease expirado).
    async fn reivindicar_pendentes(
        &self,
        agora: DateTime<Utc>,
        lease_expira_em: DateTime<Utc>,
        max: usize,
    ) -> Result<Vec<ConsultaLote>, ErroTrabalho>;
    /// Renova o lease/heartbeat do lote em processamento.
    async fn renovar_lease(
        &self,
        id: Uuid,
        lease_expira_em: DateTime<Utc>,
    ) -> Result<(), ErroTrabalho>;
    /// Recupera lotes "presas" por crash: `executando` com lease expirado
    /// volta a `pendente`.
    async fn recuperar_presos(&self, agora: DateTime<Utc>) -> Result<i64, ErroTrabalho>;
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
    /// Job runner (Fase B.1): reivindica atomicamente itens pendentes do
    /// lote (inclusive itens recuperados de crash — lease expirado volta a
    /// `pendente` antes), incrementando `tentativas` e renovando o lease.
    async fn reivindicar_pendentes(
        &self,
        consulta_lote_id: Uuid,
        agora: DateTime<Utc>,
        lease_expira_em: DateTime<Utc>,
        max: usize,
    ) -> Result<Vec<ItemConsultaLote>, ErroTrabalho>;
    /// Agenda nova tentativa do item com backoff (`proxima_tentativa_em`);
    /// sem data (`None`) volta a `pendente` imediatamente.
    async fn agendar_reexecucao(
        &self,
        id: Uuid,
        erro: String,
        proxima_tentativa_em: Option<DateTime<Utc>>,
    ) -> Result<(), ErroTrabalho>;
    /// Recupera itens "presas" por crash: `executando` com lease expirado
    /// volta a `pendente`.
    async fn recuperar_presos(&self, agora: DateTime<Utc>) -> Result<i64, ErroTrabalho>;
    /// Contagem por status dos itens do lote, para a finalização.
    async fn contar_por_status(
        &self,
        consulta_lote_id: Uuid,
        agora: DateTime<Utc>,
    ) -> Result<ContagemItensLote, ErroTrabalho>;
    /// Execução manual (Fase B.1): devolve itens `falhou`/`bloqueado` a
    /// `pendente` (limpa erro/resultado/agendamento) para reprocessamento.
    async fn resetar_para_reexecucao(&self, consulta_lote_id: Uuid) -> Result<i64, ErroTrabalho>;
}

/// Contagem de itens do lote por status, na finalização do processamento.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ContagemItensLote {
    pub pendente: i64,
    pub executando: i64,
    pub concluido: i64,
    pub falhou: i64,
    pub bloqueado: i64,
    /// Itens pendentes cujo backoff ainda não venceu (`proxima_tentativa_em`
    /// no futuro) — não devem impedir a liberação do lote, apenas a sua
    /// finalização.
    pub pendente_backoff: i64,
}
