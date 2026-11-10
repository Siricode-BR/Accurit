use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::Serialize;

use crate::application::serpro::ObterConfiguracaoError;
use crate::domain::serpro::ambiente::AmbienteSerpro;
use crate::infrastructure::serpro::mock::{self, mock_ativo};
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
        Json(AmbienteResponse {
            ambiente_efetivo: ambiente.ambiente,
            atualizado_em: ambiente.atualizado_em,
        }),
    )
        .into_response())
}

/// GET /api/v1/serpro/mock/cenarios — cenários de teste do simulador.
/// Fora de TRIAL ou com o mock desativado responde `ativo=false` e lista vazia.
pub async fn listar_cenarios_mock(
    State(estado): State<SerproState>,
    Extension(ambiente): Extension<AmbienteResolvido>,
) -> Json<CenariosMockResponse> {
    let ativo = mock_ativo(ambiente.ambiente, estado.mock_ativo);
    let cenarios = if ativo {
        mock::cenarios()
            .into_iter()
            .map(|cenario| CenarioMockResponse {
                id: cenario.id.to_owned(),
                servico: cenario.servico.como_str().to_owned(),
                titulo: cenario.titulo.to_owned(),
                descricao: cenario.descricao.to_owned(),
                documento_exemplo: cenario.documento_exemplo.to_owned(),
                resultado_esperado: cenario.resultado_esperado.to_owned(),
            })
            .collect()
    } else {
        Vec::new()
    };

    Json(CenariosMockResponse { ativo, cenarios })
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CenarioMockResponse {
    pub id: String,
    pub servico: String,
    pub titulo: String,
    pub descricao: String,
    pub documento_exemplo: String,
    pub resultado_esperado: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CenariosMockResponse {
    pub ativo: bool,
    pub cenarios: Vec<CenarioMockResponse>,
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

/// Helper mantido para documentar que o mock só existe em TRIAL.
pub const fn ambiente_permite_mock(ambiente: AmbienteSerpro) -> bool {
    matches!(ambiente, AmbienteSerpro::Trial)
}
