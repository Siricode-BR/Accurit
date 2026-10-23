use axum::{
    Router,
    extract::{Extension, Json, Path, Query, State},
    middleware as axum_middleware,
    routing::{get, post},
};
use uuid::Uuid;

use crate::application::certificados::{
    CriarCertificadoUseCase, ListarCertificadosUseCase, ObterCertificadoUseCase,
    RevogarCertificadoUseCase,
};
use crate::domain::auth::Audiencia;
use crate::presentation::auth::{ExigirAudiencia, RequestContext, exigir};
use crate::presentation::certificados::dto::ListarCertificadosQuery;
use crate::presentation::certificados::handlers;
use std::sync::Arc;

#[derive(Clone)]
pub struct CertificadosRouterState {
    pub criar: Arc<CriarCertificadoUseCase>,
    pub listar: Arc<ListarCertificadosUseCase>,
    pub obter: Arc<ObterCertificadoUseCase>,
    pub revogar: Arc<RevogarCertificadoUseCase>,
}

pub fn construir_certificados(
    criar: CriarCertificadoUseCase,
    listar: ListarCertificadosUseCase,
    obter: ObterCertificadoUseCase,
    revogar: RevogarCertificadoUseCase,
    auth_state: crate::presentation::auth::AuthState,
) -> Router {
    let auth = ExigirAudiencia::novo(auth_state.tokens.clone(), Audiencia::Client);

    let state = CertificadosRouterState {
        criar: Arc::new(criar),
        listar: Arc::new(listar),
        obter: Arc::new(obter),
        revogar: Arc::new(revogar),
    };

    Router::new()
        .route("/", post(criar_certificado).get(listar_certificados))
        .route("/{id}", get(obter_certificado).delete(revogar_certificado))
        .route_layer(axum_middleware::from_fn_with_state(auth, exigir))
        .with_state(state)
}

async fn criar_certificado(
    State(state): State<CertificadosRouterState>,
    Extension(context): Extension<RequestContext>,
    Json(payload): Json<crate::presentation::certificados::dto::CriarCertificadoRequest>,
) -> Result<axum::response::Response, crate::presentation::error::ApiError> {
    handlers::criar_certificado(
        State(state.criar.as_ref().clone()),
        Extension(context),
        Json(payload),
    )
    .await
}

async fn listar_certificados(
    State(state): State<CertificadosRouterState>,
    Extension(context): Extension<RequestContext>,
    Query(query): Query<ListarCertificadosQuery>,
) -> Result<axum::response::Response, crate::presentation::error::ApiError> {
    handlers::listar_certificados(
        State(state.listar.as_ref().clone()),
        Extension(context),
        Query(query),
    )
    .await
}

async fn obter_certificado(
    State(state): State<CertificadosRouterState>,
    Extension(context): Extension<RequestContext>,
    Path(id): Path<Uuid>,
) -> Result<axum::response::Response, crate::presentation::error::ApiError> {
    handlers::obter_certificado(
        State(state.obter.as_ref().clone()),
        Extension(context),
        Path(id),
    )
    .await
}

async fn revogar_certificado(
    State(state): State<CertificadosRouterState>,
    Extension(context): Extension<RequestContext>,
    Path(id): Path<Uuid>,
) -> Result<axum::response::Response, crate::presentation::error::ApiError> {
    handlers::revogar_certificado(
        State(state.revogar.as_ref().clone()),
        Extension(context),
        Path(id),
    )
    .await
}
