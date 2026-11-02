//! Router da Dívida Ativa: apenas `GET` (ADR-0002 — serviço
//! estritamente consultivo). Middleware: `exigir` (audiência client,
//! RequestContext) e `resolver_ambiente` (snapshot do ambiente, SPEC
//! §8), nessa ordem.

use axum::Router;
use std::sync::Arc;

use crate::application::divida_ativa::use_cases::{
    ConsultarDevedorUseCase, ConsultarInscricaoUseCase,
};
use crate::presentation::auth::ExigirAudiencia;
use crate::presentation::auth::exigir;
use crate::presentation::divida_ativa::handlers::EstadoDividaAtiva;
use crate::presentation::serpro::ambiente::{EstadoAmbiente, resolver_ambiente};
use axum::middleware as axum_middleware;
use axum::routing::get;

pub fn construir(
    consultar_devedor: Arc<ConsultarDevedorUseCase>,
    consultar_inscricao: Arc<ConsultarInscricaoUseCase>,
    exigir_audiencia: ExigirAudiencia,
    estado_ambiente: EstadoAmbiente,
) -> Router {
    let estado = EstadoDividaAtiva {
        consultar_devedor,
        consultar_inscricao,
    };

    Router::new()
        .route(
            "/devedores/{cpf_cnpj}",
            get(crate::presentation::divida_ativa::handlers::consultar_devedor),
        )
        .route(
            "/inscricoes/{numero_inscricao}",
            get(crate::presentation::divida_ativa::handlers::consultar_inscricao),
        )
        // Ordem dos route_layer: o último é o mais externo. `exigir`
        // roda primeiro (insere RequestContext); o snapshot entra
        // logo depois, mais interno (SPEC §8).
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
