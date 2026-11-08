//! Entidades de domínio de emissão de guias.
//! São objetos puros de negócio, sem dependência de Diesel ou Axum.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::guias::enums::{StatusGuia, TipoGuia};

/// Guia emitida (DAS ou DARF) para um cliente.
#[derive(Debug, Clone)]
pub struct GuiaEmissao {
    pub id: Uuid,
    pub usuario_id: Uuid,
    pub escritorio_id: Uuid,
    pub cliente_id: Uuid,
    pub tipo_guia: TipoGuia,
    pub competencia: String, // "YYYY-MM"
    pub valor_total: f64,
    pub pdf_url: Option<String>,
    pub status: StatusGuia,
    pub idempotencia_chave: String,
    pub criado_em: DateTime<Utc>,
    pub atualizado_em: DateTime<Utc>,
}

/// Dados para criação de uma nova emissão de guia.
#[derive(Debug, Clone)]
pub struct NovoGuiaEmissao {
    pub usuario_id: Uuid,
    pub escritorio_id: Uuid,
    pub cliente_id: Uuid,
    pub tipo_guia: TipoGuia,
    pub competencia: String,
    pub valor_total: f64,
    pub idempotencia_chave: String,
}
