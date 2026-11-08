use axum::Extension;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;

use crate::application::integra_contador::ComandoAutenticarProcurador;
use crate::application::integra_contador::ComandoIntegraContador;
use crate::application::integra_contador::ComandoListarProcuracao;
use crate::application::integra_contador::ComandoValidarProcuracao;
use crate::domain::integra_contador::erros::ErroIntegraContador;
use crate::presentation::auth::RequestContext;
use crate::presentation::error::ApiError;
use crate::presentation::integra_contador::dto::{
    ConsultaProcuracoesQuery, EstadoProcuracaoClienteResponse, LinhaCatalogoServicoResponse,
    RequisicaoAutenticarProcurador, RequisicaoConsultaProcuracoes, RequisicaoExecucao,
    RequisicaoHabilitarServico, RequisicaoValidarProcuracao, ResultadoAutenticarProcuradorResponse,
    ResultadoIntegraContadorResponse, ResultadoValidarProcuracaoResponse,
    StatusIntegraContadorResponse,
};
use crate::presentation::integra_contador::state::EstadoIntegraContador;
use crate::presentation::middleware::request_id::RequestId;
use crate::presentation::serpro::ambiente::AmbienteResolvido;

pub async fn status(
    Extension(ambiente): Extension<AmbienteResolvido>,
) -> Json<StatusIntegraContadorResponse> {
    Json(StatusIntegraContadorResponse::novo(ambiente))
}

pub async fn executar(
    State(estado): State<EstadoIntegraContador>,
    Extension(contexto): Extension<RequestContext>,
    Extension(ambiente): Extension<AmbienteResolvido>,
    request_id: RequestId,
    Json(requisicao): Json<RequisicaoExecucao>,
) -> Result<Json<ResultadoIntegraContadorResponse>, ApiError> {
    let escritorio_id = escritorio_do_contexto(&contexto)?;
    let comando = ComandoIntegraContador {
        escritorio_id,
        usuario_id: contexto.usuario_id,
        documento_contribuinte: requisicao.documento_contribuinte,
        verbo: requisicao.verbo,
        pedido_dados: requisicao.pedido_dados,
        confirmacao_explicita: requisicao.confirmacao_explicita,
        chave_idempotencia: requisicao.chave_idempotencia,
        ambiente: ambiente.ambiente,
        request_id: request_id.como_str().to_owned(),
    };

    let resultado = estado
        .executar
        .executar(comando)
        .await
        .map_err(mapear_erro)?;
    Ok(Json(resultado.into()))
}

/// Valida a procuração e-CAC de um cliente: assina o Termo de Autorização
/// com o certificado A1 do escritório, envia a AUTENTICAPROCURADOR
/// (ENVIOXMLASSINADO81) e guarda o token por cliente. Responde apenas com
/// status e expiração — o token nunca sai do backend (SPEC, REGRA SEG).
pub async fn autenticar_procurador(
    State(estado): State<EstadoIntegraContador>,
    Extension(contexto): Extension<RequestContext>,
    Extension(ambiente): Extension<AmbienteResolvido>,
    request_id: RequestId,
    Json(requisicao): Json<RequisicaoAutenticarProcurador>,
) -> Result<Json<ResultadoAutenticarProcuradorResponse>, ApiError> {
    let escritorio_id = escritorio_do_contexto(&contexto)?;
    let comando = ComandoAutenticarProcurador {
        escritorio_id,
        usuario_id: contexto.usuario_id,
        cliente_id: requisicao.cliente_id,
        ambiente: ambiente.ambiente,
        request_id: request_id.como_str().to_owned(),
    };

    let resultado = estado
        .autenticar_procurador
        .executar(comando)
        .await
        .map_err(mapear_erro)?;

    Ok(Json(ResultadoAutenticarProcuradorResponse {
        cliente_id: requisicao.cliente_id,
        status: "AUTORIZADA",
        expira_em: Some(resultado.expira_em),
    }))
}

pub async fn consultar_procuracoes(
    State(estado): State<EstadoIntegraContador>,
    Extension(contexto): Extension<RequestContext>,
    Extension(ambiente): Extension<AmbienteResolvido>,
    request_id: RequestId,
    Json(requisicao): Json<RequisicaoConsultaProcuracoes>,
) -> Result<Json<ResultadoIntegraContadorResponse>, ApiError> {
    let escritorio_id = escritorio_do_contexto(&contexto)?;
    let resultado = estado
        .executar
        .consultar_procuracoes(
            escritorio_id,
            contexto.usuario_id,
            requisicao.documento_contribuinte,
            ambiente.ambiente,
            request_id.como_str().to_owned(),
        )
        .await
        .map_err(mapear_erro)?;
    Ok(Json(resultado.into()))
}

