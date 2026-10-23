use axum::extract::{Extension, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::application::serpro::ObterConfiguracaoSerproCommand;
use crate::presentation::auth::RequestContext;
use crate::presentation::error::ApiError;
use crate::presentation::serpro::dto::SerproConfiguracaoResponse;
use crate::presentation::serpro::state::SerproState;

/// GET /api/v1/serpro/configuracao — leitura do ambiente ativo do
/// escritório do token do portal (SPEC §10.2): campo de exibição, sem
/// troca de modo e sem segredos na resposta.
pub async fn obter_configuracao(
    State(estado): State<SerproState>,
    Extension(contexto): Extension<RequestContext>,
) -> Result<Response, ApiError> {
    let escritorio_id = contexto.escritorio_id.ok_or_else(|| {
        ApiError::erro_interno().com_detalhe("escritorio_id ausente no token do portal")
    })?;

    let resultado = estado
        .obter_configuracao
        .executar(ObterConfiguracaoSerproCommand { escritorio_id })
        .await
        .map_err(map_obter_para_api)?;

    Ok((
        StatusCode::OK,
        axum::Json(SerproConfiguracaoResponse {
            escritorio_id: resultado.escritorio_id,
            modo_trial_serpro: resultado.modo_trial_serpro,
            ambiente_efetivo: resultado.ambiente_efetivo,
            versao: resultado.versao,
            atualizado_em: resultado.atualizado_em,
        }),
    )
        .into_response())
}

pub(crate) fn map_obter_para_api(
    erro: crate::application::serpro::ObterConfiguracaoError,
) -> ApiError {
    use crate::application::serpro::ObterConfiguracaoError;
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
