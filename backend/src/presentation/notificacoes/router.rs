//! Fase C — Rotas de notificações.

use axum::middleware as axum_middleware;
use axum::routing::{get, post, patch};
use axum::Router;

use crate::presentation::auth::{ExigirAudiencia, exigir};
use crate::presentation::notificacoes::handlers::{
    buscar_preferencia, contar_nao_lidas, criar_notificacao, criar_ou_atualizar_preferencia,
    listar_notificacoes, listar_templates, marcar_como_lida, marcar_todas_como_lidas,
};
use crate::presentation::notificacoes::state::EstadoNotificacoes;

pub fn construir(
    estado: EstadoNotificacoes,
    exigir_audiencia: ExigirAudiencia,
) -> Router {
    Router::new()
        .route("/notificacoes", post(criar_notificacao).get(listar_notificacoes))
        .route(
            "/notificacoes/contagem-nao-lidas",
            get(contar_nao_lidas),
        )
        .route(
            "/notificacoes/{id}/ler",
            patch(marcar_como_lida),
        )
        .route(
            "/notificacoes/marcar-todas-lidas",
            post(marcar_todas_como_lidas),
        )
        .route(
            "/preferencias-notificacao",
            get(buscar_preferencia).post(criar_ou_atualizar_preferencia),
        )
        .route("/templates-notificacao", get(listar_templates))
        .route_layer(axum_middleware::from_fn_with_state(
            exigir_audiencia,
            exigir,
        ))
        .with_state(estado)
}
