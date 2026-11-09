//! Fase E — Router de clientes, agora com gestão de carteira: sócios/contatos,
//! filiais, tags, carteira (responsável/observações) e importação em massa.

use axum::{
    Router,
    extract::{Extension, Json, Path, Query, State},
    middleware as axum_middleware,
    response::{IntoResponse, Response},
    routing::{get, post},
};

use crate::application::clientes::{
    AtualizarClienteUseCase, CarteiraClienteUseCase, CriarClienteUseCase, ExcluirClienteUseCase,
    GestaoClienteTagsUseCase, GestaoFiliaisUseCase, GestaoSociosContatosUseCase,
    ImportarClientesUseCase, ListarClientesCommand, ListarClientesUseCase,
    ListarConsultasDoClienteUseCase, ObterClienteUseCase, SituacaoFiscalDoClienteUseCase,
    VerificarAcessoUseCase,
};
use crate::domain::auth::Audiencia;
use crate::presentation::auth::{ExigirAudiencia, RequestContext, exigir};
use crate::presentation::clientes::dto::{
    AtualizarClienteRequest, CriarClienteRequest, ListagemClientesResponse, ListarClientesQuery,
    ListarConsultasQuery,
};
use crate::presentation::clientes::guardas;
use crate::presentation::clientes::handlers;
use crate::presentation::clientes::handlers_carteira;
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
    // Fase E — gestão de carteira
    pub verificar: Arc<VerificarAcessoUseCase>,
    pub socios: Arc<GestaoSociosContatosUseCase>,
    pub filiais: Arc<GestaoFiliaisUseCase>,
    pub tags: Arc<GestaoClienteTagsUseCase>,
    pub carteira: Arc<CarteiraClienteUseCase>,
    pub importar: Arc<ImportarClientesUseCase>,
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
    verificar: VerificarAcessoUseCase,
    socios: GestaoSociosContatosUseCase,
    filiais: GestaoFiliaisUseCase,
    tags: GestaoClienteTagsUseCase,
    carteira: CarteiraClienteUseCase,
    importar: ImportarClientesUseCase,
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
        verificar: Arc::new(verificar),
        socios: Arc::new(socios),
        filiais: Arc::new(filiais),
        tags: Arc::new(tags),
        carteira: Arc::new(carteira),
        importar: Arc::new(importar),
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
        // ===== Fase E — carteira do cliente =====
        .route(
            "/{id}/socios-contatos",
            get(handlers_carteira::listar_socios_contatos)
                .post(handlers_carteira::criar_socio_contato),
        )
        .route(
            "/{id}/socios-contatos/{socio_id}",
            axum::routing::delete(handlers_carteira::remover_socio_contato),
        )
        .route(
            "/{id}/filiais",
            get(handlers_carteira::listar_filiais).post(handlers_carteira::criar_filial),
        )
        .route(
            "/{id}/filiais/{filial_id}",
            axum::routing::delete(handlers_carteira::remover_filial),
        )
        .route(
            "/tags",
            get(handlers_carteira::listar_tags).post(handlers_carteira::criar_tag),
        )
        .route(
            "/{id}/tags",
            get(handlers_carteira::listar_tags_do_cliente).post(handlers_carteira::vincular_tag),
        )
        .route(
            "/{id}/tags/{tag_id}",
            axum::routing::delete(handlers_carteira::desvincular_tag),
        )
        .route(
            "/{id}/carteira",
            get(handlers_carteira::obter_carteira).put(handlers_carteira::definir_carteira),
        )
        // ===== Importação em massa (prévia → confirmar) =====
        .route("/importar", post(handlers_carteira::importar_analise))
        .route(
            "/importar/confirmar",
            post(handlers_carteira::importar_confirmar),
        )
        .route("/meu-acesso", get(handlers_carteira::meu_acesso))
        .route_layer(axum_middleware::from_fn_with_state(auth, exigir))
        .with_state(state)
}

// ===== Wrappers legados com as guardas da Fase E =====

