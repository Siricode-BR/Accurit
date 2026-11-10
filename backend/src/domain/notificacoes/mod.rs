//! Fase C — Alertas e notificações.
//! Domínio de notificações in-app, e-mail e WhatsApp, com preferências
//! por usuário e gatilhos automáticos de rotina fiscal.

pub mod entidades;
pub mod enums;
pub mod erros;
pub mod ports;

pub use entidades::{
    Notificacao, NovoNotificacao, NovoPreferenciaNotificacao, NovoTemplateNotificacao,
    PreferenciaNotificacao, TemplateNotificacao,
};
pub use enums::{CanalNotificacao, StatusNotificacao, TipoNotificacao};
pub use erros::ErroNotificacao;
pub use ports::{
    ProvedorEnvioNotificacao, RepositorioNotificacoes, RepositorioPreferenciasNotificacao,
    RepositorioTemplatesNotificacao, SchedulerNotificacao,
};
