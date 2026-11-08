//! Fase B — Casos de uso de trabalhos agendados e consultas em lote.

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::application::divida_ativa::use_cases::{
    ConsultarDevedorUseCase, ConsultarInscricaoUseCase,
};
use crate::application::integra_contador::ExecutarIntegraContadorUseCase;
use crate::domain::serpro::ambiente::AmbienteSerpro;
use crate::domain::trabalhos::entidades::{
    ConsultaLote, ExecucaoTrabalho, ItemConsultaLote, NovoConsultaLote, NovoItemConsultaLote,
    NovoTrabalhoAgendado, TrabalhoAgendado,
};
use crate::domain::trabalhos::enums::TipoTrabalho;
use crate::domain::trabalhos::erros::ErroTrabalho;
use crate::domain::trabalhos::ports::{
    ExecutorTrabalho, RepositorioConsultasLote, RepositorioExecucoesTrabalho,
    RepositorioItensConsultaLote, RepositorioTrabalhosAgendados, SchedulerTrabalho,
};

// ============================================================================
// Caso de uso: Criar trabalho agendado
// ============================================================================

pub struct CriarTrabalhoAgendadoUseCase {
    repositorio: Arc<dyn RepositorioTrabalhosAgendados>,
}

impl CriarTrabalhoAgendadoUseCase {
    pub fn novo(repositorio: Arc<dyn RepositorioTrabalhosAgendados>) -> Self {
        Self { repositorio }
    }

    pub async fn executar(
        &self,
        comando: NovoTrabalhoAgendado,
    ) -> Result<TrabalhoAgendado, ErroTrabalho> {
        self.repositorio.criar(&comando).await
    }
}

// ============================================================================
// Caso de uso: Listar trabalhos agendados por escritório
// ============================================================================

pub struct ListarTrabalhosAgendadosUseCase {
    repositorio: Arc<dyn RepositorioTrabalhosAgendados>,
}

impl ListarTrabalhosAgendadosUseCase {
    pub fn novo(repositorio: Arc<dyn RepositorioTrabalhosAgendados>) -> Self {
        Self { repositorio }
    }

    pub async fn executar(
        &self,
        escritorio_id: Uuid,
    ) -> Result<Vec<TrabalhoAgendado>, ErroTrabalho> {
        self.repositorio.listar_por_escritorio(escritorio_id).await
    }
}

// ============================================================================
// Caso de uso: Buscar trabalho agendado por ID
// ============================================================================

pub struct BuscarTrabalhoAgendadoUseCase {
    repositorio: Arc<dyn RepositorioTrabalhosAgendados>,
}

impl BuscarTrabalhoAgendadoUseCase {
    pub fn novo(repositorio: Arc<dyn RepositorioTrabalhosAgendados>) -> Self {
        Self { repositorio }
    }

    pub async fn executar(&self, id: Uuid) -> Result<Option<TrabalhoAgendado>, ErroTrabalho> {
        self.repositorio.buscar_por_id(id).await
    }
}

// ============================================================================
// Caso de uso: Habilitar / Desabilitar trabalho agendado
// ============================================================================

pub struct HabilitarDesabilitarTrabalhoUseCase {
    repositorio: Arc<dyn RepositorioTrabalhosAgendados>,
}

impl HabilitarDesabilitarTrabalhoUseCase {
    pub fn novo(repositorio: Arc<dyn RepositorioTrabalhosAgendados>) -> Self {
        Self { repositorio }
    }

    pub async fn habilitar(&self, id: Uuid) -> Result<(), ErroTrabalho> {
        self.repositorio.habilitar(id).await
    }

    pub async fn desabilitar(&self, id: Uuid) -> Result<(), ErroTrabalho> {
        self.repositorio.desabilitar(id).await
    }
}

// ============================================================================
// Caso de uso: Listar execuções de trabalho
// ============================================================================

pub struct ListarExecucoesTrabalhoUseCase {
    repositorio: Arc<dyn RepositorioExecucoesTrabalho>,
}

impl ListarExecucoesTrabalhoUseCase {
    pub fn novo(repositorio: Arc<dyn RepositorioExecucoesTrabalho>) -> Self {
        Self { repositorio }
    }

    pub async fn executar_por_trabalho(
        &self,
        trabalho_agendado_id: Uuid,
    ) -> Result<Vec<ExecucaoTrabalho>, ErroTrabalho> {
        self.repositorio
            .listar_por_trabalho(trabalho_agendado_id)
            .await
    }

    pub async fn executar_por_escritorio(
        &self,
        escritorio_id: Uuid,
    ) -> Result<Vec<ExecucaoTrabalho>, ErroTrabalho> {
        self.repositorio.listar_por_escritorio(escritorio_id).await
    }
}

// ============================================================================
// Caso de uso: Criar consulta em lote
// ============================================================================

pub struct CriarConsultaLoteUseCase {
    repositorio: Arc<dyn RepositorioConsultasLote>,
    itens_repositorio: Arc<dyn RepositorioItensConsultaLote>,
}

impl CriarConsultaLoteUseCase {
    pub fn novo(
        repositorio: Arc<dyn RepositorioConsultasLote>,
        itens_repositorio: Arc<dyn RepositorioItensConsultaLote>,
    ) -> Self {
        Self {
            repositorio,
            itens_repositorio,
        }
    }

