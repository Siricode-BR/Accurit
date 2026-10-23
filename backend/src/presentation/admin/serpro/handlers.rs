use axum::extract::{Extension, Json, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::application::serpro::{
    AlterarModoTrialSerproCommand, ObterConfiguracaoSerproCommand, ports::RepositorioError,
};
use crate::domain::auth::Papel;
use crate::domain::serpro::erros::ErroAmbiente;
use crate::presentation::admin::state::AdminState;
use crate::presentation::auth::RequestContext;
use crate::presentation::error::ApiError;
use crate::presentation::serpro::dto::{
    AlterarModoTrialRequest, AlterarModoTrialResponse, ObterConfiguracaoQuery,
    SerproConfiguracaoResponse,
};
use crate::presentation::serpro::handlers::map_obter_para_api;

/// GET /api/v1/admin/serpro/configuracao?escritorioId=...
///
/// O gestor de tenants (audiência ADMIN) não tem `escritorio_id` no token
/// (ADR-0010, D3); o escritório alvo vem na query. Cliente (portal) usa
/// `GET /api/v1/serpro/configuracao` — ver presentation/serpro.
pub async fn obter_configuracao_serpro(
    State(estado): State<AdminState>,
    Extension(contexto): Extension<RequestContext>,
    Query(query): Query<ObterConfiguracaoQuery>,
) -> Result<Response, ApiError> {
    garante_admin(&contexto)?;

    let resultado = estado
        .serpro
        .obter_configuracao
        .executar(ObterConfiguracaoSerproCommand {
            escritorio_id: query.escritorio_id,
        })
        .await
        .map_err(map_obter_para_api)?;

    Ok((
        StatusCode::OK,
        Json(SerproConfiguracaoResponse {
            escritorio_id: resultado.escritorio_id,
            modo_trial_serpro: resultado.modo_trial_serpro,
            ambiente_efetivo: resultado.ambiente_efetivo,
            versao: resultado.versao,
            atualizado_em: resultado.atualizado_em,
        }),
    )
        .into_response())
}

/// PATCH /api/v1/admin/serpro/trial
///
/// Concorrência otimista por `versao`; troca para produção é bloqueada
/// quando faltam credencial de produção do Integra Contador ou certificado
/// A1 em validade (ADR-0005 §3/§5). Auditoria gravada no sucesso.
pub async fn alterar_modo_trial_serpro(
    State(estado): State<AdminState>,
    Extension(contexto): Extension<RequestContext>,
    Json(payload): Json<AlterarModoTrialRequest>,
) -> Result<Response, ApiError> {
    garante_admin(&contexto)?;

    let comando = AlterarModoTrialSerproCommand {
        escritorio_id: payload.escritorio_id,
        modo_trial_serpro: payload.modo_trial_serpro,
        administrador_id: contexto.usuario_id,
        ip: contexto.ip,
        versao_esperada: payload.versao,
    };

    let resultado = estado
        .serpro
        .alterar_modo
        .executar(comando)
        .await
        .map_err(map_alterar_para_api)?;

    Ok((
        StatusCode::OK,
        Json(AlterarModoTrialResponse {
            modo_trial_serpro: resultado.modo_trial_serpro,
            ambiente_efetivo: resultado.ambiente_efetivo,
            versao_atual: resultado.versao_atual,
            atualizado_em: resultado.atualizado_em,
        }),
    )
        .into_response())
}

fn garante_admin(contexto: &RequestContext) -> Result<(), ApiError> {
    if contexto.papel != Papel::Admin {
        return Err(ApiError::proibido());
    }
    Ok(())
}

fn map_alterar_para_api(erro: crate::application::serpro::AlterarModoTrialError) -> ApiError {
    use crate::application::serpro::AlterarModoTrialError;
    match erro {
        AlterarModoTrialError::Ambiente(ErroAmbiente::ConflitoVersao { esperado, .. }) => {
            ApiError::conflito("versão do escritório divergiu; recarregue e tente novamente")
                .com_detalhe(format!("versão esperada: {esperado}"))
        }
        AlterarModoTrialError::Ambiente(ErroAmbiente::ProducaoBloqueada(mensagem)) => {
            ApiError::validacao(mensagem)
        }
        AlterarModoTrialError::Ambiente(ErroAmbiente::EscritorioNaoEncontrado(_)) => {
            ApiError::nao_encontrado("escritório")
        }
        AlterarModoTrialError::Ambiente(ErroAmbiente::RepositorioIndisponivel) => {
            ApiError::indisponivel()
        }
        AlterarModoTrialError::Repositorio(repositorio) => match repositorio {
            RepositorioError::Indisponivel => ApiError::indisponivel(),
            _ => ApiError::erro_interno(),
        },
        AlterarModoTrialError::Auditoria(_) => ApiError::erro_interno(),
    }
}
