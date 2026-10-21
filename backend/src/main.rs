use std::process::ExitCode;

use accurit_backend::infrastructure::config::Config;
use accurit_backend::infrastructure::observability;
use accurit_backend::startup;

#[tokio::main]
async fn main() -> ExitCode {
    let config = match Config::from_env() {
        Ok(config) => config,
        Err(erro) => {
            eprintln!("erro de configuração: {erro}");
            return ExitCode::FAILURE;
        }
    };

    if let Err(erro) = observability::init(&config) {
        eprintln!("erro ao iniciar logs: {erro}");
        return ExitCode::FAILURE;
    }

    tracing::info!(
        ambiente = %config.app_env,
        porta = config.port,
        "iniciando accurit-backend"
    );

    match startup::run(config).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(erro) => {
            tracing::error!(erro = %erro, "falha ao iniciar ou executar o servidor");
            ExitCode::FAILURE
        }
    }
}
