//! Fase C — Handlers de notificações.

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use uuid::Uuid;

use crate::domain::notificacoes::enums::{CanalNotificacao, TipoNotificacao};
use crate::domain::notificacoes::erros::ErroNotificacao;
use crate::presentation::auth::RequestContext;
use crate::presentation::error::ApiError;
use crate::presentation::notificacoes::dto::{
    RequisicaoCriarNotificacao, RequisicaoPreferenciaNotificacao, RespostaContagemNaoLidas,
    RespostaNotificacao, RespostaPreferenciaNotificacao,
};
use crate::presentation::notificacoes::state::EstadoNotificacoes;

/// POST /notificacoes — criar uma notificação.
pub async fn criar_notificacao(
    State(estado): State<EstadoNotificacoes>,
    Extension(contexto): Extension<RequestContext>,
    Json(requisicao): Json<RequisicaoCriarNotificacao>,
) -> Result<Json<RespostaNotificacao>, ApiError> {
    let canal = match requisicao.canal.as_str() {
        "in_app" => CanalNotificacao::InApp,
        "email" => CanalNotificacao::Email,
        "whatsapp" => CanalNotificacao::WhatsApp,
        _ => return Err(ApiError::validacao("canal inválido")),
    };
    let tipo = match requisicao.tipo.as_str() {
        "pendencia_divida_ativa" => TipoNotificacao::PendenciaDividaAtiva,
        "mensagem_caixa_postal" => TipoNotificacao::MensagemCaixaPostal,
        "procuracao_vencendo" => TipoNotificacao::ProcuracaoVencendo,
        "certificado_a1_vencendo" => TipoNotificacao::CertificadoA1Vencendo,
        "das_vencendo" => TipoNotificacao::DasVencendo,
        "falha_rotina" => TipoNotificacao::FalhaRotina,
        _ => return Err(ApiError::validacao("tipo inválido")),
    };

    let notificacao = estado
        .criar_notificacao
        .executar(crate::domain::notificacoes::entidades::NovoNotificacao {
            usuario_id: requisicao.usuario_id,
            escritorio_id: contexto.escritorio_id.ok_or_else(|| {
                ApiError::nova(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "CONTEXTO_TOKEN_INVALIDO",
                    "Token sem escritório",
                )
            })?,
            canal,
            tipo,
            titulo: requisicao.titulo,
            corpo: requisicao.corpo,
            referencia_tipo: requisicao.referencia_tipo,
            referencia_id: requisicao.referencia_id,
        })
        .await
        .map_err(mapear_erro_notificacao)?;

    Ok(Json(RespostaNotificacao::from(notificacao)))
}

/// GET /notificacoes — listar notificações do usuário.
pub async fn listar_notificacoes(
    State(estado): State<EstadoNotificacoes>,
    Extension(contexto): Extension<RequestContext>,
) -> Result<Json<Vec<RespostaNotificacao>>, ApiError> {
    let usuario_id = contexto.usuario_id;
    let nao_lidas_apenas = true;
    let limite = 50;

    let notificacoes = estado
        .listar_notificacoes
        .executar(usuario_id, nao_lidas_apenas, limite)
        .await
        .map_err(mapear_erro_notificacao)?;

    Ok(Json(
        notificacoes
            .into_iter()
            .map(RespostaNotificacao::from)
            .collect(),
    ))
}

/// GET /notificacoes/contagem-nao-lidas — contar não lidas.
pub async fn contar_nao_lidas(
    State(estado): State<EstadoNotificacoes>,
    Extension(contexto): Extension<RequestContext>,
) -> Result<Json<RespostaContagemNaoLidas>, ApiError> {
    let usuario_id = contexto.usuario_id;
    let total = estado
        .contar_nao_lidas
        .executar(usuario_id)
        .await
        .map_err(mapear_erro_notificacao)?;

    Ok(Json(RespostaContagemNaoLidas {
        nao_lidas: total as i64,
    }))
}