// ===== FASE A — procurações persistentes e catálogo configurável =====

/// `POST /procuracoes/validar` — consulta o SERPRO (OBTERPROCURACAO41) se a
/// última verificação saiu do TTL. Persistência fica em `procuracoes`
/// (registro agregado, P7).
pub async fn validar_procuracao(
    State(estado): State<EstadoIntegraContador>,
    Extension(contexto): Extension<RequestContext>,
    Extension(ambiente): Extension<AmbienteResolvido>,
    request_id: RequestId,
    Json(requisicao): Json<RequisicaoValidarProcuracao>,
) -> Result<Json<ResultadoValidarProcuracaoResponse>, ApiError> {
    validar_ou_revalidar(estado, contexto, ambiente, request_id, requisicao, false).await
}

/// `POST /procuracoes/revalidar` — força a consulta ignorando o TTL.
pub async fn revalidar_procuracao(
    State(estado): State<EstadoIntegraContador>,
    Extension(contexto): Extension<RequestContext>,
    Extension(ambiente): Extension<AmbienteResolvido>,
    request_id: RequestId,
    Json(requisicao): Json<RequisicaoValidarProcuracao>,
) -> Result<Json<ResultadoValidarProcuracaoResponse>, ApiError> {
    validar_ou_revalidar(estado, contexto, ambiente, request_id, requisicao, true).await
}

#[allow(clippy::too_many_arguments)]
async fn validar_ou_revalidar(
    estado: EstadoIntegraContador,
    contexto: RequestContext,
    ambiente: AmbienteResolvido,
    request_id: RequestId,
    requisicao: RequisicaoValidarProcuracao,
    forcar: bool,
) -> Result<Json<ResultadoValidarProcuracaoResponse>, ApiError> {
    let escritorio_id = escritorio_do_contexto(&contexto)?;
    let resultado = estado
        .validar_procuracao
        .executar(ComandoValidarProcuracao {
            escritorio_id,
            usuario_id: contexto.usuario_id,
            cliente_id: requisicao.cliente_id,
            ambiente: ambiente.ambiente,
            request_id: request_id.como_str().to_owned(),
            forcar,
        })
        .await
        .map_err(mapear_erro)?;
    Ok(Json(resultado.into()))
}

/// `GET /procuracoes?clienteId=…` — estado consolidado da procuração do
/// cliente (registro agregado + estado por serviço, fail-closed — P7).
pub async fn listar_procuracao(
    State(estado): State<EstadoIntegraContador>,
    Extension(contexto): Extension<RequestContext>,
    Extension(ambiente): Extension<AmbienteResolvido>,
    axum::extract::Query(consulta): axum::extract::Query<ConsultaProcuracoesQuery>,
) -> Result<Json<EstadoProcuracaoClienteResponse>, ApiError> {
    let escritorio_id = escritorio_do_contexto(&contexto)?;
    let resultado = estado
        .listar_procuracao
        .executar(ComandoListarProcuracao {
            escritorio_id,
            cliente_id: consulta.cliente_id,
            ambiente: ambiente.ambiente,
        })
        .await
        .map_err(mapear_erro)?;
    Ok(Json(resultado.into()))
}

/// `GET /admin/integra-contador/catalogo` — linhas do catálogo configurável.
pub async fn catalogo(
    State(estado): State<EstadoIntegraContador>,
) -> Result<Json<Vec<LinhaCatalogoServicoResponse>>, ApiError> {
    let linhas = estado.catalogo.listar().await.map_err(mapear_erro)?;
    Ok(Json(
        linhas
            .into_iter()
            .map(LinhaCatalogoServicoResponse::from)
            .collect(),
    ))
}

/// `PATCH /admin/integra-contador/catalogo/{idSistema}/{idServico}` —
/// liga/desliga uma linha. Fail-closed: sem versão confirmada não habilita;
/// o contrato precisa existir no catálogo de referência do domínio.
pub async fn definir_habilitado_catalogo(
    State(estado): State<EstadoIntegraContador>,
    axum::extract::Path((id_sistema, id_servico)): axum::extract::Path<(String, String)>,
    Json(requisicao): Json<RequisicaoHabilitarServico>,
) -> Result<Json<LinhaCatalogoServicoResponse>, ApiError> {
    // Fail-closed contra o catálogo de referência do domínio: só é possível
    // ligar/desligar linha cujo contrato exista no catálogo estático.
    let existe_no_dominio = crate::domain::integra_contador::servicos::catalogo_de_referencia()
        .iter()
        .any(|servico| {
            servico.id_sistema.eq_ignore_ascii_case(&id_sistema)
                && servico.id_servico.eq_ignore_ascii_case(&id_servico)
        });
    if !existe_no_dominio {
        return Err(ApiError::nova(
            StatusCode::NOT_FOUND,
            "NAO_ENCONTRADO",
            "Serviço não existe no catálogo de referência do domínio.",
        ));
    }
    let linha = estado
        .catalogo
        .definir_habilitado(&id_sistema, &id_servico, requisicao.habilitado)
        .await
        .map_err(mapear_erro)?;
    Ok(Json(linha.into()))
}

