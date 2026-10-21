use std::process::ExitCode;

use accurit_backend::bootstrap;
use accurit_backend::infrastructure::config::Config;
use accurit_backend::infrastructure::observability;
use accurit_backend::startup;

#[tokio::main]
async fn main() -> ExitCode {
    let mut argumentos: Vec<String> = std::env::args().skip(1).collect();

    let comando = argumentos.first().map(String::as_str).map(str::to_owned);
    if comando.as_deref() == Some("bootstrap-user") {
        argumentos.remove(0);
        return executar_bootstrap(&argumentos).await;
    }

    executar_servidor().await
}

async fn executar_bootstrap(argumentos: &[String]) -> ExitCode {
    if argumentos.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("{}", bootstrap::AJUDA);
        return ExitCode::SUCCESS;
    }

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

    match bootstrap::executar(argumentos).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(erro) => {
            eprintln!("erro no bootstrap-user: {erro}");
            ExitCode::FAILURE
        }
    }
}

async fn executar_servidor() -> ExitCode {
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