/// PATCH /notificacoes/{id}/ler — marcar como lida.
pub async fn marcar_como_lida(
    State(estado): State<EstadoNotificacoes>,
    Extension(contexto): Extension<RequestContext>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let usuario_id = contexto.usuario_id;
    estado
        .marcar_como_lida
        .executar(id, usuario_id)
        .await
        .map_err(mapear_erro_notificacao)?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /notificacoes/marcar-todas-lidas — marcar todas como lidas.
pub async fn marcar_todas_como_lidas(
    State(estado): State<EstadoNotificacoes>,
    Extension(contexto): Extension<RequestContext>,
) -> Result<Json<RespostaContagemNaoLidas>, ApiError> {
    let usuario_id = contexto.usuario_id;
    let total = estado
        .marcar_todas_como_lidas
        .executar(usuario_id)
        .await
        .map_err(mapear_erro_notificacao)?;

    Ok(Json(RespostaContagemNaoLidas {
        nao_lidas: total as i64,
    }))
}

/// GET /preferencias-notificacao — buscar preferências do usuário.
pub async fn buscar_preferencia(
    State(estado): State<EstadoNotificacoes>,
    Extension(contexto): Extension<RequestContext>,
) -> Result<Json<Option<RespostaPreferenciaNotificacao>>, ApiError> {
    let usuario_id = contexto.usuario_id;
    let preferencia = estado
        .buscar_preferencia
        .executar(usuario_id)
        .await
        .map_err(mapear_erro_notificacao)?;

    Ok(Json(preferencia.map(RespostaPreferenciaNotificacao::from)))
}

/// POST /preferencias-notificacao — criar/atualizar preferências.
pub async fn criar_ou_atualizar_preferencia(
    State(estado): State<EstadoNotificacoes>,
    Extension(contexto): Extension<RequestContext>,
    Json(requisicao): Json<RequisicaoPreferenciaNotificacao>,
) -> Result<Json<RespostaPreferenciaNotificacao>, ApiError> {
    let usuario_id = contexto.usuario_id;
    let preferencia = estado
        .criar_ou_atualizar_preferencia
        .executar(
            crate::domain::notificacoes::entidades::NovoPreferenciaNotificacao {
                usuario_id,
                canal_email: requisicao.canal_email,
                canal_whatsapp: requisicao.canal_whatsapp,
                canal_in_app: requisicao.canal_in_app,
                tipos_ativados: requisicao.tipos_ativados,
            },
        )
        .await
        .map_err(mapear_erro_notificacao)?;

    Ok(Json(RespostaPreferenciaNotificacao::from(preferencia)))
}

/// GET /templates-notificacao — listar templates ativos.
pub async fn listar_templates(
    State(estado): State<EstadoNotificacoes>,
) -> Result<Json<Vec<crate::presentation::notificacoes::dto::RespostaTemplateNotificacao>>, ApiError>
{
    let templates = estado
        .listar_templates
        .executar()
        .await
        .map_err(mapear_erro_notificacao)?;

    Ok(Json(
        templates
            .into_iter()
            .map(
                |t| crate::presentation::notificacoes::dto::RespostaTemplateNotificacao {
                    id: t.id,
                    tipo: t.tipo.as_str().to_owned(),
                    assunto: t.assunto,
                    corpo_texto: t.corpo_texto,
                    corpo_html: t.corpo_html,
                    variaveis: t.variaveis,
                    ativo: t.ativo,
                    criado_em: t.criado_em,
                    atualizado_em: t.atualizado_em,
                },
            )
            .collect(),
    ))
}

fn mapear_erro_notificacao(erro: ErroNotificacao) -> ApiError {
    match erro {
        ErroNotificacao::NotificacaoNaoEncontrada => ApiError::nova(
            StatusCode::NOT_FOUND,
            "NOTIFICACAO_NAO_ENCONTRADA",
            "Notificação não encontrada.",
        ),
        ErroNotificacao::PreferenciaNaoEncontrada => ApiError::nova(
            StatusCode::NOT_FOUND,
            "PREFERENCIA_NAO_ENCONTRADA",
            "Preferência de notificação não encontrada.",
        ),
        ErroNotificacao::TemplateNaoEncontrado => ApiError::nova(
            StatusCode::NOT_FOUND,
            "TEMPLATE_NAO_ENCONTRADO",
            "Template de notificação não encontrado.",
        ),
        ErroNotificacao::SemPermissao => ApiError::nova(
            StatusCode::FORBIDDEN,
            "SEM_PERMISSAO",
            "Usuário não possui permissão para esta notificação.",
        ),
        ErroNotificacao::CanalNaoConfigurado(canal) => ApiError::nova(
            StatusCode::BAD_REQUEST,
            "CANAL_NAO_CONFIGURADO",
            format!("Canal não configurado: {canal}"),
        ),
        ErroNotificacao::ProvedorIndisponivel(msg) => ApiError::nova(
            StatusCode::SERVICE_UNAVAILABLE,
            "PROVEDOR_INDISPONIVEL",
            &msg,
        ),
        ErroNotificacao::Persistencia(msg) => {
            ApiError::nova(StatusCode::INTERNAL_SERVER_ERROR, "PERSISTENCIA", &msg)
        }
        ErroNotificacao::Interno(msg) => {
            ApiError::nova(StatusCode::INTERNAL_SERVER_ERROR, "ERRO_INTERNO", &msg)
        }
    }
}

use axum::Extension;
