//! Fase B — Handlers de trabalhos agendados e consultas em lote.

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use serde_json::Value;
use uuid::Uuid;

use crate::domain::trabalhos::enums::{StatusExecucao, StatusLote, TipoTrabalho};
use crate::domain::trabalhos::erros::ErroTrabalho;
use crate::presentation::auth::RequestContext;
use crate::presentation::error::ApiError;
use crate::presentation::trabalhos::dto::{
    RequisicaoAlternarTrabalho, RequisicaoCriarConsultaLote, RequisicaoCriarTrabalho,
    RespostaConsultaLote, RespostaExecucaoTrabalho, RespostaItemConsultaLote,
    RespostaTrabalhoAgendado,
};
use crate::presentation::trabalhos::state::EstadoTrabalhos;

/// POST /trabalhos — cria um trabalho agendado.
pub async fn criar_trabalho(
    State(estado): State<EstadoTrabalhos>,
    Extension(contexto): Extension<RequestContext>,
    Json(requisicao): Json<RequisicaoCriarTrabalho>,
) -> Result<Json<RespostaTrabalhoAgendado>, ApiError> {
    let escritorio_id = contexto.escritorio_id.ok_or_else(|| {
        ApiError::nova(
            StatusCode::INTERNAL_SERVER_ERROR,
            "CONTEXTO_TOKEN_INVALIDO",
            "Token sem escritório",
        )
    })?;

    let tipo = match requisicao.tipo.as_str() {
        "consulta_divida_ativa" => TipoTrabalho::ConsultaDividaAtiva,
        "validar_procuracao" => TipoTrabalho::ValidarProcuracao,
        "consulta_integra_contador" => TipoTrabalho::ConsultaIntegraContador,
        _ => return Err(ApiError::validacao("tipo de trabalho inválido")),
    };

    let trabalho = estado
        .criar_trabalho
        .executar(crate::domain::trabalhos::entidades::NovoTrabalhoAgendado {
            escritorio_id,
            nome: requisicao.nome,
            tipo,
            produto: requisicao.produto,
            ambiente: requisicao.ambiente,
            cron_expression: requisicao.cron_expression,
        })
        .await
        .map_err(mapear_erro_trabalho)?;

    Ok(Json(RespostaTrabalhoAgendado::from(trabalho)))
}

/// GET /trabalhos — lista trabalhos agendados do escritório.
pub async fn listar_trabalhos(
    State(estado): State<EstadoTrabalhos>,
    Extension(contexto): Extension<RequestContext>,
) -> Result<Json<Vec<RespostaTrabalhoAgendado>>, ApiError> {
    let escritorio_id = contexto.escritorio_id.ok_or_else(|| {
        ApiError::nova(
            StatusCode::INTERNAL_SERVER_ERROR,
            "CONTEXTO_TOKEN_INVALIDO",
            "Token sem escritório",
        )
    })?;

    let trabalhos = estado
        .listar_trabalhos
        .executar(escritorio_id)
        .await
        .map_err(mapear_erro_trabalho)?;

    Ok(Json(
        trabalhos
            .into_iter()
            .map(RespostaTrabalhoAgendado::from)
            .collect(),
    ))
}

