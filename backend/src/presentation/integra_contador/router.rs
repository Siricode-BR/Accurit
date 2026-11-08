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
        .route(
            "/procuracoes/autenticar",
            post(handlers::autenticar_procurador),
        )
        .route("/procuracoes", get(handlers::listar_procuracao))
        .route("/procuracoes/validar", post(handlers::validar_procuracao))
        .route(
            "/procuracoes/revalidar",
            post(handlers::revalidar_procuracao),
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

/// `/admin/integra-contador` — backoffice: catálogo de serviços (ADR-0013).
/// Não usa `resolver_ambiente`: o catálogo não é específico de ambiente.
pub fn construir_admin(estado: EstadoIntegraContador, exigir_audiencia: ExigirAudiencia) -> Router {
    Router::new()
        .route("/catalogo", get(handlers::catalogo))
        .route(
            "/catalogo/{id_sistema}/{id_servico}",
            axum::routing::patch(handlers::definir_habilitado_catalogo),
        )
        .route_layer(axum_middleware::from_fn_with_state(
            exigir_audiencia,
            exigir,
        ))
        .with_state(estado)
}
