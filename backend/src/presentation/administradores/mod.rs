//! Etapa 0.2 — rotas de gestão de administradores do backoffice
//! (`/api/v1/administradores`). Somente papel `ADMIN` da plataforma.

use axum::{
    Router,
    extract::{Extension, Json, Path, Query, State},
    http::StatusCode,
    middleware as axum_middleware,
    response::{IntoResponse, Response},
    routing::get,
};
use serde::{Deserialize, Serialize};

use crate::application::admin::{GestaoAdministradoresUseCase, RepositorioError};
use crate::domain::auth::{Audiencia, Papel};
use crate::presentation::auth::{AuthState, ExigirAudiencia, RequestContext, exigir};
use crate::presentation::error::ApiError;
use std::sync::Arc;

const POR_PAGINA_PADRAO: usize = 20;

/// Payloads / respostas da API (camelCase para o front admin).

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdministradorCriarRequest {
    pub nome: String,
    pub email: String,
    pub cpf: String,
    pub senha: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdministradorAtualizarRequest {
    #[serde(default)]
    pub nome: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub ativo: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdministradorAlterarSenhaRequest {
    pub senha: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdministradorResponse {
    pub id: String,
    pub nome: String,
    pub email: String,
    pub cpf: String,
    pub ativo: bool,
    pub criado_em: String,
    pub atualizado_em: String,
    pub ultimo_login_em: Option<String>,
}

impl From<crate::application::admin::AdministradorDto> for AdministradorResponse {
    fn from(dto: crate::application::admin::AdministradorDto) -> Self {
        let formato = "%Y-%m-%dT%H:%M:%S%.3fZ";
        Self {
            id: dto.id.to_string(),
            nome: dto.nome,
            email: dto.email,
            cpf: dto.cpf,
            ativo: dto.ativo,
            criado_em: dto.criado_em.format(formato).to_string(),
            atualizado_em: dto.atualizado_em.format(formato).to_string(),
            ultimo_login_em: dto.ultimo_login_em.map(|t| t.format(formato).to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdministradoresPaginadosResponse {
    pub itens: Vec<AdministradorResponse>,
    pub page: usize,
    pub page_size: usize,
    pub total: usize,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListaAdministradoresQuery {
    #[serde(default)]
    pub page: Option<usize>,
    #[serde(default)]
    pub page_size: Option<usize>,
    #[serde(default)]
    pub q: Option<String>,
}

/// Erros de autorização específicos desta rota.
fn exigir_admin(context: &RequestContext) -> Result<(), ApiError> {
    if context.papel != Papel::Admin {
        return Err(ApiError::proibido());
    }
    Ok(())
}

fn erro_repositorio(erro: RepositorioError) -> ApiError {
    match erro {
        RepositorioError::Indisponivel => ApiError::indisponivel(),
        RepositorioError::Validacao(msg) => ApiError::validacao(msg),
        RepositorioError::EscritorioExistente => {
            ApiError::conflito("Escritório com este CNPJ já existe")
        }
        RepositorioError::UsuarioExistente => {
            ApiError::conflito("Já existe um administrador com este e-mail")
        }
        RepositorioError::AutoDesativacao => {
            ApiError::conflito("Você não pode desativar o seu próprio acesso")
        }
        RepositorioError::UltimoAdministradorAtivo => {
            ApiError::conflito("Não é possível desativar o último administrador ativo")
        }
        RepositorioError::Dados => ApiError::erro_interno(),
    }
}

#[derive(Clone)]
struct EstadoAdministradores {
    gestao: Arc<GestaoAdministradoresUseCase>,
}

pub fn construir_administradores(
    gestao: GestaoAdministradoresUseCase,
    auth_state: AuthState,
) -> Router {
    let auth = ExigirAudiencia::novo(auth_state.tokens.clone(), Audiencia::Admin);

    Router::new()
        .route("/", get(listar).post(criar))
        .route("/{id}", get(obter).patch(atualizar).put(alterar_senha))
        .route_layer(axum_middleware::from_fn_with_state(auth, exigir))
        .with_state(EstadoAdministradores {
            gestao: Arc::new(gestao),
        })
}

async fn listar(
    State(estado): State<EstadoAdministradores>,
    Extension(context): Extension<RequestContext>,
    Query(params): Query<ListaAdministradoresQuery>,
) -> Result<Response, ApiError> {
    exigir_admin(&context)?;

    let pagina = params.page.unwrap_or(1);
    let por_pagina = params.page_size.unwrap_or(POR_PAGINA_PADRAO);
    let (itens, total) = estado
        .gestao
        .listar(pagina, por_pagina, params.q.as_deref())
        .await
        .map_err(erro_repositorio)?;

    Ok((
        StatusCode::OK,
        Json(AdministradoresPaginadosResponse {
            itens: itens.into_iter().map(AdministradorResponse::from).collect(),
            page: pagina,
            page_size: por_pagina,
            total,
        }),
    )
        .into_response())
}

async fn obter(
    State(estado): State<EstadoAdministradores>,
    Extension(context): Extension<RequestContext>,
    Path(id): Path<String>,
) -> Result<Response, ApiError> {
    exigir_admin(&context)?;

    let dto = estado
        .gestao
        .buscar_por_id(&id)
        .await
        .map_err(erro_repositorio)?
        .ok_or_else(|| ApiError::nao_encontrado("administrador"))?;

    Ok((StatusCode::OK, Json(AdministradorResponse::from(dto))).into_response())
}

async fn criar(
    State(estado): State<EstadoAdministradores>,
    Extension(context): Extension<RequestContext>,
    Json(payload): Json<AdministradorCriarRequest>,
) -> Result<Response, ApiError> {
    exigir_admin(&context)?;

    let novo = crate::application::admin::AdministradorNovo {
        nome: payload.nome,
        email: payload.email,
        cpf: payload.cpf,
        senha: payload.senha,
    };
    let dto = estado
        .gestao
        .criar(&novo, context.usuario_id, context.ip)
        .await
        .map_err(erro_repositorio)?;

    Ok((StatusCode::CREATED, Json(AdministradorResponse::from(dto))).into_response())
}

async fn atualizar(
    State(estado): State<EstadoAdministradores>,
    Extension(context): Extension<RequestContext>,
    Path(id): Path<String>,
    Json(payload): Json<AdministradorAtualizarRequest>,
) -> Result<Response, ApiError> {
    exigir_admin(&context)?;

    let mudancas = crate::application::admin::AdministradorAtualizar {
        nome: payload.nome,
        email: payload.email,
        ativo: payload.ativo,
    };
    let dto = estado
        .gestao
        .atualizar(&id, &mudancas, context.usuario_id, context.ip)
        .await
        .map_err(erro_repositorio)?;

    Ok((StatusCode::OK, Json(AdministradorResponse::from(dto))).into_response())
}

async fn alterar_senha(
    State(estado): State<EstadoAdministradores>,
    Extension(context): Extension<RequestContext>,
    Path(id): Path<String>,
    Json(payload): Json<AdministradorAlterarSenhaRequest>,
) -> Result<Response, ApiError> {
    exigir_admin(&context)?;

    let dto = estado
        .gestao
        .alterar_senha(&id, &payload.senha, context.usuario_id, context.ip)
        .await
        .map_err(erro_repositorio)?;

    Ok((StatusCode::OK, Json(AdministradorResponse::from(dto))).into_response())
}
