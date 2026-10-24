use axum::{
    extract::{Extension, Json, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::application::admin::Paginacao;
use crate::presentation::admin::dto::{
    EscritoriosPaginadosResponse, PaginacaoQuery, ProvisionarTenantRequest,
    ProvisionarTenantResponse,
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

/// GET /api/v1/admin/escritorios?page=...&pageSize=... — lista paginada de
/// escritórios com apenas metadados (id, slug, nome, modoTrialSerpro,
/// versao, atualizadoEm), fonte da tela do toggle SERPRO da Fase 3.
pub async fn listar_escritorios(
    State(state): State<AdminState>,
    Extension(context): Extension<RequestContext>,
    Query(paginacao): Query<PaginacaoQuery>,
) -> Result<Response, ApiError> {
    if context.papel != Papel::Admin {
        return Err(ApiError::proibido());
    }

    let janela = Paginacao::nova(
        paginacao.page.unwrap_or(1),
        paginacao.page_size.unwrap_or(Paginacao::POR_PAGINA_PADRAO),
    );

    let (resumos, total) =
        state
            .listar_escritorios
            .executar(&janela)
            .map_err(|erro| match erro {
                crate::application::admin::RepositorioError::Indisponivel => {
                    ApiError::indisponivel()
                }
                _ => ApiError::erro_interno(),
            })?;

    let itens = resumos.into_iter().map(Into::into).collect();
    Ok((
        StatusCode::OK,
        Json(EscritoriosPaginadosResponse {
            itens,
            page: janela.pagina,
            page_size: janela.por_pagina,
            total,
        }),
    )
        .into_response())
}