async fn acesso_do_contexto(
    verificar: &Arc<VerificarAcessoUseCase>,
    contexto: &RequestContext,
) -> Result<crate::domain::clientes::AcessoCarteira, ApiError> {
    guardas::acesso_do_contexto(verificar, contexto)
}

async fn criar_cliente(
    State(state): State<ClientesRouterState>,
    Extension(contexto): Extension<RequestContext>,
    Json(payload): Json<CriarClienteRequest>,
) -> Result<Response, ApiError> {
    let acesso = acesso_do_contexto(&state.verificar, &contexto).await?;
    guardas::exigir_escrita(&acesso, None)?;
    handlers::criar_cliente(
        State((*state.criar).clone()),
        Extension(contexto),
        Json(payload),
    )
    .await
}

async fn obter_cliente(
    State(state): State<ClientesRouterState>,
    Extension(contexto): Extension<RequestContext>,
    Path(id): Path<Uuid>,
) -> Result<Response, ApiError> {
    let acesso = acesso_do_contexto(&state.verificar, &contexto).await?;
    guardas::exigir_ver(&acesso, Some(id))?;
    handlers::obter_cliente(State((*state.obter).clone()), Extension(contexto), Path(id)).await
}

async fn listar_clientes(
    State(state): State<ClientesRouterState>,
    Extension(contexto): Extension<RequestContext>,
    Query(query): Query<ListarClientesQuery>,
) -> Result<Response, ApiError> {
    let acesso = acesso_do_contexto(&state.verificar, &contexto).await?;
    let escritorio_id = contexto.escritorio_id.ok_or_else(ApiError::proibido)?;
    let (filtro, paginacao) = query.para_filtro_e_paginacao()?;

    let listagem = state
        .listar
        .executar(ListarClientesCommand {
            escritorio_id,
            filtro,
            paginacao,
            apenas_ids: acesso.clientes_permitidos.clone(),
        })
        .await?;
    Ok(Json(ListagemClientesResponse::from(listagem)).into_response())
}

async fn listar_consultas_do_cliente(
    State(state): State<ClientesRouterState>,
    Extension(contexto): Extension<RequestContext>,
    Path(id): Path<Uuid>,
    Query(query): Query<ListarConsultasQuery>,
) -> Result<Response, ApiError> {
    let acesso = acesso_do_contexto(&state.verificar, &contexto).await?;
    guardas::exigir_ver(&acesso, Some(id))?;
    handlers::listar_consultas_do_cliente(
        State((*state.listar_consultas).clone()),
        Extension(contexto),
        Path(id),
        Query(query),
    )
    .await
}

async fn situacao_fiscal_do_cliente(
    State(state): State<ClientesRouterState>,
    Extension(contexto): Extension<RequestContext>,
    Path(id): Path<Uuid>,
) -> Result<Response, ApiError> {
    let acesso = acesso_do_contexto(&state.verificar, &contexto).await?;
    guardas::exigir_ver(&acesso, Some(id))?;
    handlers::situacao_fiscal_do_cliente(
        State((*state.situacao_fiscal).clone()),
        Extension(contexto),
        Path(id),
    )
    .await
}

async fn atualizar_cliente(
    State(state): State<ClientesRouterState>,
    Extension(contexto): Extension<RequestContext>,
    Path(id): Path<Uuid>,
    Json(payload): Json<AtualizarClienteRequest>,
) -> Result<Response, ApiError> {
    let acesso = acesso_do_contexto(&state.verificar, &contexto).await?;
    guardas::exigir_escrita(&acesso, Some(id))?;
    handlers::atualizar_cliente(
        State((*state.atualizar).clone()),
        Extension(contexto),
        Path(id),
        Json(payload),
    )
    .await
}

async fn excluir_cliente(
    State(state): State<ClientesRouterState>,
    Extension(contexto): Extension<RequestContext>,
    Path(id): Path<Uuid>,
) -> Result<Response, ApiError> {
    let acesso = acesso_do_contexto(&state.verificar, &contexto).await?;
    guardas::exigir_escrita(&acesso, Some(id))?;
    handlers::excluir_cliente(State((*state.excluir).clone()), Extension(contexto), Path(id)).await
}
