use axum::{
    extract::{Extension, Json, Path, Query, State},
    middleware as axum_middleware,
    routing::{delete, get, post, put},
    Router,
};

use crate::application::clientes::{
    AtualizarClienteUseCase, CriarClienteUseCase, ExcluirClienteUseCase, ListarClientesUseCase,
    ObterClienteUseCase, PoliticaClientes,
};
use crate::domain::auth::Audiencia;
use crate::presentation::auth::{exigir, ExigirAudiencia, RequestContext};
use crate::presentation::clientes::dto::{
    AtualizarClienteRequest, CriarClienteRequest, ListarClientesQuery,
};
use crate::presentation::error::ApiError;
use crate::presentation::clientes::handlers;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct ClientesRouterState {
    pub criar: Arc<CriarClienteUseCase>,
    pub obter: Arc<ObterClienteUseCase>,
    pub listar: Arc<ListarClientesUseCase>,
    pub atualizar: Arc<AtualizarClienteUseCase>,
    pub excluir: Arc<ExcluirClienteUseCase>,
    pub politica: PoliticaClientes,
}

pub fn construir_clientes(
    criar: CriarClienteUseCase,
    obter: ObterClienteUseCase,
    listar: ListarClientesUseCase,
    atualizar: AtualizarClienteUseCase,
    excluir: ExcluirClienteUseCase,
    politica: PoliticaClientes,
    auth_state: crate::presentation::auth::AuthState,
) -> Router {
    let auth = ExigirAudiencia::novo(auth_state.tokens.clone(), Audiencia::Client);

    let state = ClientesRouterState {
        criar: Arc::new(criar),
        obter: Arc::new(obter),
        listar: Arc::new(listar),
        atualizar: Arc::new(atualizar),
        excluir: Arc::new(excluir),
        politica,
    };

    Router::new()
        .route("/", post(criar_cliente).get(listar_clientes))
        .route(
            "/{id}",
            get(obter_cliente)
                .put(atualizar_cliente)
                .delete(excluir_cliente),
        )
        .route_layer(axum_middleware::from_fn_with_state(auth, exigir))
        .with_state(state)
}

async fn criar_cliente(
    State(state): State<ClientesRouterState>,
    Extension(context): Extension<RequestContext>,
    Json(payload): Json<CriarClienteRequest>,
) -> Result<axum::response::Response, ApiError> {
    handlers::criar_cliente(
        State(state.criar.as_ref().clone()),
        State(state.politica),
        Extension(context),
        Json(payload),
    ).await
}

async fn obter_cliente(
    State(state): State<ClientesRouterState>,
    Extension(context): Extension<RequestContext>,
    Path(id): Path<Uuid>,
) -> Result<axum::response::Response, ApiError> {
    handlers::obter_cliente(
        State(state.obter.as_ref().clone()),
        Extension(context),
        Path(id),
    ).await
}

async fn listar_clientes(
    State(state): State<ClientesRouterState>,
    Extension(context): Extension<RequestContext>,
    Query(query): Query<ListarClientesQuery>,
) -> Result<axum::response::Response, ApiError> {
    handlers::listar_clientes(
        State(state.listar.as_ref().clone()),
        Extension(context),
        Query(query),
    ).await
}

async fn atualizar_cliente(
    State(state): State<ClientesRouterState>,
    Extension(context): Extension<RequestContext>,
    Path(id): Path<Uuid>,
    Json(payload): Json<AtualizarClienteRequest>,
) -> Result<axum::response::Response, ApiError> {
    handlers::atualizar_cliente(
        State(state.atualizar.as_ref().clone()),
        State(state.politica),
        Extension(context),
        Path(id),
        Json(payload),
    ).await
}

async fn excluir_cliente(
    State(state): State<ClientesRouterState>,
    Extension(context): Extension<RequestContext>,
    Path(id): Path<Uuid>,
) -> Result<axum::response::Response, ApiError> {
    handlers::excluir_cliente(
        State(state.excluir.as_ref().clone()),
        Extension(context),
        Path(id),
    ).await
}

pub fn construir_admin_escritorios(
    provisionar_use_case: crate::application::admin::ProvisionarTenantUseCase,
    auth_state: crate::presentation::auth::AuthState,
) -> Router {
    let auth = ExigirAudiencia::novo(auth_state.tokens.clone(), Audiencia::Admin);

    let use_case = Arc::new(provisionar_use_case);

    Router::new()
        .route("/escritorios", post(provisionar_tenant))
        .route_layer(axum_middleware::from_fn_with_state(auth, exigir))
        .with_state(use_case.clone())
}

async fn provisionar_tenant(
    State(use_case): State<Arc<crate::application::admin::ProvisionarTenantUseCase>>,
    Extension(context): Extension<RequestContext>,
    Json(payload): Json<crate::presentation::admin::dto::ProvisionarTenantRequest>,
) -> Result<axum::response::Response, crate::presentation::error::ApiError> {
    crate::presentation::admin::handlers::provisionar_tenant(
        State(use_case.as_ref().clone()),
        Extension(context),
        Json(payload),
    ).await
}