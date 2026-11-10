//! Fase F — DTOs de relatórios e link de compartilhamento.

use base64::Engine;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::application::relatorios::use_cases::{ComandoCriarLink, ComandoGerarRelatorio};
use crate::domain::relatorios::enums::{FormatoRelatorio, TipoRelatorio};

/// Requisição para gerar um relatório.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CriarRelatorioRequest {
    pub tipo_relatorio: String, // "situacao_fiscal" | "historico_consultas" | "pendencias_carteira"
    pub formato: String,        // "PDF" | "XLSX"
    pub cliente_id: Option<Uuid>,
    pub competencia: Option<String>, // "YYYY-MM"
}

/// Resumo de um relatório para listagem.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelatorioResumoDto {
    pub id: Uuid,
    pub tipo_relatorio: String,
    pub formato: String,
    pub status: String,
    pub nome_arquivo: Option<String>,
    pub expira_em: DateTime<Utc>,
    pub criado_em: DateTime<Utc>,
}

/// Detalhe completo de um relatório (inclui conteúdo base64 quando concluído).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelatorioDto {
    pub id: Uuid,
    pub tipo_relatorio: String,
    pub formato: String,
    pub status: String,
    pub conteudo_base64: Option<String>,
    pub nome_arquivo: Option<String>,
    pub expira_em: DateTime<Utc>,
    pub criado_em: DateTime<Utc>,
    pub atualizado_em: DateTime<Utc>,
}

/// Requisição para criar um link de compartilhamento.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CriarLinkCompartilhamentoRequest {
    pub expira_em: DateTime<Utc>,
    #[serde(default)]
    pub max_acessos: Option<i32>,
}

/// DTO de um link de compartilhamento.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkCompartilhamentoDto {
    pub id: Uuid,
    pub relatorio_id: Uuid,
    pub token: String,
    pub expira_em: DateTime<Utc>,
    pub max_acessos: i32,
    pub acessos: i32,
    pub revogado_em: Option<DateTime<Utc>>,
    pub acessado_em: Option<DateTime<Utc>>,
    pub criado_em: DateTime<Utc>,
}

impl From<crate::domain::relatorios::entidades::Relatorio> for RelatorioDto {
    fn from(r: crate::domain::relatorios::entidades::Relatorio) -> Self {
        Self {
            id: r.id,
            tipo_relatorio: r.tipo_relatorio.as_str().to_owned(),
            formato: r.formato.as_str().to_owned(),
            status: r.status.as_str().to_owned(),
            conteudo_base64: r
                .conteudo
                .map(|c| base64::engine::general_purpose::STANDARD.encode(&c)),
            nome_arquivo: r.nome_arquivo,
            expira_em: r.expira_em,
            criado_em: r.criado_em,
            atualizado_em: r.atualizado_em,
        }
    }
}

impl LinkCompartilhamentoDto {
    /// Monta o DTO devolvendo o token em claro **apenas nesta resposta** de criação.
    pub fn de_criacao(
        l: crate::domain::relatorios::entidades::LinkCompartilhamento,
        token: String,
    ) -> Self {
        Self {
            id: l.id,
            relatorio_id: l.relatorio_id,
            token,
            expira_em: l.expira_em,
            max_acessos: l.max_acessos,
            acessos: l.acessos,
            revogado_em: l.revogado_em,
            acessado_em: l.acessado_em,
            criado_em: l.criado_em,
        }
    }
}

impl From<crate::domain::relatorios::entidades::LinkCompartilhamento> for LinkCompartilhamentoDto {
    fn from(l: crate::domain::relatorios::entidades::LinkCompartilhamento) -> Self {
        Self {
            id: l.id,
            relatorio_id: l.relatorio_id,
            // O token em claro não é recuperável do banco (apenas hash).
            token: String::new(),
            expira_em: l.expira_em,
            max_acessos: l.max_acessos,
            acessos: l.acessos,
            revogado_em: l.revogado_em,
            acessado_em: l.acessado_em,
            criado_em: l.criado_em,
        }
    }
}

impl TryFrom<CriarRelatorioRequest> for ComandoGerarRelatorio {
    type Error = String;

    fn try_from(req: CriarRelatorioRequest) -> Result<Self, Self::Error> {
        let tipo_relatorio = match req.tipo_relatorio.as_str() {
            "situacao_fiscal" => TipoRelatorio::SituacaoFiscal,
            "historico_consultas" => TipoRelatorio::HistoricoConsultas,
            "pendencias_carteira" => TipoRelatorio::PendenciasCarteira,
            outro => return Err(format!("tipo de relatório inválido: {outro}")),
        };
        let formato = match req.formato.as_str() {
            "PDF" => FormatoRelatorio::Pdf,
            "XLSX" => FormatoRelatorio::Xlsx,
            outro => return Err(formato_invalido(outro)),
        };
        Ok(Self {
            tipo_relatorio,
            formato,
            cliente_id: req.cliente_id,
            competencia: req.competencia,
            apenas_ids: None,
            ..Default::default()
        })
    }
}

impl TryFrom<CriarLinkCompartilhamentoRequest> for ComandoCriarLink {
    type Error = String;

    fn try_from(req: CriarLinkCompartilhamentoRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            expira_em: req.expira_em,
            max_acessos: req.max_acessos.unwrap_or(50).clamp(1, 1000),
            ..Default::default()
        })
    }
}

fn formato_invalido(valor: &str) -> String {
    format!("formato de exportação inválido: {valor}")
}

impl Default for ComandoGerarRelatorio {
    fn default() -> Self {
        Self {
            escritorio_id: Uuid::nil(),
            usuario_id: Uuid::nil(),
            cliente_id: None,
            tipo_relatorio: TipoRelatorio::SituacaoFiscal,
            formato: FormatoRelatorio::Pdf,
            competencia: None,
            apenas_ids: None,
        }
    }
}

impl Default for ComandoCriarLink {
    fn default() -> Self {
        Self {
            relatorio_id: Uuid::nil(),
            escritorio_id: Uuid::nil(),
            expira_em: Utc::now() + chrono::Duration::days(7),
            max_acessos: 50,
        }
    }
}
