use axum::extract::{Extension, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::application::serpro::ObterConfiguracaoError;
use crate::presentation::auth::RequestContext;
use crate::presentation::error::ApiError;
use crate::presentation::serpro::ambiente::AmbienteResolvido;
use crate::presentation::serpro::dto::AmbienteResponse;
use crate::presentation::serpro::state::SerproState;

/// GET /api/v1/serpro/ambiente — leitura do ambiente efetivo do
/// escritório do token do portal. O ambiente foi resolvido **uma vez** pelo
/// middleware `resolver_ambiente` e vive na extensão `AmbienteResolvido`
/// (snapshot imutável da requisição; D8 de DECISOES-PENDENTES).
pub async fn obter_ambiente(
    State(_estado): State<SerproState>,
    Extension(contexto): Extension<RequestContext>,
    Extension(ambiente): Extension<AmbienteResolvido>,
) -> Result<Response, ApiError> {
    let _ = contexto.escritorio_id; // o snapshot já refletiu o token do portal

    Ok((
        StatusCode::OK,
        axum::Json(AmbienteResponse {
            ambiente_efetivo: ambiente.ambiente,
            atualizado_em: ambiente.atualizado_em,
        }),
    )
        .into_response())
}

/// Mapeamento compartilhado de `ObterConfiguracaoError` para erros HTTP —
/// usado pelo middleware de snapshot e por quem chama o use case diretamente.
pub(crate) fn map_obter_para_api(erro: ObterConfiguracaoError) -> ApiError {
    match erro {
        ObterConfiguracaoError::EscritorioNaoEncontrado => ApiError::nao_encontrado("escritório"),
        ObterConfiguracaoError::Repositorio(repositorio) => map_repositorio(repositorio),
    }
}

pub(crate) fn map_repositorio(
    erro: crate::application::serpro::ports::RepositorioError,
) -> ApiError {
    use crate::application::serpro::ports::RepositorioError;
    match erro {
        RepositorioError::Indisponivel => ApiError::indisponivel(),
        RepositorioError::NaoEncontrado => ApiError::nao_encontrado("escritório"),
        _ => ApiError::erro_interno(),
    }
}
