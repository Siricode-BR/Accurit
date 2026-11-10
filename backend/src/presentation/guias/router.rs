//! Fase D — Rotas de emissão de guias.

use axum::Router;
use axum::middleware as axum_middleware;
use axum::routing::{get, post};

use crate::presentation::auth::{ExigirAudiencia, exigir};
use crate::presentation::guias::handlers::{baixar_guia, buscar_guia, emitir_guia, listar_guias};
use crate::presentation::guias::state::EstadoGuias;

pub fn construir(estado: EstadoGuias, exigir_audiencia: ExigirAudiencia) -> Router {
    Router::new()
        .route("/guias", post(emitir_guia).get(listar_guias))
        .route("/guias/{id}", get(buscar_guia))
        .route("/guias/{id}/baixar", post(baixar_guia))
        .route_layer(axum_middleware::from_fn_with_state(
            exigir_audiencia,
            exigir,
        ))
        .with_state(estado)
}
