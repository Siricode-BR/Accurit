//! Router `/divida-ativa` (Fase 4): leituras autenticadas do portal
//! com audiência `Client`, snapshot de ambiente uma vez por
//! requisição (SPEC §8) e rate limit global preservado.

use std::sync::Arc;

use axum::middleware as axum_middleware;
use axum::{Router, routing::get};

use crate::application::divida_ativa::use_cases::{
    ConsultarDevedorUseCase, ConsultarInscricaoUseCase,
};
use crate::domain::auth::Audiencia;
use crate::presentation::auth::{AuthState, ExigirAudiencia, exigir};
use crate::presentation::divida_ativa::handlers::{consultar_devedor, consultar_inscricao};
use crate::presentation::serpro::ambiente::{EstadoAmbiente, resolver_ambiente};

#[derive(Clone)]
pub struct EstadoDivida {
    pub devedor: Arc<ConsultarDevedorUseCase>,
    pub inscricao: Arc<ConsultarInscricaoUseCase>,
}

/// Constrói o sub-router. Os middleware seguem a mesma ordem do
/// portal SERPRO (SPEC §8): `exigir` (insere RequestContext) é o mais
/// externo; o snapshot `resolver_ambiente` é o mais interno.
pub fn construir(
    devedor: Arc<ConsultarDevedorUseCase>,
    inscricao: Arc<ConsultarInscricaoUseCase>,
    auth_state: AuthState,
    estado_ambiente: EstadoAmbiente,
) -> Router {
    let exigencia = ExigirAudiencia::novo(auth_state.tokens.clone(), Audiencia::Client);

    Router::new()
        .route("/devedor/{cpf_cnpj}", get(consultar_devedor))
        .route("/inscricao/{numero}", get(consultar_inscricao))
        .route_layer(axum_middleware::from_fn_with_state(
            estado_ambiente,
            resolver_ambiente,
        ))
        .route_layer(axum_middleware::from_fn_with_state(exigencia, exigir))
        .with_state(EstadoDivida { devedor, inscricao })
}
