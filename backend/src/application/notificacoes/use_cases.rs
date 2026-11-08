//! Fase C — Casos de uso de notificações.

use std::sync::Arc;

use uuid::Uuid;

use crate::domain::notificacoes::erros::ErroNotificacao;
use crate::domain::notificacoes::entidades::{
    Notificacao, NovoNotificacao, NovoPreferenciaNotificacao, PreferenciaNotificacao,
    TemplateNotificacao,
};
use crate::domain::notificacoes::ports::{
    RepositorioNotificacoes, RepositorioPreferenciasNotificacao,
    RepositorioTemplatesNotificacao,
};

/// Criar uma notificação.
pub struct CriarNotificacaoUseCase {
    repositorio: Arc<dyn RepositorioNotificacoes>,
}

impl CriarNotificacaoUseCase {
    pub fn novo(repositorio: Arc<dyn RepositorioNotificacoes>) -> Self {
        Self { repositorio }
    }

    pub async fn executar(
        &self,
        comando: NovoNotificacao,
    ) -> Result<Notificacao, ErroNotificacao> {
        self.repositorio.criar(&comando).await
    }
}

/// Listar notificações de um usuário.
pub struct ListarNotificacoesUseCase {
    repositorio: Arc<dyn RepositorioNotificacoes>,
}

impl ListarNotificacoesUseCase {
    pub fn novo(repositorio: Arc<dyn RepositorioNotificacoes>) -> Self {
        Self { repositorio }
    }

    pub async fn executar(
        &self,
        usuario_id: Uuid,
        apenas_nao_lidas: bool,
        limite: i64,
    ) -> Result<Vec<Notificacao>, ErroNotificacao> {
        self.repositorio
            .listar_por_usuario(usuario_id, apenas_nao_lidas, limite)
            .await
    }
}

/// Marcar uma notificação como lida.
pub struct MarcarNotificacaoLidaUseCase {
    repositorio: Arc<dyn RepositorioNotificacoes>,
}

impl MarcarNotificacaoLidaUseCase {
    pub fn novo(repositorio: Arc<dyn RepositorioNotificacoes>) -> Self {
        Self { repositorio }
    }

    pub async fn executar(
        &self,
        id: Uuid,
        usuario_id: Uuid,
    ) -> Result<(), ErroNotificacao> {
        self.repositorio.marcar_como_lida(id, usuario_id).await
    }
}

/// Marcar todas as notificações de um usuário como lidas.
pub struct MarcarTodasNotificacoesLidasUseCase {
    repositorio: Arc<dyn RepositorioNotificacoes>,
}

impl MarcarTodasNotificacoesLidasUseCase {
    pub fn novo(repositorio: Arc<dyn RepositorioNotificacoes>) -> Self {
        Self { repositorio }
    }

    pub async fn executar(
        &self,
        usuario_id: Uuid,
    ) -> Result<usize, ErroNotificacao> {
        self.repositorio.marcar_todas_como_lidas(usuario_id).await
    }
}

/// Contar notificações não lidas de um usuário.
pub struct ContarNotificacoesNaoLidasUseCase {
    repositorio: Arc<dyn RepositorioNotificacoes>,
}

impl ContarNotificacoesNaoLidasUseCase {
    pub fn novo(repositorio: Arc<dyn RepositorioNotificacoes>) -> Self {
        Self { repositorio }
    }

    pub async fn executar(&self, usuario_id: Uuid) -> Result<i64, ErroNotificacao> {
        self.repositorio.contar_nao_lidas(usuario_id).await
    }
}

/// Criar ou atualizar preferências de notificação.
pub struct CriarOuAtualizarPreferenciaUseCase {
    repositorio: Arc<dyn RepositorioPreferenciasNotificacao>,
}

impl CriarOuAtualizarPreferenciaUseCase {
    pub fn novo(repositorio: Arc<dyn RepositorioPreferenciasNotificacao>) -> Self {
        Self { repositorio }
    }

    pub async fn executar(
        &self,
        comando: NovoPreferenciaNotificacao,
    ) -> Result<PreferenciaNotificacao, ErroNotificacao> {
        self.repositorio.criar_ou_atualizar(&comando).await
    }
}

/// Buscar preferências de notificação de um usuário.
pub struct BuscarPreferenciaUseCase {
    repositorio: Arc<dyn RepositorioPreferenciasNotificacao>,
}

impl BuscarPreferenciaUseCase {
    pub fn novo(repositorio: Arc<dyn RepositorioPreferenciasNotificacao>) -> Self {
        Self { repositorio }
    }

    pub async fn executar(
        &self,
        usuario_id: Uuid,
    ) -> Result<Option<PreferenciaNotificacao>, ErroNotificacao> {
        self.repositorio.buscar_por_usuario(usuario_id).await
    }
}

/// Listar templates de notificação ativos.
pub struct ListarTemplatesNotificacaoUseCase {
    repositorio: Arc<dyn RepositorioTemplatesNotificacao>,
}

impl ListarTemplatesNotificacaoUseCase {
    pub fn novo(repositorio: Arc<dyn RepositorioTemplatesNotificacao>) -> Self {
        Self { repositorio }
    }

    pub async fn executar(&self) -> Result<Vec<TemplateNotificacao>, ErroNotificacao> {
        self.repositorio.listar_ativos().await
    }
}
