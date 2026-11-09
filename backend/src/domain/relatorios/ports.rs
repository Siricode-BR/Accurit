//! Fase F — Portas do domínio de relatórios.
//! Implementadas pela infrastructure (Diesel).

use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::relatorios::enums::StatusRelatorio;
use crate::domain::relatorios::entidades::{
    LinkCompartilhamento, NovoLinkCompartilhamento, NovoRelatorio, Relatorio,
};
use crate::domain::relatorios::erros::ErroRelatorio;

/// Repositório de relatórios gerados.
#[async_trait]
pub trait RepositorioRelatorios: Send + Sync {
    async fn criar(&self, relatorio: &NovoRelatorio) -> Result<Relatorio, ErroRelatorio>;
    async fn buscar_por_id(&self, id: Uuid) -> Result<Option<Relatorio>, ErroRelatorio>;
    async fn listar_por_escritorio(
        &self,
        escritorio_id: Uuid,
        limite: i64,
    ) -> Result<Vec<Relatorio>, ErroRelatorio>;
    async fn atualizar_status_e_conteudo(
        &self,
        id: Uuid,
        status: StatusRelatorio,
        conteudo: Option<Vec<u8>>,
        nome_arquivo: Option<String>,
    ) -> Result<(), ErroRelatorio>;
}

/// Repositório de links de compartilhamento de relatórios.
#[async_trait]
pub trait RepositorioLinksCompartilhamento: Send + Sync {
    async fn criar(
        &self,
        link: &NovoLinkCompartilhamento,
    ) -> Result<LinkCompartilhamento, ErroRelatorio>;
    async fn buscar_por_token(
        &self,
        token: &str,
    ) -> Result<Option<LinkCompartilhamento>, ErroRelatorio>;
    async fn registrar_acesso(
        &self,
        id: Uuid,
    ) -> Result<(), ErroRelatorio>;
    async fn listar_expirados(
        &self,
        agora: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<LinkCompartilhamento>, ErroRelatorio>;
}
