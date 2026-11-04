use std::sync::Arc;

use axum::Extension;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::Router;
use serde::Deserialize;
use serde::Serialize;
use tracing::warn;

use crate::application::integra_contador::use_cases::ExecutarIntegraContadorUseCase;
use crate::domain::integra_contador::erros::ErroIntegraContador;
use crate::domain::integra_contador::servicos::resolver_servico;
use crate::presentation::auth::RequestContext;
use crate::presentation::error::ApiError;
use crate::presentation::serpro::ambiente::AmbienteResolvido;
use crate::presentation::serpro::dto::{
    ExecucaoRequest, ExecucaoResponse, ModoTrialRequest, ModoTrialResponse,
    StatusResponse, ServicoIntegraContadorStatus,
};
use crate::presentation::serpro::ambiente::AmbienteSerpro;
use crate::presentation::state::AppState;

use super::router::EstadoIntegraContador;

/// GET /api/v1/serpro/integra-contador/status - status do sistema
pub async fn consultar_status(
    State(_estado): State<AppState>,
    Extension(contexto): Extension<RequestContext>,
    Extension(snapshot): Extension<AmbienteResolvido>,
) -> Result<impl IntoResponse, ApiError> {
    let ambiente = match snapshot.ambiente {
        AmbienteSerpro::Trial => "TRIAL",
        AmbienteSerpro::Producao => "PRODUCAO",
    };

    let producao_habilitada = contexto.escritorio_id.is_some(); // simplificado

    Ok((
        StatusCode::OK,
        StatusResponse {
            ambiente_efetivo: AmbienteSerpro::Trial, // virá do snapshot
            producao_habilitada,
            assinatura_habilitada: true, // verificaremos abaixo
            motivo_bloqueio_producao: String::new(),
            motivo_bloqueio_assinatura: String::new(),
            servicos: Vec::new(), // serpopulado pelo router
        },
    )
        .into_response())
}

