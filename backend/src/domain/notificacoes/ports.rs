//! Portas do domínio de notificações.
//! Implementadas pela infrastructure (Diesel, SMTP, WhatsApp gateway, etc.).

use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::notificacoes::entidades::{
    Notificacao, NovoNotificacao, NovoPreferenciaNotificacao, PreferenciaNotificacao,
    TemplateNotificacao,
};
use crate::domain::notificacoes::enums::{CanalNotificacao, TipoNotificacao};
use crate::domain::notificacoes::erros::ErroNotificacao;

/// Repositório de notificações.
#[async_trait]
pub trait RepositorioNotificacoes: Send + Sync {
    async fn criar(&self, notificacao: &NovoNotificacao) -> Result<Notificacao, ErroNotificacao>;
    async fn buscar_por_id(&self, id: Uuid) -> Result<Option<Notificacao>, ErroNotificacao>;
    async fn listar_por_usuario(
        &self,
        usuario_id: Uuid,
        apenas_nao_lidas: bool,
        limite: i64,
    ) -> Result<Vec<Notificacao>, ErroNotificacao>;
    async fn listar_por_escritorio(
        &self,
        escritorio_id: Uuid,
        apenas_nao_lidas: bool,
        limite: i64,
    ) -> Result<Vec<Notificacao>, ErroNotificacao>;
    async fn marcar_como_lida(&self, id: Uuid, usuario_id: Uuid) -> Result<(), ErroNotificacao>;
    async fn marcar_todas_como_lidas(&self, usuario_id: Uuid) -> Result<usize, ErroNotificacao>;
    async fn contar_nao_lidas(&self, usuario_id: Uuid) -> Result<i64, ErroNotificacao>;
}

/// Repositório de preferências de notificação.
#[async_trait]
pub trait RepositorioPreferenciasNotificacao: Send + Sync {
    async fn buscar_por_usuario(
        &self,
        usuario_id: Uuid,
    ) -> Result<Option<PreferenciaNotificacao>, ErroNotificacao>;
    async fn criar_ou_atualizar(
        &self,
        preferencia: &NovoPreferenciaNotificacao,
    ) -> Result<PreferenciaNotificacao, ErroNotificacao>;
}

/// Repositório de templates de notificação.
#[async_trait]
pub trait RepositorioTemplatesNotificacao: Send + Sync {
    async fn buscar_por_tipo(
        &self,
        tipo: TipoNotificacao,
    ) -> Result<Option<TemplateNotificacao>, ErroNotificacao>;
    async fn listar_ativos(&self) -> Result<Vec<TemplateNotificacao>, ErroNotificacao>;
}

/// Provedor de envio de notificação (e-mail SMTP, WhatsApp gateway, in-app).
#[async_trait]
pub trait ProvedorEnvioNotificacao: Send + Sync {
    async fn enviar(
        &self,
        canal: CanalNotificacao,
        destino: &str,
        assunto: &str,
        corpo: &str,
    ) -> Result<(), ErroNotificacao>;
}

/// Scheduler de notificações: verifica gatilhos e gera notificações.
pub trait SchedulerNotificacao: Send + Sync {
    fn verificar_pendencias_divida_ativa(&self) -> Vec<NovoNotificacao>;
    fn verificar_procuracoes_vencendo(&self, dias: i64) -> Vec<NovoNotificacao>;
    fn verificar_certificados_vencendo(&self, dias: i64) -> Vec<NovoNotificacao>;
    fn verificar_das_vencendo(&self, dias: i64) -> Vec<NovoNotificacao>;
    fn verificar_falhas_rotina(&self) -> Vec<NovoNotificacao>;
}
