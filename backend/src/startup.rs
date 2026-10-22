use secrecy::ExposeSecret;
use thiserror::Error;
use tokio::net::TcpListener;

use crate::application::saude::ConsultarSaude;
use crate::infrastructure::config::Config;
use crate::infrastructure::persistence::migrations::{
    MIGRATIONS, MigrationError, aplicar_migrations_com_lock, conectar_com_retry,
};
use crate::infrastructure::persistence::pool::{PoolIndisponivel, criar_pool};
use crate::infrastructure::persistence::saude::PostgresSaudeBanco;
use crate::presentation::auth::{AuthInitError, AuthState};
use crate::presentation::routes;
use crate::presentation::state::AppState;

#[derive(Debug, Error)]
pub enum StartupError {
    #[error(transparent)]
    Migracao(#[from] MigrationError),
    #[error(transparent)]
    Pool(#[from] PoolIndisponivel),
    #[error(transparent)]
    Auth(#[from] AuthInitError),
    #[error("não foi possível associar a porta {0}")]
    Bind(u16),
    #[error("falha no servidor HTTP")]
    Servidor,
    #[error("tarefa de inicialização falhou")]
    Tarefa,
}

pub async fn run(config: Config) -> Result<(), StartupError> {
    let database_url = config.database.url.expose_secret().to_owned();

    let aplicadas = tokio::task::spawn_blocking(move || {
        let conexao = conectar_com_retry(&database_url)?;
        aplicar_migrations_com_lock(conexao, MIGRATIONS)
    })
    .await
    .map_err(|_| StartupError::Tarefa)??;

    tracing::info!(migracoes = aplicadas, "migrações aplicadas");

    let database_url = config.database.url.expose_secret().to_owned();
    let pool = tokio::task::spawn_blocking(move || criar_pool(&database_url))
        .await
        .map_err(|_| StartupError::Tarefa)??;

    let auth = AuthState::novo(pool.clone(), &config)?;
    let estado = AppState::novo(ConsultarSaude::novo(PostgresSaudeBanco::novo(pool.clone())), pool);
    let aplicacao = routes::construir(estado, Some(auth), &config);

    let listener = TcpListener::bind(("0.0.0.0", config.port))
        .await
        .map_err(|_| StartupError::Bind(config.port))?;

    let endereco = listener
        .local_addr()
        .map(|endereco| endereco.to_string())
        .unwrap_or_else(|_| format!("0.0.0.0:{}", config.port));

    tracing::info!(endereco, "servidor HTTP iniciado");

    axum::serve(listener, aplicacao)
        .with_graceful_shutdown(desligamento())
        .await
        .map_err(|_| StartupError::Servidor)
}

async fn desligamento() {
    let ctrl_c = async {
        if tokio::signal::ctrl_c().await.is_err() {
            tracing::error!("não foi possível registrar o sinal de interrupção");
            std::future::pending::<()>().await;
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut sinal) => {
                sinal.recv().await;
            }
            Err(_) => {
                tracing::error!("não foi possível registrar o sinal de término");
                std::future::pending::<()>().await;
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("sinal de desligamento recebido; encerrando");
}
