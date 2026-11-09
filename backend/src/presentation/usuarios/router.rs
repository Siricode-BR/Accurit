//! Fase E — Router de usuários com paginação no servidor e, para o portal,
//! gestão de função e carteira (`/{id}/acesso`, `/{id}/carteira`), restrita à
//! função GESTOR.

use std::sync::Arc;

use axum::{
    Router,
    extract::{Extension, Json, Path, Query, State},
    http::StatusCode,
    middleware as axum_middleware,
    routing::{get, post},
};
use uuid::Uuid;

use crate::application::clientes::{
    GestaoAcessoUsuariosUseCase, VerificarAcessoUseCase,
};
use crate::application::usuarios::{
    Ator, AtualizarUsuarioCommand, CriarUsuarioCommand, FiltroUsuarios, GestaoUsuariosUseCase,
    OrigemAtor,
};
use crate::domain::auth::{Audiencia, Papel};
use crate::domain::clientes::FuncaoUsuario;
use crate::presentation::auth::{AuthState, ExigirAudiencia, RequestContext, exigir};
use crate::presentation::error::ApiError;
use crate::presentation::usuarios::dto::{
    AtualizarUsuarioRequest, AcessoUsuarioDto, CriarUsuarioRequest, DefinirAcessoRequest,
    DefinirCarteiraRequest, ListaUsuariosResponse, ListarUsuariosQuery, RedefinirSenhaRequest,
    UsuarioDto,
};

/// Estado dos routers de usuários.
#[derive(Clone)]
pub struct Estado {
    pub gestao: Arc<GestaoUsuariosUseCase>,
    pub acesso: Arc<GestaoAcessoUsuariosUseCase>,
    pub verificar: Arc<VerificarAcessoUseCase>,
}

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

/// Fase E — gestão de função/carteira exige papel `CLIENTE` e função GESTOR
/// (efeetiva, lida do banco por requisição).
fn ator_gestor(
    ctx: &RequestContext,
    verificar: &Arc<VerificarAcessoUseCase>,
) -> Result<Ator, ApiError> {
    let ator = ator_portal(ctx)?;
    let escritorio = ator.escopo().ok_or_else(ApiError::proibido)?;
    let acesso = verificar
        .executar(escritorio, ctx.usuario_id, ctx.papel)
        .map_err(ApiError::from)?;
    if !acesso.funcao.administra_usuarios() {
        return Err(ApiError::proibido());
    }
    Ok(ator)
}

async fn listar(
    State(estado): State<Estado>,
    Extension(ctx): Extension<RequestContext>,
    Query(q): Query<ListarUsuariosQuery>,
    admin: bool,
) -> Result<Json<ListaUsuariosResponse>, ApiError> {
    let ator = if admin {
        ator_admin(&ctx)?
    } else {
        ator_portal(&ctx)?
    };
    let listagem = estado
        .gestao
        .listar_paginado(
            &ator,
            FiltroUsuarios {
                escritorio_id: q.escritorio_id,
                busca: q.busca,
                ativo: q.ativo,
                pagina: q.pagina,
                tamanho: q.tamanho,
            },
        )?;
    Ok(Json(ListaUsuariosResponse {
        itens: listagem.itens.into_iter().map(Into::into).collect(),
        total: listagem.total,
        pagina: listagem.pagina,
        tamanho: listagem.tamanho,
    }))
}

