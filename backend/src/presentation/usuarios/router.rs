use std::sync::Arc;

use axum::{
    Router,
    extract::{Extension, Json, Path, Query, State},
    http::StatusCode,
    middleware as axum_middleware,
    routing::{get, post},
};
use uuid::Uuid;

use crate::application::usuarios::{
    Ator, AtualizarUsuarioCommand, CriarUsuarioCommand, FiltroUsuarios, GestaoUsuariosUseCase,
    OrigemAtor,
};
use crate::domain::auth::{Audiencia, Papel};
use crate::presentation::auth::{AuthState, ExigirAudiencia, RequestContext, exigir};
use crate::presentation::error::ApiError;
use crate::presentation::usuarios::dto::{
    AtualizarUsuarioRequest, CriarUsuarioRequest, ListaUsuariosResponse, ListarUsuariosQuery,
    RedefinirSenhaRequest, UsuarioDto,
};

type Estado = Arc<GestaoUsuariosUseCase>;

/// Titular (`CLIENTE`) do escritório; `OPERADOR` não gerencia usuários.
fn ator_portal(ctx: &RequestContext) -> Result<Ator, ApiError> {
    let escritorio = ctx.escritorio_id.ok_or_else(ApiError::proibido)?;
    if ctx.papel != Papel::Cliente {
        return Err(ApiError::proibido());
    }
    Ok(Ator {
        id: ctx.usuario_id,
        origem: OrigemAtor::Escritorio(escritorio),
        ip: ctx.ip,
    })
}

fn ator_admin(ctx: &RequestContext) -> Result<Ator, ApiError> {
    if ctx.papel != Papel::Admin {
        return Err(ApiError::proibido());
    }
    Ok(Ator {
        id: ctx.usuario_id,
        origem: OrigemAtor::Backoffice,
        ip: ctx.ip,
    })
}

async fn listar(
    State(uc): State<Estado>,
    Extension(ctx): Extension<RequestContext>,
    Query(q): Query<ListarUsuariosQuery>,
    admin: bool,
) -> Result<Json<ListaUsuariosResponse>, ApiError> {
    let ator = if admin { ator_admin(&ctx)? } else { ator_portal(&ctx)? };
    let itens: Vec<UsuarioDto> = uc
        .listar(
            &ator,
            FiltroUsuarios {
                escritorio_id: q.escritorio_id,
                busca: q.busca,
                ativo: q.ativo,
            },
        )?
        .into_iter()
        .map(Into::into)
        .collect();
    Ok(Json(ListaUsuariosResponse {
        total: itens.len(),
        itens,
    }))
}

async fn criar(
    uc: Estado,
    ctx: RequestContext,
    req: CriarUsuarioRequest,
    admin: bool,
) -> Result<(StatusCode, Json<UsuarioDto>), ApiError> {
    let ator = if admin { ator_admin(&ctx)? } else { ator_portal(&ctx)? };
    let criado = uc.criar(
        &ator,
        CriarUsuarioCommand {
            escritorio_id: if admin { req.escritorio_id } else { None },
            nome: req.nome,
            email: req.email,
            cpf: req.cpf,
            papel: req.papel,
            senha: req.senha,
        },
    )?;
    Ok((StatusCode::CREATED, Json(criado.into())))
}

async fn obter(
    uc: Estado,
    ctx: RequestContext,
    id: Uuid,
    admin: bool,
) -> Result<Json<UsuarioDto>, ApiError> {
    let ator = if admin { ator_admin(&ctx)? } else { ator_portal(&ctx)? };
    Ok(Json(uc.obter(&ator, id)?.into()))
}

async fn atualizar(
    uc: Estado,
    ctx: RequestContext,
    id: Uuid,
    req: AtualizarUsuarioRequest,
    admin: bool,
) -> Result<Json<UsuarioDto>, ApiError> {
    let ator = if admin { ator_admin(&ctx)? } else { ator_portal(&ctx)? };
    let novo = uc.atualizar(
        &ator,
        id,
        AtualizarUsuarioCommand {
            nome: req.nome,
            papel: req.papel,
            ativo: req.ativo,
        },
    )?;
    Ok(Json(novo.into()))
}

async fn senha(
    uc: Estado,
    ctx: RequestContext,
    id: Uuid,
    req: RedefinirSenhaRequest,
    admin: bool,
) -> Result<StatusCode, ApiError> {
    let ator = if admin { ator_admin(&ctx)? } else { ator_portal(&ctx)? };
    uc.redefinir_senha(&ator, id, &req.senha)?;
    Ok(StatusCode::NO_CONTENT)
}

macro_rules! rotas {
    ($admin:expr) => {
        Router::new()
            .route(
                "/",
                get(
                    |s: State<Estado>, c: Extension<RequestContext>, q: Query<ListarUsuariosQuery>| {
                        listar(s, c, q, $admin)
                    },
                )
                .post(
                    |State(s): State<Estado>,
                     Extension(c): Extension<RequestContext>,
                     Json(r): Json<CriarUsuarioRequest>| criar(s, c, r, $admin),
                ),
            )
            .route(
                "/{id}",
                get(
                    |State(s): State<Estado>,
                     Extension(c): Extension<RequestContext>,
                     Path(id): Path<Uuid>| obter(s, c, id, $admin),
                )
                .patch(
                    |State(s): State<Estado>,
                     Extension(c): Extension<RequestContext>,
                     Path(id): Path<Uuid>,
                     Json(r): Json<AtualizarUsuarioRequest>| atualizar(s, c, id, r, $admin),
                ),
            )
            .route(
                "/{id}/senha",
                post(
                    |State(s): State<Estado>,
                     Extension(c): Extension<RequestContext>,
                     Path(id): Path<Uuid>,
                     Json(r): Json<RedefinirSenhaRequest>| senha(s, c, id, r, $admin),
                ),
            )
    };
}

fn proteger(router: Router<Estado>, auth: &AuthState, audiencia: Audiencia, uc: Estado) -> Router {
    router
        .route_layer(axum_middleware::from_fn_with_state(
            ExigirAudiencia::novo(auth.tokens.clone(), audiencia),
            exigir,
        ))
        .with_state(uc)
}

/// `/usuarios` — portal do escritório (audiência client).
pub fn construir_usuarios(uc: GestaoUsuariosUseCase, auth: &AuthState) -> Router {
    proteger(rotas!(false), auth, Audiencia::Client, Arc::new(uc))
}

/// Rotas `/usuarios` do backoffice, para mesclar sob `/admin`.
pub fn construir_usuarios_admin(uc: GestaoUsuariosUseCase, auth: &AuthState) -> Router {
    Router::new().nest(
        "/usuarios",
        proteger(rotas!(true), auth, Audiencia::Admin, Arc::new(uc)),
    )
}