fn escritorio_do_contexto(contexto: &RequestContext) -> Result<uuid::Uuid, ApiError> {
    contexto.escritorio_id.ok_or_else(|| {
        ApiError::nova(
            StatusCode::INTERNAL_SERVER_ERROR,
            "CONTEXTO_TOKEN_INVALIDO",
            "Token do portal sem escritório associado",
        )
    })
}

fn mapear_erro(erro: ErroIntegraContador) -> ApiError {
    match erro {
        ErroIntegraContador::ServicoNaoHabilitado => ApiError::nova(
            StatusCode::FORBIDDEN,
            "SERVICO_NAO_HABILITADO",
            "Este serviço não está habilitado no catálogo vigente.",
        ),
        ErroIntegraContador::VersaoNaoConfirmada => ApiError::nova(
            StatusCode::CONFLICT,
            "VERSAO_NAO_CONFIRMADA",
            "A versão deste serviço ainda não foi confirmada.",
        ),
        ErroIntegraContador::ClienteNaoEncontrado => ApiError::nova(
            StatusCode::NOT_FOUND,
            "NAO_ENCONTRADO",
            "Cliente não encontrado neste escritório.",
        ),
        ErroIntegraContador::ConfirmacaoNecessaria => {
            ApiError::validacao("Confirmação explícita é necessária para esta operação.")
        }
        ErroIntegraContador::IdempotenciaNecessaria => {
            ApiError::validacao("Chave de idempotência válida é necessária.")
        }
        ErroIntegraContador::ConflitoIdempotencia => {
            ApiError::conflito("A chave de idempotência já foi usada.")
        }
        ErroIntegraContador::ProcuracaoNegada | ErroIntegraContador::AcessoNegado => {
            ApiError::nova(
                StatusCode::FORBIDDEN,
                "ACESSO_NEGADO",
                "Autorização não confirmada.",
            )
        }
        ErroIntegraContador::NaoAutorizado => ApiError::nova(
            StatusCode::UNAUTHORIZED,
            "SERPRO_NAO_AUTORIZADO",
            "Autenticação SERPRO recusada.",
        ),
        ErroIntegraContador::ProducaoBloqueada => ApiError::nova(
            StatusCode::SERVICE_UNAVAILABLE,
            "PRODUCAO_BLOQUEADA",
            "Produção depende da confirmação contratual e aprovação D5.",
        ),
        ErroIntegraContador::LimiteExcedido => ApiError::nova(
            StatusCode::TOO_MANY_REQUESTS,
            "LIMITE_SERPRO",
            "Limite SERPRO excedido.",
        ),
        ErroIntegraContador::Indeterminado => ApiError::nova(
            StatusCode::GATEWAY_TIMEOUT,
            "ESTADO_INDETERMINADO",
            "A SERPRO não confirmou o resultado. Consulte o estado antes de qualquer repetição.",
        ),
        ErroIntegraContador::PayloadInvalido => {
            ApiError::validacao("A requisição ou resposta não corresponde ao contrato.")
        }
        // A SERPRO retornou HTTP 200 com corpo malformado: não é culpa do
        // cliente, é falha upstream. Nunca exponha detalhes do corpo.
        ErroIntegraContador::RespostaInvalida => ApiError::nova(
            StatusCode::BAD_GATEWAY,
            "SERPRO_RESPOSTA_INVALIDA",
            "A SERPRO retornou uma resposta inválida para o Termo de Autorização.",
        ),
        // Falha de resolução de cliente/tenant: fail-closed, mas não é
        // indisponibilidade de infra — é configuração/permissão do escritório.
        ErroIntegraContador::ConfiguracaoIncompleta => ApiError::nova(
            StatusCode::FORBIDDEN,
            "CONFIGURACAO_INCOMPLETA",
            "Configuração ou certificado do escritório incompleto para o fluxo de procurador.",
        ),
        ErroIntegraContador::Indisponivel => ApiError::indisponivel(),
        ErroIntegraContador::Persistencia => ApiError::erro_interno(),
    }
}
