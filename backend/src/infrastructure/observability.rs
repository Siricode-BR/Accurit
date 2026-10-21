use thiserror::Error;
use tracing_subscriber::EnvFilter;

use super::config::Config;

#[derive(Debug, Error)]
#[error("não foi possível inicializar o tracing")]
pub struct ObservabilityError;

pub fn init(config: &Config) -> Result<(), ObservabilityError> {
    let filtro = EnvFilter::try_new(&config.log_filter).unwrap_or_else(|_| EnvFilter::new("info"));

    if config.app_env.usa_log_json() {
        tracing_subscriber::fmt()
            .with_env_filter(filtro)
            .json()
            .with_current_span(true)
            .with_span_list(false)
            .try_init()
            .map_err(|_| ObservabilityError)
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(filtro)
            .compact()
            .with_target(false)
            .try_init()
            .map_err(|_| ObservabilityError)
    }
}
