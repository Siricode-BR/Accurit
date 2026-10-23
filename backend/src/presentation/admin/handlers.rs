use axum::{
    extract::{Extension, Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::presentation::admin::dto::{
    EscritorioResumoResponse, ProvisionarTenantRequest, ProvisionarTenantResponse,
};
use crate::presentation::auth::RequestContext;
use crate::presentation::error::ApiError;
use crate::{domain::auth::Papel, presentation::admin::state::AdminState};

pub async fn provisionar_tenant(
    State(state): State<AdminState>,
    Extension(context): Extension<RequestContext>,
    Json(payload): Json<ProvisionarTenantRequest>,
) -> Result<Response, ApiError> {
    if context.papel != Papel::Admin {
        return Err(ApiError::proibido());
    }

    let comando = crate::application::admin::ProvisionarTenantCommand {
        slug: payload.slug,
        nome: payload.nome,
        email_titular: payload.email_titular,
        cpf_titular: payload.cpf_titular,
        senha_titular: payload.senha_titular,
        ip: context.ip,
        administrador_id: context.usuario_id,
    };

    let resultado = state.provisionar_tenant.executar(comando).await?;
    Ok((
        StatusCode::CREATED,
        Json(ProvisionarTenantResponse {
            escritorio_id: resultado.escritorio_id,
            slug: resultado.slug,
            titular_id: resultado.titular_id,
            email_titular: resultado.email_titular,
        }),
    )
        .into_response())
}

/// GET /api/v1/admin/escritorios — listagem de tenants do backoffice,
/// pré-requisito da tela do toggle SERPRO (Fase 3; remediação mínima de
/// uma lacuna da Fase 2B documentada no ROADMAP).
pub async fn listar_escritorios(
    State(state): State<AdminState>,
    Extension(context): Extension<RequestContext>,
) -> Result<Response, ApiError> {
    if context.papel != Papel::Admin {
        return Err(ApiError::proibido());
    }

    let resumos = state
        .listar_escritorios
        .executar()
        .map_err(|erro| match erro {
            crate::application::admin::RepositorioError::Indisponivel => ApiError::indisponivel(),
            _ => ApiError::erro_interno(),
        })?;

    let corpo: Vec<EscritorioResumoResponse> = resumos.into_iter().map(Into::into).collect();
    Ok((StatusCode::OK, Json(corpo)).into_response())
}
