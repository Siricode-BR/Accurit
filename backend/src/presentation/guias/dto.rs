//! Fase D — DTOs de emissão de guias.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::guias::entidades::GuiaEmissao;
use crate::domain::guias::enums::{StatusGuia, TipoGuia};

/// Requisição para emitir uma guia.
#[derive(Debug, Deserialize)]
pub struct RequisicaoEmitirGuia {
    pub cliente_id: Uuid,
    pub tipo_guia: String, // "DAS" | "DARF"
    pub competencia: String, // "YYYY-MM"
    pub valor_total: f64,
    pub idempotencia_chave: String,
}

/// Resposta de uma guia emitida.
#[derive(Debug, Serialize)]
pub struct RespostaGuiaEmissao {
    pub id: Uuid,
    pub usuario_id: Uuid,
    pub escritorio_id: Uuid,
    pub cliente_id: Uuid,
    pub tipo_guia: String,
    pub competencia: String,
    pub valor_total: f64,
    pub pdf_url: Option<String>,
    pub status: String,
    pub idempotencia_chave: String,
    pub criado_em: DateTime<Utc>,
    pub atualizado_em: DateTime<Utc>,
}

impl From<GuiaEmissao> for RespostaGuiaEmissao {
    fn from(g: GuiaEmissao) -> Self {
        Self {
            id: g.id,
            usuario_id: g.usuario_id,
            escritorio_id: g.escritorio_id,
            cliente_id: g.cliente_id,
            tipo_guia: g.tipo_guia.as_str().to_owned(),
            competencia: g.competencia,
            valor_total: g.valor_total,
            pdf_url: g.pdf_url,
            status: g.status.as_str().to_owned(),
            idempotencia_chave: g.idempotencia_chave,
            criado_em: g.criado_em,
            atualizado_em: g.atualizado_em,
        }
    }
}
