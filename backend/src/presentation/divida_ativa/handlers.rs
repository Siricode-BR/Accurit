//! Handlers do consultivo de Dívida Ativa (Fase 4): duas leituras
//! autenticadas do portal (audiência client), ambiente resolvido
//! uma vez pelo middleware (SPEC §8) e mapeamento estável de erros
//! (código/status/mensagem, SPEC §4.4) com Retry-After no 429.

use axum::extract::{Extension, Path, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};

use crate::application::divida_ativa::use_cases::{
    ConsultaDevedorCommand, ConsultaInscricaoCommand,
};
use crate::domain::divida_ativa::erros::{ErroConsultaDivida, ErroSerpro};
use crate::domain::serpro::erros::ErroEndpoint;
use crate::presentation::auth::RequestContext;
use crate::presentation::divida_ativa::dto::{
    DevedorResponse, InscricaoDetalheDto, InscricaoDevedorDto, InscricaoResponse,
};
use crate::presentation::divida_ativa::router::EstadoDivida;
use crate::presentation::error::ApiError;
use crate::presentation::middleware::request_id::RequestId;
use crate::presentation::serpro::ambiente::AmbienteResolvido;

/// GET /divida-ativa/devedor/{cpfCnpj} — pendências e valores pela
/// chave de documento; o ambiente vem do snapshot do middleware
/// (uma vez por requisição, SPEC §8).
pub async fn consultar_devedor(
    State(estado): State<EstadoDivida>,
    Extension(contexto): Extension<RequestContext>,
    Extension(ambiente): Extension<AmbienteResolvido>,
    Extension(requisicao_id): Extension<RequestId>,
    Path(cpf_cnpj): Path<String>,
) -> Result<Response, ApiError> {
    let Some(escritorio_id) = contexto.escritorio_id else {
        return Err(ApiError::erro_interno());
    };

    let comando = ConsultaDevedorCommand {
        escritorio_id,
        usuario_id: contexto.usuario_id,
        ambiente: ambiente.ambiente,
        cpf_cnpj,
    };

    match estado.devedor.executar(comando).await {
        Ok(resultado) => {
            let corpo = DevedorResponse {
                ambiente: resultado.ambiente,
                estado: resultado.estado.as_str(),
                fonte_consulta: resultado.fonte.as_str(),
                chave_mascarada: resultado.chave_mascarada,
                horario_consulta: resultado.horario_consulta,
                tentativas: resultado.tentativas,
                inscricoes: resultado
                    .inscricoes
                    .iter()
                    .map(InscricaoDevedorDto::do_dominio)
                    .collect(),
            };
            Ok((StatusCode::OK, axum::Json(corpo)).into_response())
        }
        Err(erro) => Ok(resposta_do_erro(&erro, requisicao_id)),
    }
}

/// GET /divida-ativa/inscricao/{numeroInscricao} — detalhe completo
/// da inscrição (leitura; sem ações de negociação na API).
pub async fn consultar_inscricao(
    State(estado): State<EstadoDivida>,
    Extension(contexto): Extension<RequestContext>,
    Extension(ambiente): Extension<AmbienteResolvido>,
    Extension(requisicao_id): Extension<RequestId>,
    Path(numero_inscricao): Path<String>,
) -> Result<Response, ApiError> {
    let Some(escritorio_id) = contexto.escritorio_id else {
        return Err(ApiError::erro_interno());
    };

    let comando = ConsultaInscricaoCommand {
        escritorio_id,
        usuario_id: contexto.usuario_id,
        ambiente: ambiente.ambiente,
        numero_inscricao,
    };

    match estado.inscricao.executar(comando).await {
        Ok(resultado) => {
            let corpo = InscricaoResponse {
                ambiente: resultado.ambiente,
                estado: resultado.estado.as_str(),
                chave_mascarada: resultado.chave_mascarada,
                horario_consulta: resultado.horario_consulta,
                detalhe: InscricaoDetalheDto::do_dominio(&resultado.detalhe),
            };
            Ok((StatusCode::OK, axum::Json(corpo)).into_response())
        }
        Err(erro) => Ok(resposta_do_erro(&erro, requisicao_id)),
    }
}

/// Mapeamento `ErroConsultaDivida` → `ApiError`: mensagem falável e
/// neutra (nunca vaza configuração), código estável e status do
/// painel (SPEC §4.4).
pub fn mapear_erro_consulta(erro: &ErroConsultaDivida) -> ApiError {
    let mensagem: String = match erro {
        ErroConsultaDivida::ParametroInvalido(campo) => {
            format!("chave de consulta inválida ({campo})")
        }
        ErroConsultaDivida::Endpoint(ErroEndpoint::DividaAtivaProducaoBloqueada) => {
            "serviço Dívida Ativa indisponível no momento".to_owned()
        }
        ErroConsultaDivida::Endpoint(_) => "endpoint da Dívida Ativa não configurado".to_owned(),
        ErroConsultaDivida::Serpro(ErroSerpro::NaoEncontrado) => {
            "chave não encontrada na Dívida Ativa".to_owned()
        }
        ErroConsultaDivida::Serpro(ErroSerpro::LimiteExcedido { .. }) => {
            "limite de consultas do SERPRO atingido; retente mais tarde".to_owned()
        }
        ErroConsultaDivida::Serpro(_) => {
            "serviço da Dívida Ativa indisponível no momento".to_owned()
        }
        ErroConsultaDivida::RespostaInvalida(_) => {
            "resposta do serviço da Dívida Ativa fora do contrato".to_owned()
        }
        ErroConsultaDivida::LimiteLocalExcedido => {
            "limite de requisições por escritório excedido; retente em instantes".to_owned()
        }
        ErroConsultaDivida::PersistenciaIndisponivel => {
            "histórico de consultas indisponível; tente novamente".to_owned()
        }
    };

    let status =
        StatusCode::from_u16(erro.status_http()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    ApiError::nova(status, erro.codigo(), mensagem)
}

/// Resposta de erro com `Retry-After` (ecoa o 429 do SERPRO;
/// limite local sugere 1s).
pub fn resposta_do_erro(erro: &ErroConsultaDivida, requisicao_id: RequestId) -> Response {
    let mut resposta = mapear_erro_consulta(erro).resposta(requisicao_id.como_str());
    let segundos = erro
        .retry_after_segundos()
        .or_else(|| matches!(erro, ErroConsultaDivida::LimiteLocalExcedido).then_some(1));
    if let Some(segundos) = segundos
        && let Ok(valor) = HeaderValue::from_str(segundos.to_string().as_str())
    {
        resposta.headers_mut().insert(header::RETRY_AFTER, valor);
    }
    resposta
}
