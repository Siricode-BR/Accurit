//! Fase F — Rotas de relatórios e link de compartilhamento.

use axum::Router;
use axum::middleware as axum_middleware;
use axum::routing::{get, post};

use crate::presentation::auth::{ExigirAudiencia, exigir};
use crate::presentation::relatorios::handlers::{
    baixar_relatorio, buscar_relatorio, criar_link_compartilhamento, criar_relatorio,
    listar_relatorios, obter_relatorio_por_link,
};
use crate::presentation::relatorios::state::EstadoRelatorios;

/// Rotas autenticadas (audiência `client`) de relatórios.
pub fn construir(estado: EstadoRelatorios, exigir_audiencia: ExigirAudiencia) -> Router {
    Router::new()
        .route("/relatorios", post(criar_relatorio).get(listar_relatorios))
        .route("/relatorios/{id}", get(buscar_relatorio))
        .route("/relatorios/{id}/baixar", get(baixar_relatorio))
        .route(
            "/relatorios/{id}/compartilhar",
            post(criar_link_compartilhamento),
        )
        .route_layer(axum_middleware::from_fn_with_state(
            exigir_audiencia,
            exigir,
        ))
        .with_state(estado)
}

/// Rota pública de acesso a relatório via link seguro (somente leitura, sem autenticação).
pub fn construir_publico(estado: EstadoRelatorios) -> Router {
    Router::new()
        .route(
            "/relatorios/compartilhado/{token}",
            get(obter_relatorio_por_link),
        )
        .with_state(estado)
}