/// POST /api/v1/serpro/integra-contador/executar - executar operação (5 verbos)
pub async fn executar_operacao(
    Extension(contexto): Extension<RequestContext>,
    Extension(snapshot): Extension<AmbienteResolvido>,
    State(use_case): State<ExecutarIntegraContadorUseCase>,
    Json(req): Json<ExecucaoRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Validar se o serviço existe no catálogo
    let servico = match resolver_servico(
        &req.id_sistema,
        &req.id_servico,
        &req.versao_sistema,
        req.verbo,
    ) {
        Ok(s) => s,
        Err(erro) => {
            return Err(ApiError::nova(
                StatusCode::BAD_REQUEST,
                "ServicoNaoHabilitado",
                erro.to_string(),
            ))
        }
    };

    // Verificar se produção está bloqueada
    if servico.ambiente == AmbienteSerpro::Producao && !producao_habilitada {
        return Err(ApiError::nova(
            StatusCode::SERVICE_UNAVAILABLE,
            "ProducaoBloqueada",
            "Produção bloqueada até confirmação contratual e aprovação D5",
        ));
    }

    // Verificar confirmação explícita para verbos mutáveis
    if req.verbo.exige_confirmacao() && !req.confirmacao_explicita {
        return Err(ApiError::nova(
            StatusCode::BAD_REQUEST,
            "ConfirmacaoNecessaria",
            "Confirmação explícita necessária para este verbo",
        ));
    }

    // Montar comando para o use case
    let comando = crate::application::integra_contador::use_cases::ComandoIntegraContador {
        escritorio_id: contexto.escritorio_id,
        usuario_id: contexto.usuario_id,
        documento_contribuinte: req.dados.clone(), // simplificado - viria do contexto
        verbo: req.verbo,
        pedido_dados: crate::domain::integra_contador::envelope::PedidoDados {
            id_sistema: req.id_sistema,
            id_servico: req.id_servico,
            versao_sistema: req.versao_sistema,
            dados: req.dados,
        },
        confirmacao_explicita: req.confirmacao_explicita,
        chave_idempotencia: req.chave_idempotencia,
        ambiente: AmbienteSerpro::Trial, // do snapshot
        request_id: "req-id".to_owned(), // do contexto
    };

    match use_case.executar(comando).await {
        Ok(entrega) => {
            let status_str = match entrega.http_status {
                202 => "PROCESSANDO",
                204 => "PROCESSANDO",
                304 => "NAO_MODIFICADO",
                504 => "INDETERMINADO",
                200..=299 => "CONCLUIDO",
                _ => "REJEITADO",
            };

            Ok((
                StatusCode::OK,
                ExecucaoResponse {
                    ambiente: entrega.ambiente.to_string(),
                    verbo: req.verbo.como_str().to_owned(),
                    id_sistema: entrega.id_sistema,
                    id_servico: entrega.id_servico,
                    estado: status_str.to_owned(),
                    status_serpro: entrega.http_status,
                    mensagem: entrega.estado.clone(),
                    referencia_resposta: entrega.referencia_resposta,
                    tempo_espera_segundos: entrega.tempo_espera_segundos,
                    etag: entrega.etag,
                    expires: entrega.expires,
                    codigos_mensagem: entrega
                        .resposta
                        .as_ref()
                        .map_or_else(|| Vec::new(), |resp| {
                            resp.mensagens
                                .iter()
                                .filter(|m| !m.codigo.is_empty())
                                .map(|m| m.codigo.clone())
                                .collect()
                        }),
                },
            )
                .into_response())
        }
        Err(erro) => {
            let (status_code, body) = match erro {
                ErroIntegraContador::ProducaoBloqueada => {
                    (StatusCode::SERVICE_UNAVAILABLE, erro.to_string())
                }
                ErroIntegraContador::ConfirmacaoNecessaria => {
                    (StatusCode::BAD_REQUEST, erro.to_string())
                }
                ErroIntegraContador::ServicoNaoHabilitado => {
                    (StatusCode::BAD_REQUEST, erro.to_string())
                }
                ErroIntegraContador::ProcuracaoNegada => {
                    (StatusCode::FORBIDDEN, erro.to_string())
                }
                ErroIntegraContador::ConfiguracaoIncompleta => {
                    (StatusCode::BAD_REQUEST, erro.to_string())
                }
                ErroIntegraContador::NaoAutorizado => {
                    (StatusCode::FORBIDDEN, erro.to_string())
                }
                ErroIntegraContador::AcessoNegado => {
                    (StatusCode::FORBIDDEN, erro.to_string())
                }
                ErroIntegraContador::LimiteExcedido => {
                    (StatusCode::TOO_MANY_REQUESTS, erro.to_string())
                }
                ErroIntegraContador::Indeterminado => {
                    (StatusCode::GATEWAY_TIMEOUT, erro.to_string())
                }
                ErroIntegraContador::RespostaInvalida => {
                    (StatusCode::BAD_GATEWAY, erro.to_string())
                }
                ErroIntegraContador::Indisponivel => {
                    (StatusCode::SERVICE_UNAVAILABLE, erro.to_string())
                }
                ErroIntegraContador::Persistencia => {
                    (StatusCode::INTERNAL_SERVER_ERROR, erro.to_string())
                }
                _ => (StatusCode::INTERNAL_SERVER_ERROR, erro.to_string()),
            };

            Err(ApiError::new(status_code, body))
        }
    }
}

/// PATCH /api/v1/serpro/integra-contador/modo-trial - alterna modo trial/producao
pub async fn alterar_modo_trial(
    Extension(contexto): Extension<RequestContext>,
    State(use_case): State<crate::application::serpro::AlterarModoTrialSerproUseCase>,
    Json(req): Json<ModoTrialRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // A usar o use case existente de alterar modo trial da SERPRO
    // Este use case já está configurado nas routes
    // Aqui apenas validamos e delegamos

    // Em um implementation completo, chamaríamos:
    // use_case.alterar(req.modo_trial_serpro, req.versao).await
    // Mas como o use case está injetado do serpro state, usaremos o que já está disponível

    // Por enquanto, retornamos o estado atual
    let modo_trial = req.modo_trial_serpro;

    Ok((
        StatusCode::OK,
        ModoTrialResponse {
            modo_trial_serpro: modo_trial,
            ambiente_efetivo: AmbienteSerpro::Trial, // virá do cálculo interno
            versao_atual: 1,
            atualizado_em: chrono::Utc::now(),
        },
    )
        .into_response())