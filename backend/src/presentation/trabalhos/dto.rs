//! Fase B — DTOs de trabalho agendado, execuções e consultas em lote.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[allow(unused_imports)]
use crate::domain::trabalhos::enums::{StatusExecucao as _, StatusLote as _, TipoTrabalho as _};

/// Requisição para criar um trabalho agendado.
#[derive(Debug, Deserialize)]
pub struct RequisicaoCriarTrabalho {
    pub nome: String,
    pub tipo: String, // "consulta_divida_ativa" | "validar_procuracao" | "consulta_integra_contador"
    pub produto: String, // "INTEGRA_CONTADOR" | "DIVIDA_ATIVA"
    pub ambiente: String, // "TRIAL" | "PRODUCAO"
    pub cron_expression: String,
}

/// Resposta de um trabalho agendado.
#[derive(Debug, Serialize)]
pub struct RespostaTrabalhoAgendado {
    pub id: Uuid,
    pub escritorio_id: Uuid,
    pub nome: String,
    pub tipo: String,
    pub produto: String,
    pub ambiente: String,
    pub cron_expression: String,
    pub habilitado: bool,
    pub proxima_execucao: Option<DateTime<Utc>>,
    pub ultima_execucao: Option<DateTime<Utc>>,
    pub criado_em: DateTime<Utc>,
    pub atualizado_em: DateTime<Utc>,
}

/// Requisição para habilitar/desabilitar um trabalho.
#[derive(Debug, Deserialize)]
pub struct RequisicaoAlternarTrabalho {
    pub habilitado: bool,
}

/// Resposta de uma execução de trabalho.
#[derive(Debug, Serialize)]
pub struct RespostaExecucaoTrabalho {
    pub id: Uuid,
    pub trabalho_agendado_id: Uuid,
    pub escritorio_id: Uuid,
    pub status: String,
    pub iniciado_em: DateTime<Utc>,
    pub finalizado_em: Option<DateTime<Utc>>,
    pub erro_mensagem: Option<String>,
    pub registros_processados: i32,
    pub registros_com_erro: i32,
    pub criado_em: DateTime<Utc>,
}

/// Requisição para criar uma consulta em lote.
#[derive(Debug, Deserialize)]
pub struct RequisicaoCriarConsultaLote {
    pub nome: String,
    pub produto: String, // "INTEGRA_CONTADOR" | "DIVIDA_ATIVA"
    pub servico: String,
    pub verbo: Option<String>,
    pub documentos: Vec<RequisicaoItemConsultaLote>,
}

/// Item de consulta em lote na requisição.
#[derive(Debug, Deserialize)]
pub struct RequisicaoItemConsultaLote {
    pub documento: String,
    pub tipo: String, // "PF" | "PJ"
}

/// Resposta de uma consulta em lote.
#[derive(Debug, Serialize)]
pub struct RespostaConsultaLote {
    pub id: Uuid,
    pub escritorio_id: Uuid,
    pub nome: String,
    pub produto: String,
    pub servico: String,
    pub verbo: Option<String>,
    pub status: String,
    pub total_itens: i32,
    pub itens_concluidos: i32,
    pub itens_com_erro: i32,
    pub criado_em: DateTime<Utc>,
    pub iniciado_em: Option<DateTime<Utc>>,
    pub finalizado_em: Option<DateTime<Utc>>,
}

/// Resposta de um item de consulta em lote.
#[derive(Debug, Serialize)]
pub struct RespostaItemConsultaLote {
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

impl From<crate::domain::trabalhos::entidades::TrabalhoAgendado> for RespostaTrabalhoAgendado {
    fn from(t: crate::domain::trabalhos::entidades::TrabalhoAgendado) -> Self {
        Self {
            id: t.id,
            escritorio_id: t.escritorio_id,
            nome: t.nome,
            tipo: t.tipo.as_str().to_owned(),
            produto: t.produto,
            ambiente: t.ambiente,
            cron_expression: t.cron_expression,
            habilitado: t.habilitado,
            proxima_execucao: t.proxima_execucao,
            ultima_execucao: t.ultima_execucao,
            criado_em: t.criado_em,
            atualizado_em: t.atualizado_em,
        }
    }
}

impl From<crate::domain::trabalhos::entidades::ExecucaoTrabalho> for RespostaExecucaoTrabalho {
    fn from(e: crate::domain::trabalhos::entidades::ExecucaoTrabalho) -> Self {
        Self {
            id: e.id,
            trabalho_agendado_id: e.trabalho_agendado_id,
            escritorio_id: e.escritorio_id,
            status: e.status.as_str().to_owned(),
            iniciado_em: e.iniciado_em,
            finalizado_em: e.finalizado_em,
            erro_mensagem: e.erro_mensagem,
            registros_processados: e.registros_processados,
            registros_com_erro: e.registros_com_erro,
            criado_em: e.criado_em,
        }
    }
}

impl From<crate::domain::trabalhos::entidades::ConsultaLote> for RespostaConsultaLote {
    fn from(l: crate::domain::trabalhos::entidades::ConsultaLote) -> Self {
        Self {
            id: l.id,
            escritorio_id: l.escritorio_id,
            nome: l.nome,
            produto: l.produto,
            servico: l.servico,
            verbo: l.verbo,
            status: l.status.as_str().to_owned(),
            total_itens: l.total_itens,
            itens_concluidos: l.itens_concluidos,
            itens_com_erro: l.itens_com_erro,
            criado_em: l.criado_em,
            iniciado_em: l.iniciado_em,
            finalizado_em: l.finalizado_em,
        }
    }
}

impl From<crate::domain::trabalhos::entidades::ItemConsultaLote> for RespostaItemConsultaLote {
    fn from(i: crate::domain::trabalhos::entidades::ItemConsultaLote) -> Self {
        Self {
            id: i.id,
            consulta_lote_id: i.consulta_lote_id,
            documento: i.documento,
            tipo: i.tipo,
            status: i.status,
            resultado_redigido: i.resultado_redigido,
            erro: i.erro,
            criado_em: i.criado_em,
            processado_em: i.processado_em,
        }
    }
}
