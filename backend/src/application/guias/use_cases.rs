//! Fase D — Casos de uso de emissão de guias.

use std::sync::Arc;

use uuid::Uuid;

use crate::domain::guias::enums::StatusGuia;
use crate::domain::guias::erros::ErroGuia;
use crate::domain::guias::entidades::{GuiaEmissao, NovoGuiaEmissao};
use crate::domain::guias::ports::{
    RepositorioGuiasEmissao, ProvedorEmissaoGuia,
};

/// Emitir uma guia (DAS ou DARF) com idempotência.
pub struct EmitirGuiaUseCase {
    repositorio: Arc<dyn RepositorioGuiasEmissao>,
    provedor: Arc<dyn ProvedorEmissaoGuia>,
}

impl EmitirGuiaUseCase {
    pub fn novo(
        repositorio: Arc<dyn RepositorioGuiasEmissao>,
        provedor: Arc<dyn ProvedorEmissaoGuia>,
    ) -> Self {
        Self { repositorio, provedor }
    }

    pub async fn executar(
        &self,
        comando: NovoGuiaEmissao,
    ) -> Result<GuiaEmissao, ErroGuia> {
        // Verificar idempotência: já existe uma emissão com esta chave?
        if let Some(existente) = self
            .repositorio
            .buscar_por_idempotencia(&comando.idempotencia_chave)
            .await?
        {
            return Ok(existente);
        }

        // Chamar o provedor de emissão (SERPRO Integra Contador)
        let (pdf_url, _) = self
            .provedor
            .emitir_guia(
                "", // cliente_cpf_cnpj seria obtido do cliente
                comando.tipo_guia,
                &comando.competencia,
            )
            .await?;

        // Criar o registro de emissão com pdf_url preenchido
        let mut guia = self.repositorio.criar(&comando).await?;
        guia.pdf_url = Some(pdf_url);
        self.repositorio
            .atualizar_status(guia.id, guia.status, guia.pdf_url.clone())
            .await?;
        Ok(guia)
    }
}

/// Listar guias de um usuário.
pub struct ListarGuiasUseCase {
    repositorio: Arc<dyn RepositorioGuiasEmissao>,
}

impl ListarGuiasUseCase {
    pub fn novo(repositorio: Arc<dyn RepositorioGuiasEmissao>) -> Self {
        Self { repositorio }
    }

    pub async fn executar(
        &self,
        usuario_id: Uuid,
        limite: i64,
    ) -> Result<Vec<GuiaEmissao>, ErroGuia> {
        self.repositorio.listar_por_usuario(usuario_id, limite).await
    }
}

/// Buscar uma guia por ID.
pub struct BuscarGuiaUseCase {
    repositorio: Arc<dyn RepositorioGuiasEmissao>,
}

impl BuscarGuiaUseCase {
    pub fn novo(repositorio: Arc<dyn RepositorioGuiasEmissao>) -> Self {
        Self { repositorio }
    }

    pub async fn executar(&self, id: Uuid) -> Result<Option<GuiaEmissao>, ErroGuia> {
        self.repositorio.buscar_por_id(id).await
    }
}

/// Baixar uma guia (marcar como baixada).
pub struct BaixarGuiaUseCase {
    repositorio: Arc<dyn RepositorioGuiasEmissao>,
}

impl BaixarGuiaUseCase {
    pub fn novo(repositorio: Arc<dyn RepositorioGuiasEmissao>) -> Self {
        Self { repositorio }
    }

    pub async fn executar(
        &self,
        id: Uuid,
    ) -> Result<GuiaEmissao, ErroGuia> {
        let guia = self
            .repositorio
            .buscar_por_id(id)
            .await?
            .ok_or(ErroGuia::GuiaNaoEncontrada)?;

        if guia.status != StatusGuia::Emitida {
            return Err(ErroGuia::NaoPodeBaixar(guia.status.as_str().to_owned()));
        }

        self.repositorio
            .atualizar_status(id, StatusGuia::Baixada, guia.pdf_url)
            .await?;

        self.repositorio
            .buscar_por_id(id)
            .await?
            .ok_or(ErroGuia::GuiaNaoEncontrada)
    }
}
