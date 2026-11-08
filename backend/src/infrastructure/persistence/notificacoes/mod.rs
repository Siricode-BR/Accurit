//! Fase C — Repositórios Diesel para notificações, preferências e templates.

pub mod repositorio_notificacoes;
pub mod repositorio_preferencias_notificacao;
pub mod repositorio_templates_notificacao;

pub use repositorio_notificacoes::NotificacoesDiesel;
pub use repositorio_preferencias_notificacao::PreferenciasNotificacaoDiesel;
pub use repositorio_templates_notificacao::TemplatesNotificacaoDiesel;
