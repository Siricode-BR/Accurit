#! Router da Integra Contador: lida com os 5 verbos (Apoiar, Consultar, Declarar, Emitir, Monitorar).
#!
#! Rotas disponíveis:
#' - GET  /status - status do sistema (ambiente, producao, assinatura)
#' - POST /executar - executar qualquer um dos 5 verbos
#' - PATCH /modo-trial - alterna TRIAL/PRODUCAO (admin)
#'
#! Middleware ordem (SPEC §8):
#' 1. `exigir` (audiência client/backoffice) - insere RequestContext
#' 2. `resolver_ambiente` - snapshot do ambiente (único por requisição)
#' 3. `with_state(EstadoIntegraContador)` - share use case e estado

use axum::Router;
use axum::routing::{get, patch, post};
use std::sync::Arc;

use crate::presentation::auth::{ExigirAudiencia, exigir};
use crate::presentation::serpro::ambiente::{EstadoAmbiente, resolver_ambiente};
use crate::presentation::integra_contador::state::EstadoIntegraContador;

pub fn construir(
    estado: Arc<EstadoIntegraContador>,
    exigir_audiencia: ExigirAudiencia,
    estado_ambiente: EstadoAmbiente,
) -> Router {
    Router::new()
        .route(
            "/status",
            get(crate::presentation::integra_contador::handlers::consultar_status),
        )
        .route(
            "/executar",
            post(crate::presentation::integra_contador::handlers::executar_operacao),
        )
        .route(
            "/modo-trial",
            patch(crate::presentation::integra_contador::handlers::alterar_modo_trial),
        )
        // Ordem dos route_layer: o último adicionado é o mais externo.
        // `exigir` roda primeiro (insere RequestContext);
        // o snapshot entra logo depois, mais interno (SPEC §8).
        .route_layer(axum::middleware::from_fn_with_state(
            estado_ambiente,
            resolver_ambiente,
        ))
        .route_layer(axum::middleware::from_fn_with_state(
            exigir_audiencia,
            exigir,
        ))
        .with_state(estado)
}

pub fn construir_admin(
    estado: Arc<EstadoIntegraContador>,
    exigir_audiencia: ExigirAudiencia,
) -> Router {
    Router::new()
        .route(
            "/catalogo",
            get(crate::presentation::integra_contador::handlers::catalogo),
        )
        .route(
            "/catalogo/{id_sistema}/{id_servico}",
            patch(crate::presentation::integra_contador::handlers::definir_habilitado_catalogo),
        )
        .route_layer(axum::middleware::from_fn_with_state(
            exigir_audiencia,
            exigir,
        ))
        .with_state(estado)
}