    pub async fn executar(
        &self,
        comando: NovoConsultaLote,
        documentos: &[NovoItemConsultaLote],
    ) -> Result<ConsultaLote, ErroTrabalho> {
        let lote = self.repositorio.criar(&comando).await?;
        let _itens = self.itens_repositorio.criar(lote.id, documentos).await?;
        Ok(lote)
    }
}

// ============================================================================
// Caso de uso: Listar consultas em lote por escritório
// ============================================================================

pub struct ListarConsultasLoteUseCase {
    repositorio: Arc<dyn RepositorioConsultasLote>,
}

impl ListarConsultasLoteUseCase {
    pub fn novo(repositorio: Arc<dyn RepositorioConsultasLote>) -> Self {
        Self { repositorio }
    }

    pub async fn executar(&self, escritorio_id: Uuid) -> Result<Vec<ConsultaLote>, ErroTrabalho> {
        self.repositorio.listar_por_escritorio(escritorio_id).await
    }
}

// ============================================================================
// Caso de uso: Buscar consulta em lote por ID
// ============================================================================

pub struct BuscarConsultaLoteUseCase {
    repositorio: Arc<dyn RepositorioConsultasLote>,
}

impl BuscarConsultaLoteUseCase {
    pub fn novo(repositorio: Arc<dyn RepositorioConsultasLote>) -> Self {
        Self { repositorio }
    }

    pub async fn executar(&self, id: Uuid) -> Result<Option<ConsultaLote>, ErroTrabalho> {
        self.repositorio.buscar_por_id(id).await
    }
}

// ============================================================================
// Caso de uso: Listar itens de consulta em lote
// ============================================================================

pub struct ListarItensConsultaLoteUseCase {
    repositorio: Arc<dyn RepositorioItensConsultaLote>,
}

impl ListarItensConsultaLoteUseCase {
    pub fn novo(repositorio: Arc<dyn RepositorioItensConsultaLote>) -> Self {
        Self { repositorio }
    }

    pub async fn executar(
        &self,
        consulta_lote_id: Uuid,
    ) -> Result<Vec<ItemConsultaLote>, ErroTrabalho> {
        self.repositorio.listar_por_lote(consulta_lote_id).await
    }
}

// ============================================================================
// Executor de trabalho: executa uma tarefa fiscal agendada
// ============================================================================

#[allow(dead_code)]
pub struct ExecutorTrabalhoImpl {
    consultar_devedor: Arc<ConsultarDevedorUseCase>,
    consultar_inscricao: Arc<ConsultarInscricaoUseCase>,
    executar_integra_contador: Arc<ExecutarIntegraContadorUseCase>,
}

impl ExecutorTrabalhoImpl {
    pub fn novo(
        consultar_devedor: Arc<ConsultarDevedorUseCase>,
        consultar_inscricao: Arc<ConsultarInscricaoUseCase>,
        executar_integra_contador: Arc<ExecutarIntegraContadorUseCase>,
    ) -> Self {
        Self {
            consultar_devedor,
            consultar_inscricao,
            executar_integra_contador,
        }
    }
}

#[async_trait]
impl ExecutorTrabalho for ExecutorTrabalhoImpl {
    async fn executar(&self, trabalho: &TrabalhoAgendado) -> Result<(i32, i32), ErroTrabalho> {
        let ambiente = match trabalho.ambiente.as_str() {
            "TRIAL" => AmbienteSerpro::Trial,
            "PRODUCAO" => AmbienteSerpro::Producao,
            _ => return Err(ErroTrabalho::Interno("ambiente inválido".into())),
        };

        match trabalho.tipo {
            TipoTrabalho::ConsultaDividaAtiva => {
                self.executar_consulta_divida_ativa(trabalho, ambiente)
                    .await
            }
            TipoTrabalho::ValidarProcuracao => {
                self.executar_validar_procuracao(trabalho, ambiente).await
            }
            TipoTrabalho::ConsultaIntegraContador => {
                self.executar_consulta_integra_contador(trabalho, ambiente)
                    .await
            }
        }
    }
}

impl ExecutorTrabalhoImpl {
    async fn executar_consulta_divida_ativa(
        &self,
        _trabalho: &TrabalhoAgendado,
        _ambiente: AmbienteSerpro,
    ) -> Result<(i32, i32), ErroTrabalho> {
        // Implementação placeholder: a consulta real usa ConsultarDevedorUseCase
        // com documentos específicos. Para o job runner, isso seria chamado
        // com uma lista de documentos configurada no trabalho.
        Ok((0, 0))
    }

    async fn executar_validar_procuracao(
        &self,
        _trabalho: &TrabalhoAgendado,
        _ambiente: AmbienteSerpro,
    ) -> Result<(i32, i32), ErroTrabalho> {
        Ok((0, 0))
    }

    async fn executar_consulta_integra_contador(
        &self,
        _trabalho: &TrabalhoAgendado,
        _ambiente: AmbienteSerpro,
    ) -> Result<(i32, i32), ErroTrabalho> {
        Ok((0, 0))
    }
}

// ============================================================================
// Scheduler de trabalho: implementação simples de cron (placeholder)
// ============================================================================

pub struct SchedulerTrabalhoSimples;

impl SchedulerTrabalhoSimples {
    pub fn novo() -> Self {
        Self
    }
}

impl SchedulerTrabalho for SchedulerTrabalhoSimples {
    fn proxima_execucao(&self, _cron: &str, _depois_de: DateTime<Utc>) -> Option<DateTime<Utc>> {
        // Placeholder: retorna None para não executar automaticamente
        #![allow(dead_code)]
        None
    }
}
