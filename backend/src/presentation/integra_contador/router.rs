use axum::Router;
use axum::middleware as axum_middleware;
use axum::routing::{get, post};

use crate::presentation::auth::{ExigirAudiencia, exigir};
use crate::presentation::integra_contador::handlers;
use crate::presentation::integra_contador::state::EstadoIntegraContador;
use crate::presentation::serpro::ambiente::{EstadoAmbiente, resolver_ambiente};

pub fn construir(
    estado: EstadoIntegraContador,
    exigir_audiencia: ExigirAudiencia,
    estado_ambiente: EstadoAmbiente,
) -> Router {
    Router::new()
        .route("/status", get(handlers::status))
        .route("/executar", post(handlers::executar))
        .route(
            "/procuracoes/consultar",
            post(handlers::consultar_procuracoes),
        )
        .route_layer(axum_middleware::from_fn_with_state(
            estado_ambiente,
            resolver_ambiente,
        ))
        .route_layer(axum_middleware::from_fn_with_state(
            exigir_audiencia,
            exigir,
        ))
        .with_state(estado)
}
