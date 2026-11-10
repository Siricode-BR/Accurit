//! Fase F — Entidades de domínio de relatórios e links de compartilhamento.

use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::domain::relatorios::enums::{FormatoRelatorio, StatusRelatorio, TipoRelatorio};

/// Relatório gerado por um usuário do escritório.
#[derive(Debug, Clone)]
pub struct Relatorio {
    pub id: Uuid,
    pub escritorio_id: Uuid,
    pub usuario_id: Uuid,
    pub cliente_id: Option<Uuid>,
    pub tipo_relatorio: TipoRelatorio,
    pub formato: FormatoRelatorio,
    pub status: StatusRelatorio,
    pub conteudo: Option<Vec<u8>>,
    pub nome_arquivo: Option<String>,
    pub competencia: Option<String>,
    pub expira_em: DateTime<Utc>,
    pub criado_em: DateTime<Utc>,
    pub atualizado_em: DateTime<Utc>,
}

/// Dados para criação de um novo relatório.
#[derive(Debug, Clone)]
pub struct NovoRelatorio {
    pub escritorio_id: Uuid,
    pub usuario_id: Uuid,
    pub cliente_id: Option<Uuid>,
    pub tipo_relatorio: TipoRelatorio,
    pub formato: FormatoRelatorio,
    pub competencia: Option<String>,
}

/// Link seguro e expirável para compartilhamento de um relatório.
#[derive(Debug, Clone)]
pub struct LinkCompartilhamento {
    pub id: Uuid,
    pub relatorio_id: Uuid,
    pub token_hash: String,
    pub expira_em: DateTime<Utc>,
    pub max_acessos: i32,
    pub acessos: i32,
    pub revogado_em: Option<DateTime<Utc>>,
    pub acessado_em: Option<DateTime<Utc>>,
    pub criado_em: DateTime<Utc>,
}

/// Dados para criação de um novo link de compartilhamento. O token opaco é
/// gerado no caso de uso; o banco guarda apenas o hash SHA-256.
#[derive(Debug, Clone)]
pub struct NovoLinkCompartilhamento {
    pub relatorio_id: Uuid,
    pub token_hash: String,
    pub expira_em: DateTime<Utc>,
    pub max_acessos: i32,
}

/// Hash SHA-256 do token opaco. O valor em claro nunca é persistido.
pub fn hash_token(token: &str) -> String {
    let digest = Sha256::digest(token.as_bytes());
    hex::encode(digest)
}