/// GET /trabalhos/{id} — busca um trabalho agendado por ID.
pub async fn buscar_trabalho(
    State(estado): State<EstadoTrabalhos>,
    Extension(contexto): Extension<RequestContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<RespostaTrabalhoAgendado>, ApiError> {
    let escritorio_id = contexto.escritorio_id.ok_or_else(|| {
        ApiError::nova(
            StatusCode::INTERNAL_SERVER_ERROR,
            "CONTEXTO_TOKEN_INVALIDO",
            "Token sem escritório",
        )
    })?;

    let trabalho = estado
        .buscar_trabalho
        .executar(id)
        .await
        .map_err(mapear_erro_trabalho)?
        .ok_or_else(|| {
            ApiError::nova(
                StatusCode::NOT_FOUND,
                "TRABALHO_NAO_ENCONTRADO",
                "Trabalho não encontrado",
            )
        })?;

    if trabalho.escritorio_id != escritorio_id {
        return Err(ApiError::nova(
            StatusCode::NOT_FOUND,
            "TRABALHO_NAO_ENCONTRADO",
            "Trabalho não encontrado",
        ));
    }

    Ok(Json(RespostaTrabalhoAgendado::from(trabalho)))
}

/// PATCH /trabalhos/{id}/alternar — habilita ou desabilita um trabalho.
pub async fn alternar_trabalho(
    State(estado): State<EstadoTrabalhos>,
    Extension(contexto): Extension<RequestContext>,
    Path(id): Path<Uuid>,
    Json(requisicao): Json<RequisicaoAlternarTrabalho>,
) -> Result<StatusCode, ApiError> {
    let escritorio_id = contexto.escritorio_id.ok_or_else(|| {
        ApiError::nova(
            StatusCode::INTERNAL_SERVER_ERROR,
            "CONTEXTO_TOKEN_INVALIDO",
            "Token sem escritório",
        )
    })?;

    // Verifica se o trabalho pertence ao escritório
    let trabalho = estado
        .buscar_trabalho
        .executar(id)
        .await
        .map_err(mapear_erro_trabalho)?
        .ok_or_else(|| {
            ApiError::nova(
                StatusCode::NOT_FOUND,
                "TRABALHO_NAO_ENCONTRADO",
                "Trabalho não encontrado",
            )
        })?;

    if trabalho.escritorio_id != escritorio_id {
        return Err(ApiError::nova(
            StatusCode::NOT_FOUND,
            "TRABALHO_NAO_ENCONTRADO",
            "Trabalho não encontrado",
        ));
    }

    if requisicao.habilitado {
        estado
            .habilitar_desabilitar
            .habilitar(id)
            .await
            .map_err(mapear_erro_trabalho)?;
    } else {
        estado
            .habilitar_desabilitar
            .desabilitar(id)
            .await
            .map_err(mapear_erro_trabalho)?;
    }

    Ok(StatusCode::NO_CONTENT)
}

/// GET /execucoes/trabalho/{trabalho_id} — lista execuções de um trabalho.
pub async fn listar_execucoes_por_trabalho(
    State(estado): State<EstadoTrabalhos>,
    Extension(contexto): Extension<RequestContext>,
    Path(trabalho_id): Path<Uuid>,
) -> Result<Json<Vec<RespostaExecucaoTrabalho>>, ApiError> {
    let escritorio_id = contexto.escritorio_id.ok_or_else(|| {
        ApiError::nova(
            StatusCode::INTERNAL_SERVER_ERROR,
            "CONTEXTO_TOKEN_INVALIDO",
            "Token sem escritório",
        )
    })?;

    // Verifica se o trabalho pertence ao escritório
    let trabalho = estado
        .buscar_trabalho
        .executar(trabalho_id)
        .await
        .map_err(mapear_erro_trabalho)?
        .ok_or_else(|| {
            ApiError::nova(
                StatusCode::NOT_FOUND,
                "TRABALHO_NAO_ENCONTRADO",
                "Trabalho não encontrado",
            )
        })?;

    if trabalho.escritorio_id != escritorio_id {
        return Err(ApiError::nova(
            StatusCode::NOT_FOUND,
            "TRABALHO_NAO_ENCONTRADO",
            "Trabalho não encontrado",
        ));
    }

    let execucoes = estado
        .listar_execucoes
        .executar_por_trabalho(trabalho_id)
        .await
        .map_err(mapear_erro_trabalho)?;

    Ok(Json(
        execucoes
            .into_iter()
            .map(RespostaExecucaoTrabalho::from)
            .collect(),
    ))
}

/// GET /execucoes — lista execuções do escritório.
pub async fn listar_execucoes_escritorio(
    State(estado): State<EstadoTrabalhos>,
    Extension(contexto): Extension<RequestContext>,
) -> Result<Json<Vec<RespostaExecucaoTrabalho>>, ApiError> {
    let escritorio_id = contexto.escritorio_id.ok_or_else(|| {
        ApiError::nova(
            StatusCode::INTERNAL_SERVER_ERROR,
            "CONTEXTO_TOKEN_INVALIDO",
            "Token sem escritório",
        )
    })?;

    let execucoes = estado
        .listar_execucoes
        .executar_por_escritorio(escritorio_id)
        .await
        .map_err(mapear_erro_trabalho)?;

    Ok(Json(
        execucoes
            .into_iter()
            .map(RespostaExecucaoTrabalho::from)
            .collect(),
    ))
}

/// POST /consultas-lote — cria uma consulta em lote.
pub async fn criar_consulta_lote(
    State(estado): State<EstadoTrabalhos>,
    Extension(contexto): Extension<RequestContext>,
    Json(requisicao): Json<RequisicaoCriarConsultaLote>,
) -> Result<Json<RespostaConsultaLote>, ApiError> {
    let escritorio_id = contexto.escritorio_id.ok_or_else(|| {
        ApiError::nova(
            StatusCode::INTERNAL_SERVER_ERROR,
            "CONTEXTO_TOKEN_INVALIDO",
            "Token sem escritório",
        )
    })?;

    let documentos: Vec<crate::domain::trabalhos::entidades::NovoItemConsultaLote> = requisicao
        .documentos
        .iter()
        .map(
            |d| crate::domain::trabalhos::entidades::NovoItemConsultaLote {
                documento: d.documento.clone(),
                tipo: d.tipo.clone(),
            },
        )
        .collect();

    let lote = estado
        .criar_lote
        .executar(
            crate::domain::trabalhos::entidades::NovoConsultaLote {
                escritorio_id,
                nome: requisicao.nome,
                produto: requisicao.produto,
                servico: requisicao.servico,
                verbo: requisicao.verbo,
            },
            &documentos,
        )
        .await
        .map_err(mapear_erro_trabalho)?;

    Ok(Json(RespostaConsultaLote::from(lote)))
}

/// GET /consultas-lote — lista consultas em lote do escritório.
pub async fn listar_consultas_lote(
    State(estado): State<EstadoTrabalhos>,
    Extension(contexto): Extension<RequestContext>,
) -> Result<Json<Vec<RespostaConsultaLote>>, ApiError> {
    let escritorio_id = contexto.escritorio_id.ok_or_else(|| {
        ApiError::nova(
            StatusCode::INTERNAL_SERVER_ERROR,
            "CONTEXTO_TOKEN_INVALIDO",
            "Token sem escritório",
        )
    })?;

    let lotes = estado
        .listar_lotes
        .executar(escritorio_id)
        .await
        .map_err(mapear_erro_trabalho)?;

    Ok(Json(
        lotes.into_iter().map(RespostaConsultaLote::from).collect(),
    ))
}

/// GET /consultas-lote/{id} — busca uma consulta em lote por ID.
pub async fn buscar_consulta_lote(
    State(estado): State<EstadoTrabalhos>,
    Extension(contexto): Extension<RequestContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<RespostaConsultaLote>, ApiError> {
    let escritorio_id = contexto.escritorio_id.ok_or_else(|| {
        ApiError::nova(
            StatusCode::INTERNAL_SERVER_ERROR,
            "CONTEXTO_TOKEN_INVALIDO",
            "Token sem escritório",
        )
    })?;

    let lote = estado
        .buscar_lote
        .executar(id)
        .await
        .map_err(mapear_erro_trabalho)?
        .ok_or_else(|| {
            ApiError::nova(
                StatusCode::NOT_FOUND,
                "CONSULTA_LOTE_NAO_ENCONTRADA",
                "Lote não encontrado",
            )
        })?;

    if lote.escritorio_id != escritorio_id {
        return Err(ApiError::nova(
            StatusCode::NOT_FOUND,
            "CONSULTA_LOTE_NAO_ENCONTRADA",
            "Lote não encontrado",
        ));
    }

    Ok(Json(RespostaConsultaLote::from(lote)))
}

/// GET /consultas-lote/{id}/itens — lista itens de uma consulta em lote.
pub async fn listar_itens_consulta_lote(
    State(estado): State<EstadoTrabalhos>,
    Extension(contexto): Extension<RequestContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<RespostaItemConsultaLote>>, ApiError> {
    let escritorio_id = contexto.escritorio_id.ok_or_else(|| {
        ApiError::nova(
            StatusCode::INTERNAL_SERVER_ERROR,
            "CONTEXTO_TOKEN_INVALIDO",
            "Token sem escritório",
        )
    })?;

    // Verifica se o lote pertence ao escritório
    let lote = estado
        .buscar_lote
        .executar(id)
        .await
        .map_err(mapear_erro_trabalho)?
        .ok_or_else(|| {
            ApiError::nova(
                StatusCode::NOT_FOUND,
                "CONSULTA_LOTE_NAO_ENCONTRADA",
                "Lote não encontrado",
            )
        })?;

    if lote.escritorio_id != escritorio_id {
        return Err(ApiError::nova(
            StatusCode::NOT_FOUND,
            "CONSULTA_LOTE_NAO_ENCONTRADA",
            "Lote não encontrado",
        ));
    }

    let itens = estado
        .listar_itens_lote
        .executar(id)
        .await
        .map_err(mapear_erro_trabalho)?;

    Ok(Json(
        itens
            .into_iter()
            .map(RespostaItemConsultaLote::from)
            .collect(),
    ))
}

/// GET /trabalhos/tipos — lista tipos de trabalho disponíveis.
pub async fn listar_tipos_trabalho() -> Json<Value> {
    Json(serde_json::json!([
        { "valor": TipoTrabalho::ConsultaDividaAtiva.as_str(), "label": "Consulta Dívida Ativa" },
        { "valor": TipoTrabalho::ValidarProcuracao.as_str(), "label": "Validar Procuração" },
        { "valor": TipoTrabalho::ConsultaIntegraContador.as_str(), "label": "Consulta Integra Contador" },
    ]))
}

/// GET /trabalhos/status-execucao — lista status de execução disponíveis.
pub async fn listar_status_execucao() -> Json<Value> {
    Json(serde_json::json!([
        { "valor": StatusExecucao::Pendente.as_str(), "label": "Pendente" },
        { "valor": StatusExecucao::Executando.as_str(), "label": "Executando" },
        { "valor": StatusExecucao::Concluido.as_str(), "label": "Concluído" },
        { "valor": StatusExecucao::Falhou.as_str(), "label": "Falhou" },
    ]))
}

/// GET /trabalhos/status-lote — lista status de lote disponíveis.
pub async fn listar_status_lote() -> Json<Value> {
    Json(serde_json::json!([
        { "valor": StatusLote::Pendente.as_str(), "label": "Pendente" },
        { "valor": StatusLote::Executando.as_str(), "label": "Executando" },
        { "valor": StatusLote::Concluido.as_str(), "label": "Concluído" },
        { "valor": StatusLote::Falhou.as_str(), "label": "Falhou" },
        { "valor": StatusLote::Parcial.as_str(), "label": "Parcial" },
    ]))
}

fn mapear_erro_trabalho(erro: ErroTrabalho) -> ApiError {
    match erro {
        ErroTrabalho::TrabalhoNaoEncontrado => ApiError::nova(
            StatusCode::NOT_FOUND,
            "TRABALHO_NAO_ENCONTRADO",
            "Trabalho não encontrado",
        ),
        ErroTrabalho::ExecucaoNaoEncontrada => ApiError::nova(
            StatusCode::NOT_FOUND,
            "EXECUCAO_NAO_ENCONTRADA",
            "Execução não encontrada",
        ),
        ErroTrabalho::ConsultaLoteNaoEncontrada => ApiError::nova(
            StatusCode::NOT_FOUND,
            "CONSULTA_LOTE_NAO_ENCONTRADA",
            "Consulta em lote não encontrada",
        ),
        ErroTrabalho::ItemLoteNaoEncontrado => ApiError::nova(
            StatusCode::NOT_FOUND,
            "ITEM_LOTE_NAO_ENCONTRADO",
            "Item de lote não encontrado",
        ),
        ErroTrabalho::JaDesabilitado => ApiError::nova(
            StatusCode::BAD_REQUEST,
            "TRABALHO_JA_DESABILITADO",
            "Trabalho já está desabilitado",
        ),
        ErroTrabalho::JaHabilitado => ApiError::nova(
            StatusCode::BAD_REQUEST,
            "TRABALHO_JA_HABILITADO",
            "Trabalho já está habilitado",
        ),
        ErroTrabalho::ExecucaoEmAndamento => ApiError::nova(
            StatusCode::BAD_REQUEST,
            "EXECUCAO_EM_ANDAMENTO",
            "Não é possível cancelar execução em andamento",
        ),
        ErroTrabalho::SemPermissao => ApiError::nova(
            StatusCode::FORBIDDEN,
            "SEM_PERMISSAO",
            "Escritório não possui permissão para este recurso",
        ),
        ErroTrabalho::Persistencia(msg) => {
            ApiError::nova(StatusCode::INTERNAL_SERVER_ERROR, "PERSISTENCIA", &msg)
        }
        ErroTrabalho::Interno(msg) => {
            ApiError::nova(StatusCode::INTERNAL_SERVER_ERROR, "ERRO_INTERNO", &msg)
        }
    }
}

use axum::Extension;
