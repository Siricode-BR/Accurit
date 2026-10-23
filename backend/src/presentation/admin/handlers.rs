use axum::{
    extract::{Extension, Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::application::admin::ProvisionarTenantUseCase;
use crate::domain::auth::Papel;
use crate::presentation::admin::dto::{ProvisionarTenantRequest, ProvisionarTenantResponse};
use crate::presentation::auth::RequestContext;
use crate::presentation::error::ApiError;

pub async fn provisionar_tenant(
    State(use_case): State<ProvisionarTenantUseCase>,
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

    let resultado = use_case.executar(comando).await?;
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
