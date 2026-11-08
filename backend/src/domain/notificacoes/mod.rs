//! Fase C — Alertas e notificações.
//! Domínio de notificações in-app, e-mail e WhatsApp, com preferências
//! por usuário e gatilhos automáticos de rotina fiscal.

pub mod entidades;
pub mod enums;
pub mod erros;
pub mod ports;

pub use entidades::{
    Notificacao, NovoNotificacao, PreferenciaNotificacao, NovoPreferenciaNotificacao,
    TemplateNotificacao, NovoTemplateNotificacao,
};
pub use enums::{CanalNotificacao, TipoNotificacao, StatusNotificacao};
pub use erros::ErroNotificacao;
pub use ports::{
    RepositorioNotificacoes, RepositorioPreferenciasNotificacao,
    RepositorioTemplatesNotificacao, ProvedorEnvioNotificacao, SchedulerNotificacao,
};
