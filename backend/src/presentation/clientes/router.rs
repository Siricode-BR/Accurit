use axum::{
    Router,
    extract::{Extension, Json, Path, Query, State},
    middleware as axum_middleware,
    routing::{get, post},
};

use crate::application::clientes::{
    AtualizarClienteUseCase, CriarClienteUseCase, ExcluirClienteUseCase, ListarClientesUseCase,
    ListarConsultasDoClienteUseCase, ObterClienteUseCase, SituacaoFiscalDoClienteUseCase,
};
use crate::domain::auth::Audiencia;
use crate::presentation::auth::{ExigirAudiencia, RequestContext, exigir};
use crate::presentation::clientes::dto::{
    AtualizarClienteRequest, CriarClienteRequest, ListarClientesQuery, ListarConsultasQuery,
};
use crate::presentation::clientes::handlers;
use crate::presentation::error::ApiError;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct ClientesRouterState {
    pub criar: Arc<CriarClienteUseCase>,
    pub obter: Arc<ObterClienteUseCase>,
    pub listar: Arc<ListarClientesUseCase>,
    pub atualizar: Arc<AtualizarClienteUseCase>,
    pub excluir: Arc<ExcluirClienteUseCase>,
    pub listar_consultas: Arc<ListarConsultasDoClienteUseCase>,
    pub situacao_fiscal: Arc<SituacaoFiscalDoClienteUseCase>,
}

#[allow(clippy::too_many_arguments)]
pub fn construir_clientes(
    criar: CriarClienteUseCase,
    obter: ObterClienteUseCase,
    listar: ListarClientesUseCase,
    atualizar: AtualizarClienteUseCase,
    excluir: ExcluirClienteUseCase,
    listar_consultas: ListarConsultasDoClienteUseCase,
    situacao_fiscal: SituacaoFiscalDoClienteUseCase,
    auth_state: crate::presentation::auth::AuthState,
) -> Router {
    let auth = ExigirAudiencia::novo(auth_state.tokens.clone(), Audiencia::Client);

    let state = ClientesRouterState {
        criar: Arc::new(criar),
        obter: Arc::new(obter),
        listar: Arc::new(listar),
        atualizar: Arc::new(atualizar),
        excluir: Arc::new(excluir),
        listar_consultas: Arc::new(listar_consultas),
        situacao_fiscal: Arc::new(situacao_fiscal),
    };

    Router::new()
        .route("/", post(criar_cliente).get(listar_clientes))
        .route(
            "/{id}",
            get(obter_cliente)
                .put(atualizar_cliente)
                .delete(excluir_cliente),
        )
        .route("/{id}/consultas", get(listar_consultas_do_cliente))
        .route("/{id}/situacao-fiscal", get(situacao_fiscal_do_cliente))
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
        Extension(context),
        Json(payload),
    )
    .await
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
    )
    .await
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
    )
    .await
}

async fn listar_consultas_do_cliente(
    State(state): State<ClientesRouterState>,
    Extension(context): Extension<RequestContext>,
    Path(id): Path<Uuid>,
    Query(query): Query<ListarConsultasQuery>,
) -> Result<axum::response::Response, ApiError> {
    handlers::listar_consultas_do_cliente(
        State(state.listar_consultas.as_ref().clone()),
        Extension(context),
        Path(id),
        Query(query),
    )
    .await
}

async fn situacao_fiscal_do_cliente(
    State(state): State<ClientesRouterState>,
    Extension(context): Extension<RequestContext>,
    Path(id): Path<Uuid>,
) -> Result<axum::response::Response, ApiError> {
    handlers::situacao_fiscal_do_cliente(
        State(state.situacao_fiscal.as_ref().clone()),
        Extension(context),
        Path(id),
    )
    .await
}

async fn atualizar_cliente(
    State(state): State<ClientesRouterState>,
    Extension(context): Extension<RequestContext>,
    Path(id): Path<Uuid>,
    Json(payload): Json<AtualizarClienteRequest>,
) -> Result<axum::response::Response, ApiError> {
    handlers::atualizar_cliente(
        State(state.atualizar.as_ref().clone()),
        Extension(context),
        Path(id),
        Json(payload),
    )
    .await
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
    )
    .await
}
