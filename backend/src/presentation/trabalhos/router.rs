//! Fase B — Rotas de trabalhos agendados e consultas em lote.

use axum::Router;
use axum::middleware as axum_middleware;
use axum::routing::{get, post};

use crate::presentation::auth::{ExigirAudiencia, exigir};
use crate::presentation::trabalhos::handlers::{
    alternar_trabalho, buscar_consulta_lote, buscar_trabalho, criar_consulta_lote, criar_trabalho,
    listar_consultas_lote, listar_execucoes_escritorio, listar_execucoes_por_trabalho,
    listar_itens_consulta_lote, listar_status_execucao, listar_status_lote, listar_tipos_trabalho,
    listar_trabalhos, reexecutar_consulta_lote,
};
use crate::presentation::trabalhos::state::EstadoTrabalhos;

pub fn construir(estado: EstadoTrabalhos, exigir_audiencia: ExigirAudiencia) -> Router {
    Router::new()
        .route("/trabalhos", post(criar_trabalho).get(listar_trabalhos))
        .route("/trabalhos/{id}", get(buscar_trabalho))
        .route(
            "/trabalhos/{id}/alternar",
            axum::routing::patch(alternar_trabalho),
        )
        .route(
            "/execucoes/trabalho/{trabalho_id}",
            get(listar_execucoes_por_trabalho),
        )
        .route("/execucoes", get(listar_execucoes_escritorio))
        .route(
            "/consultas-lote",
            post(criar_consulta_lote).get(listar_consultas_lote),
        )
        .route("/consultas-lote/{id}", get(buscar_consulta_lote))
        .route(
            "/consultas-lote/{id}/itens",
            get(listar_itens_consulta_lote),
        )
        .route(
            "/consultas-lote/{id}/executar",
            post(reexecutar_consulta_lote),
        )
        .route("/trabalhos/tipos", get(listar_tipos_trabalho))
        .route("/trabalhos/status-execucao", get(listar_status_execucao))
        .route("/trabalhos/status-lote", get(listar_status_lote))
        .route_layer(axum_middleware::from_fn_with_state(
            exigir_audiencia,
            exigir,
        ))
        .with_state(estado)
}
