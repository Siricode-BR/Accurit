//! LGPD — exportação, anonimização, retenção e consentimento.

use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Extension, Path, State},
    http::{StatusCode, header},
    middleware as axum_middleware,
    response::{IntoResponse, Response},
    routing::get,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::auth::{Audiencia, Papel};
use crate::infrastructure::persistence::lgpd::{ErroLgpd, LgpdDiesel, PoliticaRetencao};
use crate::presentation::auth::{AuthState, ExigirAudiencia, RequestContext, exigir};
use crate::presentation::error::ApiError;

#[derive(Clone)]
pub struct EstadoLgpd {
    pub repo: Arc<LgpdDiesel>,
}

fn erro_lgpd(erro: ErroLgpd) -> ApiError {
    match erro {
        ErroLgpd::NaoEncontrado => ApiError::nao_encontrado("cliente"),
        ErroLgpd::Validacao(mensagem) => ApiError::validacao(mensagem),
        ErroLgpd::Persistencia => ApiError::erro_interno(),
    }
}

fn escritorio(contexto: &RequestContext) -> Result<Uuid, ApiError> {
    contexto.escritorio_id.ok_or_else(ApiError::proibido)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RetencaoResponse {
    pub consultas_dias: i32,
    pub relatorios_dias: i32,
    pub auditoria_dias: i32,
}

impl From<PoliticaRetencao> for RetencaoResponse {
    fn from(p: PoliticaRetencao) -> Self {
        Self {
            consultas_dias: p.consultas_dias,
            relatorios_dias: p.relatorios_dias,
            auditoria_dias: p.auditoria_dias,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetencaoRequest {
    pub consultas_dias: i32,
    pub relatorios_dias: i32,
    pub auditoria_dias: i32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsentimentoResponse {
    pub id: Uuid,
    pub tipo: String,
    pub versao: String,
    pub aceito_em: chrono::DateTime<chrono::Utc>,
    pub revogado_em: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistrarConsentimentoRequest {
    pub tipo: String,
    #[serde(default = "versao_padrao")]
    pub versao: String,
}

fn versao_padrao() -> String {
    "1.0".to_owned()
}

/// GET /api/v1/lgpd/clientes/{id}/export — portabilidade em JSON.
pub async fn exportar_cliente(
    State(estado): State<EstadoLgpd>,
    Extension(contexto): Extension<RequestContext>,
    Path(cliente_id): Path<Uuid>,
) -> Result<Response, ApiError> {
    let escritorio_id = escritorio(&contexto)?;
    let dados = estado
        .repo
        .exportar_cliente(escritorio_id, cliente_id)
        .map_err(erro_lgpd)?;
    let corpo = serde_json::to_vec_pretty(&dados).map_err(|_| ApiError::erro_interno())?;
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json; charset=utf-8")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"lgpd-cliente-{cliente_id}.json\""),
        )
        .body(axum::body::Body::from(corpo))
        .map_err(|_| ApiError::erro_interno())
}

/// DELETE /api/v1/lgpd/clientes/{id} — anonimização com trilha de auditoria.
pub async fn anonimizar_cliente(
    State(estado): State<EstadoLgpd>,
    Extension(contexto): Extension<RequestContext>,
    Path(cliente_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    if contexto.papel == Papel::Cliente {
        return Err(ApiError::proibido());
    }
    let escritorio_id = escritorio(&contexto)?;
    estado
        .repo
        .anonimizar_cliente(escritorio_id, cliente_id, contexto.usuario_id, contexto.ip)
        .map_err(erro_lgpd)?;
    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/v1/lgpd/retencao
pub async fn obter_retencao(
    State(estado): State<EstadoLgpd>,
    Extension(contexto): Extension<RequestContext>,
) -> Result<Json<RetencaoResponse>, ApiError> {
    let escritorio_id = escritorio(&contexto)?;
    Ok(Json(
        estado
            .repo
            .obter_retencao(escritorio_id)
            .map_err(erro_lgpd)?
            .into(),
    ))
}

/// PUT /api/v1/lgpd/retencao
pub async fn salvar_retencao(
    State(estado): State<EstadoLgpd>,
    Extension(contexto): Extension<RequestContext>,
    Json(payload): Json<RetencaoRequest>,
) -> Result<Json<RetencaoResponse>, ApiError> {
    if contexto.papel == Papel::Cliente {
        return Err(ApiError::proibido());
    }
    let escritorio_id = escritorio(&contexto)?;
    let politica = PoliticaRetencao {
        consultas_dias: payload.consultas_dias,
        relatorios_dias: payload.relatorios_dias,
        auditoria_dias: payload.auditoria_dias,
    };
    estado
        .repo
        .salvar_retencao(escritorio_id, &politica)
        .map_err(erro_lgpd)?;
    Ok(Json(politica.into()))
}

/// GET /api/v1/lgpd/clientes/{id}/consentimentos
pub async fn listar_consentimentos(
    State(estado): State<EstadoLgpd>,
    Extension(contexto): Extension<RequestContext>,
    Path(cliente_id): Path<Uuid>,
) -> Result<Json<Vec<ConsentimentoResponse>>, ApiError> {
    let escritorio_id = escritorio(&contexto)?;
    let itens = estado
        .repo
        .listar_consentimentos(escritorio_id, cliente_id)
        .map_err(erro_lgpd)?;
    Ok(Json(
        itens
            .into_iter()
            .map(|c| ConsentimentoResponse {
                id: c.id,
                tipo: c.tipo,
                versao: c.versao,
                aceito_em: c.aceito_em,
                revogado_em: c.revogado_em,
            })
            .collect(),
    ))
}

/// POST /api/v1/lgpd/clientes/{id}/consentimentos
pub async fn registrar_consentimento(
    State(estado): State<EstadoLgpd>,
    Extension(contexto): Extension<RequestContext>,
    Path(cliente_id): Path<Uuid>,
    Json(payload): Json<RegistrarConsentimentoRequest>,
) -> Result<Response, ApiError> {
    let escritorio_id = escritorio(&contexto)?;
    let consentimento = estado
        .repo
        .registrar_consentimento(
            escritorio_id,
            cliente_id,
            &payload.tipo,
            &payload.versao,
            contexto.ip,
        )
        .map_err(erro_lgpd)?;
    Ok((
        StatusCode::CREATED,
        Json(ConsentimentoResponse {
            id: consentimento.id,
            tipo: consentimento.tipo,
            versao: consentimento.versao,
            aceito_em: consentimento.aceito_em,
            revogado_em: consentimento.revogado_em,
        }),
    )
        .into_response())
}

pub fn construir(estado: EstadoLgpd, auth: AuthState) -> Router {
    Router::new()
        .route("/lgpd/clientes/{id}/export", get(exportar_cliente))
        .route(
            "/lgpd/clientes/{id}",
            axum::routing::delete(anonimizar_cliente),
        )
        .route(
            "/lgpd/clientes/{id}/consentimentos",
            get(listar_consentimentos).post(registrar_consentimento),
        )
        .route("/lgpd/retencao", get(obter_retencao).put(salvar_retencao))
        .route_layer(axum_middleware::from_fn_with_state(
            ExigirAudiencia::novo(auth.tokens.clone(), Audiencia::Client),
            exigir,
        ))
        .with_state(estado)
}
