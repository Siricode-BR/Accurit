//! Portas do domínio de emissão de guias.
//! Implementadas pela infrastructure (Diesel, SERPRO, armazenamento de PDF).

use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::guias::entidades::{GuiaEmissao, NovoGuiaEmissao};
use crate::domain::guias::enums::{StatusGuia, TipoGuia};
use crate::domain::guias::erros::ErroGuia;

/// Repositório de emissões de guia.
#[async_trait]
pub trait RepositorioGuiasEmissao: Send + Sync {
    async fn criar(&self, guia: &NovoGuiaEmissao) -> Result<GuiaEmissao, ErroGuia>;
    async fn buscar_por_id(&self, id: Uuid) -> Result<Option<GuiaEmissao>, ErroGuia>;
    async fn buscar_por_idempotencia(&self, chave: &str) -> Result<Option<GuiaEmissao>, ErroGuia>;
    async fn listar_por_usuario(
        &self,
        usuario_id: Uuid,
        limite: i64,
    ) -> Result<Vec<GuiaEmissao>, ErroGuia>;
    async fn listar_por_escritorio(
        &self,
        escritorio_id: Uuid,
        limite: i64,
    ) -> Result<Vec<GuiaEmissao>, ErroGuia>;
    async fn listar_por_cliente(
        &self,
        cliente_id: Uuid,
        limite: i64,
    ) -> Result<Vec<GuiaEmissao>, ErroGuia>;
    async fn atualizar_status(
        &self,
        id: Uuid,
        status: StatusGuia,
        pdf_url: Option<String>,
    ) -> Result<(), ErroGuia>;
}

/// Provedor de emissão de guia (SERPRO Integra Contador).
#[async_trait]
pub trait ProvedorEmissaoGuia: Send + Sync {
    async fn emitir_guia(
        &self,
        cliente_cpf_cnpj: &str,
        tipo_guia: TipoGuia,
        competencia: &str,
    ) -> Result<(String, Option<String>), ErroGuia>;
}
