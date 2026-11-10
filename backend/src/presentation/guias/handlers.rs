//! Fase D — Handlers de emissão de guias.

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use uuid::Uuid;

use crate::domain::guias::enums::TipoGuia;
use crate::domain::guias::erros::ErroGuia;
use crate::presentation::auth::RequestContext;
use crate::presentation::error::ApiError;
use crate::presentation::guias::dto::{RequisicaoEmitirGuia, RespostaGuiaEmissao};
use crate::presentation::guias::state::EstadoGuias;

/// POST /guias — emitir uma guia (DAS ou DARF).
pub async fn emitir_guia(
    State(estado): State<EstadoGuias>,
    Extension(contexto): Extension<RequestContext>,
    Json(requisicao): Json<RequisicaoEmitirGuia>,
) -> Result<Json<RespostaGuiaEmissao>, ApiError> {
    let tipo_guia = match requisicao.tipo_guia.as_str() {
        "DAS" => TipoGuia::Das,
        "DARF" => TipoGuia::Darf,
        _ => return Err(ApiError::validacao("tipo de guia inválido")),
    };

    let guia = estado
        .emitir_guia
        .executar(crate::domain::guias::entidades::NovoGuiaEmissao {
            usuario_id: contexto.usuario_id,
            escritorio_id: contexto.escritorio_id.ok_or_else(|| {
                ApiError::nova(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "CONTEXTO_TOKEN_INVALIDO",
                    "Token sem escritório",
                )
            })?,
            cliente_id: requisicao.cliente_id,
            tipo_guia,
            competencia: requisicao.competencia,
            valor_total: requisicao.valor_total,
            idempotencia_chave: requisicao.idempotencia_chave,
        })
        .await
        .map_err(mapear_erro_guia)?;

    Ok(Json(RespostaGuiaEmissao::from(guia)))
}

/// GET /guias — listar guias do usuário.
pub async fn listar_guias(
    State(estado): State<EstadoGuias>,
    Extension(contexto): Extension<RequestContext>,
) -> Result<Json<Vec<RespostaGuiaEmissao>>, ApiError> {
    let usuario_id = contexto.usuario_id;
    let limite = 50;

    let guias = estado
        .listar_guias
        .executar(usuario_id, limite)
        .await
        .map_err(mapear_erro_guia)?;

    Ok(Json(
        guias.into_iter().map(RespostaGuiaEmissao::from).collect(),
    ))
}

/// GET /guias/{id} — buscar uma guia por ID.
pub async fn buscar_guia(
    State(estado): State<EstadoGuias>,
    Extension(contexto): Extension<RequestContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<RespostaGuiaEmissao>, ApiError> {
    let usuario_id = contexto.usuario_id;
    let guia = estado
        .buscar_guia
        .executar(id)
        .await
        .map_err(mapear_erro_guia)?
        .ok_or_else(|| {
            ApiError::nova(
                StatusCode::NOT_FOUND,
                "GUIA_NAO_ENCONTRADA",
                "Guia não encontrada",
            )
        })?;

    if guia.usuario_id != usuario_id {
        return Err(ApiError::nova(
            StatusCode::NOT_FOUND,
            "GUIA_NAO_ENCONTRADA",
            "Guia não encontrada",
        ));
    }

    Ok(Json(RespostaGuiaEmissao::from(guia)))
}

/// POST /guias/{id}/baixar — marcar guia como baixada.
pub async fn baixar_guia(
    State(estado): State<EstadoGuias>,
    Extension(contexto): Extension<RequestContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<RespostaGuiaEmissao>, ApiError> {
    let usuario_id = contexto.usuario_id;
    let guia = estado
        .baixar_guia
        .executar(id)
        .await
        .map_err(mapear_erro_guia)?;

    if guia.usuario_id != usuario_id {
        return Err(ApiError::nova(
            StatusCode::NOT_FOUND,
            "GUIA_NAO_ENCONTRADA",
            "Guia não encontrada",
        ));
    }

    Ok(Json(RespostaGuiaEmissao::from(guia)))
}

fn mapear_erro_guia(erro: ErroGuia) -> ApiError {
    match erro {
        ErroGuia::GuiaNaoEncontrada => ApiError::nova(
            StatusCode::NOT_FOUND,
            "GUIA_NAO_ENCONTRADA",
            "Guia não encontrada.",
        ),
        ErroGuia::JaExisteIdempotencia => ApiError::nova(
            StatusCode::CONFLICT,
            "GUIA_JA_EXISTE_IDEMPOTENCIA",
            "Já existe uma emissão com esta chave de idempotência.",
        ),
        ErroGuia::NaoPodeBaixar(status) => ApiError::nova(
            StatusCode::BAD_REQUEST,
            "NAO_PODE_BAIXAR",
            format!("Não é possível baixar guia com status {status}."),
        ),
        ErroGuia::ProvedorIndisponivel(msg) => ApiError::nova(
            StatusCode::SERVICE_UNAVAILABLE,
            "PROVEDOR_INDISPONIVEL",
            &msg,
        ),
        ErroGuia::Persistencia(msg) => {
            ApiError::nova(StatusCode::INTERNAL_SERVER_ERROR, "PERSISTENCIA", &msg)
        }
        ErroGuia::Interno(msg) => {
            ApiError::nova(StatusCode::INTERNAL_SERVER_ERROR, "ERRO_INTERNO", &msg)
        }
    }
}

use axum::Extension;