async fn criar(
    State(estado): State<Estado>,
    Extension(ctx): Extension<RequestContext>,
    Json(req): Json<CriarUsuarioRequest>,
    admin: bool,
) -> Result<(StatusCode, Json<UsuarioDto>), ApiError> {
    let ator = if admin {
        ator_admin(&ctx)?
    } else {
        ator_portal(&ctx)?
    };
    let criado = estado.gestao.criar(
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
    State(estado): State<Estado>,
    Extension(ctx): Extension<RequestContext>,
    Path(id): Path<Uuid>,
    admin: bool,
) -> Result<Json<UsuarioDto>, ApiError> {
    let ator = if admin {
        ator_admin(&ctx)?
    } else {
        ator_portal(&ctx)?
    };
    Ok(Json(estado.gestao.obter(&ator, id)?.into()))
}

async fn atualizar(
    State(estado): State<Estado>,
    Extension(ctx): Extension<RequestContext>,
    Path(id): Path<Uuid>,
    Json(req): Json<AtualizarUsuarioRequest>,
    admin: bool,
) -> Result<Json<UsuarioDto>, ApiError> {
    let ator = if admin {
        ator_admin(&ctx)?
    } else {
        ator_portal(&ctx)?
    };
    let novo = estado.gestao.atualizar(
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
    State(estado): State<Estado>,
    Extension(ctx): Extension<RequestContext>,
    Path(id): Path<Uuid>,
    Json(req): Json<RedefinirSenhaRequest>,
    admin: bool,
) -> Result<StatusCode, ApiError> {
    let ator = if admin {
        ator_admin(&ctx)?
    } else {
        ator_portal(&ctx)?
    };
    estado.gestao.redefinir_senha(&ator, id, &req.senha)?;
    Ok(StatusCode::NO_CONTENT)
}

// ===== Fase E — função e carteira do usuário (portal, função GESTOR) =====

async fn obter_acesso(
    State(estado): State<Estado>,
    Extension(ctx): Extension<RequestContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<AcessoUsuarioDto>, ApiError> {
    let ator = ator_gestor(&ctx, &estado.verificar)?;
    let alvo = estado.gestao.obter(&ator, id)?;
    let detalhe = estado
        .acesso
        .detalhe(alvo.escritorio_id, alvo.id, alvo.papel)
        .map_err(ApiError::from)?;
    Ok(Json(AcessoUsuarioDto {
        funcao: detalhe.funcao.como_str().to_owned(),
        pode_administrar_usuarios: detalhe.funcao.administra_usuarios(),
        clientes_permitidos: detalhe.clientes_permitidos,
    }))
}

async fn definir_acesso(
    State(estado): State<Estado>,
    Extension(ctx): Extension<RequestContext>,
    Path(id): Path<Uuid>,
    Json(req): Json<DefinirAcessoRequest>,
) -> Result<Json<AcessoUsuarioDto>, ApiError> {
    let ator = ator_gestor(&ctx, &estado.verificar)?;
    let alvo = estado.gestao.obter(&ator, id)?;
    if alvo.papel == Papel::Admin {
        return Err(ApiError::proibido());
    }
    let funcao = FuncaoUsuario::de_str(&req.funcao)
        .ok_or_else(|| ApiError::validacao("função deve ser GESTOR, ANALISTA_FISCAL ou OPERADOR_LEITURA"))?;
    estado
        .acesso
        .definir_funcao(alvo.escritorio_id, alvo.id, funcao)
        .map_err(ApiError::from)?;
    let detalhe = estado
        .acesso
        .detalhe(alvo.escritorio_id, alvo.id, alvo.papel)
        .map_err(ApiError::from)?;
    Ok(Json(AcessoUsuarioDto {
        funcao: detalhe.funcao.como_str().to_owned(),
        pode_administrar_usuarios: detalhe.funcao.administra_usuarios(),
        clientes_permitidos: detalhe.clientes_permitidos,
    }))
}

async fn definir_carteira(
    State(estado): State<Estado>,
    Extension(ctx): Extension<RequestContext>,
    Path(id): Path<Uuid>,
    Json(req): Json<DefinirCarteiraRequest>,
) -> Result<Json<AcessoUsuarioDto>, ApiError> {
    let ator = ator_gestor(&ctx, &estado.verificar)?;
    let alvo = estado.gestao.obter(&ator, id)?;
    if alvo.papel == Papel::Admin {
        return Err(ApiError::proibido());
    }
    estado
        .acesso
        .definir_carteira(alvo.escritorio_id, alvo.id, &req.clientes)
        .map_err(ApiError::from)?;
    let detalhe = estado
        .acesso
        .detalhe(alvo.escritorio_id, alvo.id, alvo.papel)
        .map_err(ApiError::from)?;
    Ok(Json(AcessoUsuarioDto {
        funcao: detalhe.funcao.como_str().to_owned(),
        pode_administrar_usuarios: detalhe.funcao.administra_usuarios(),
        clientes_permitidos: detalhe.clientes_permitidos,
    }))
}

macro_rules! rotas {
    ($admin:expr) => {
        Router::new()
            .route(
                "/",
                get(
                    |s: State<Estado>,
                     c: Extension<RequestContext>,
                     q: Query<ListarUsuariosQuery>| { listar(s, c, q, $admin) },
                )
                .post(
                    |s: State<Estado>,
                     c: Extension<RequestContext>,
                     r: Json<CriarUsuarioRequest>| criar(s, c, r, $admin),
                ),
            )
            .route(
                "/{id}",
                get(
                    |s: State<Estado>, c: Extension<RequestContext>, id: Path<Uuid>| {
                        obter(s, c, id, $admin)
                    },
                )
                .patch(
                    |s: State<Estado>,
                     c: Extension<RequestContext>,
                     id: Path<Uuid>,
                     r: Json<AtualizarUsuarioRequest>| atualizar(s, c, id, r, $admin),
                ),
            )
            .route(
                "/{id}/senha",
                post(
                    |s: State<Estado>,
                     c: Extension<RequestContext>,
                     id: Path<Uuid>,
                     r: Json<RedefinirSenhaRequest>| senha(s, c, id, r, $admin),
                ),
            )
    };
}

fn proteger(router: Router<Estado>, auth: &AuthState, audiencia: Audiencia, estado: Estado) -> Router {
    router
        .route_layer(axum_middleware::from_fn_with_state(
            ExigirAudiencia::novo(auth.tokens.clone(), audiencia),
            exigir,
        ))
        .with_state(estado)
}

/// `/usuarios` — portal do escritório (audiência client), com gestão de
/// função/carteira (Fase E, função GESTOR).
pub fn construir_usuarios(
    gestao: GestaoUsuariosUseCase,
    acesso: GestaoAcessoUsuariosUseCase,
    verificar: VerificarAcessoUseCase,
    auth: &AuthState,
) -> Router {
    let estado = Estado {
        gestao: Arc::new(gestao),
        acesso: Arc::new(acesso),
        verificar: Arc::new(verificar),
    };
    let rotas = rotas!(false)
        .route(
            "/{id}/acesso",
            get(obter_acesso).put(definir_acesso),
        )
        .route("/{id}/carteira", axum::routing::put(definir_carteira));
    proteger(rotas, auth, Audiencia::Client, estado)
}

/// Rotas `/usuarios` do backoffice, para mesclar sob `/admin`.
pub fn construir_usuarios_admin(
    gestao: GestaoUsuariosUseCase,
    acesso: GestaoAcessoUsuariosUseCase,
    verificar: VerificarAcessoUseCase,
    auth: &AuthState,
) -> Router {
    let estado = Estado {
        gestao: Arc::new(gestao),
        acesso: Arc::new(acesso),
        verificar: Arc::new(verificar),
    };
    proteger(rotas!(true), auth, Audiencia::Admin, estado)
}
