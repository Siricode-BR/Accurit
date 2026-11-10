//! Fase F — Handlers de relatórios e link de compartilhamento.

use axum::Json;
use axum::extract::{Extension, Path, State};
use axum::http::{StatusCode, header};
use axum::response::Response;
use uuid::Uuid;

use crate::domain::relatorios::enums::StatusRelatorio;
use crate::presentation::auth::RequestContext;
use crate::presentation::error::ApiError;
use crate::presentation::relatorios::dto::{
    CriarLinkCompartilhamentoRequest, CriarRelatorioRequest, LinkCompartilhamentoDto, RelatorioDto,
    RelatorioResumoDto,
};
use crate::presentation::relatorios::state::EstadoRelatorios;

/// POST /relatorios — gera um novo relatório.
pub async fn criar_relatorio(
    State(estado): State<EstadoRelatorios>,
    Extension(contexto): Extension<RequestContext>,
    Json(requisicao): Json<CriarRelatorioRequest>,
) -> Result<Json<RelatorioDto>, ApiError> {
    let _escritorio_id = contexto.escritorio_id.ok_or_else(ApiError::proibido)?;
    let comando: crate::application::relatorios::use_cases::ComandoGerarRelatorio = requisicao
        .try_into()
        .map_err(|e: String| ApiError::validacao(e))?;

    let relatorio = estado.gerar.executar(comando).await.map_err(|e| match e {
        crate::domain::relatorios::erros::ErroRelatorio::TipoInvalido(_)
        | crate::domain::relatorios::erros::ErroRelatorio::FormatoInvalido(_) => {
            ApiError::validacao(e.to_string())
        }
        _ => ApiError::erro_interno(),
    })?;

    Ok(Json(RelatorioDto::from(relatorio)))
}

/// GET /relatorios — lista relatórios do escritório.
pub async fn listar_relatorios(
    State(estado): State<EstadoRelatorios>,
    Extension(contexto): Extension<RequestContext>,
) -> Result<Json<Vec<RelatorioResumoDto>>, ApiError> {
    let _escritorio_id = contexto.escritorio_id.ok_or_else(ApiError::proibido)?;

    let relatorios = estado
        .listar
        .executar(_escritorio_id, 50)
        .await
        .map_err(|e| match e {
            crate::domain::relatorios::erros::ErroRelatorio::TipoInvalido(_)
            | crate::domain::relatorios::erros::ErroRelatorio::FormatoInvalido(_) => {
                ApiError::validacao(e.to_string())
            }
            _ => ApiError::erro_interno(),
        })?;

    Ok(Json(
        relatorios
            .into_iter()
            .map(|r| RelatorioResumoDto {
                id: r.id,
                tipo_relatorio: r.tipo_relatorio,
                formato: r.formato,
                status: r.status,
                nome_arquivo: r.nome_arquivo,
                expira_em: r.expira_em,
                criado_em: r.criado_em,
            })
            .collect(),
    ))
}

/// GET /relatorios/{id} — busca um relatório.
pub async fn buscar_relatorio(
    State(estado): State<EstadoRelatorios>,
    Extension(contexto): Extension<RequestContext>,
    Path(_id): Path<Uuid>,
) -> Result<Json<RelatorioDto>, ApiError> {
    let _escritorio_id = contexto.escritorio_id.ok_or_else(ApiError::proibido)?;

    let relatorio = estado
        .buscar
        .executar(_id, _escritorio_id)
        .await
        .map_err(|e| match e {
            crate::domain::relatorios::erros::ErroRelatorio::RelatorioNaoEncontrado => {
                ApiError::nao_encontrado("relatório")
            }
            _ => ApiError::erro_interno(),
        })?;

    Ok(Json(RelatorioDto::from(relatorio)))
}

/// GET /relatorios/{id}/baixar — baixa o conteúdo do relatório.
pub async fn baixar_relatorio(
    State(estado): State<EstadoRelatorios>,
    Extension(contexto): Extension<RequestContext>,
    Path(_id): Path<Uuid>,
) -> Result<Response, ApiError> {
    let _escritorio_id = contexto.escritorio_id.ok_or_else(ApiError::proibido)?;

    let relatorio = estado
        .buscar
        .executar(_id, _escritorio_id)
        .await
        .map_err(|e| match e {
            crate::domain::relatorios::erros::ErroRelatorio::RelatorioNaoEncontrado => {
                ApiError::nao_encontrado("relatório")
            }
            _ => ApiError::erro_interno(),
        })?;

    if relatorio.status != StatusRelatorio::Concluido {
        return Err(ApiError::validacao("relatório ainda não foi gerado"));
    }

    let conteudo = relatorio
        .conteudo
        .ok_or_else(|| ApiError::nao_encontrado("conteúdo do relatório"))?;
    let nome_arquivo = relatorio
        .nome_arquivo
        .unwrap_or_else(|| "relatorio".to_owned());
    let extensao = nome_arquivo.rsplit('.').next().unwrap_or("bin");
    let content_type = match extensao {
        "pdf" => "application/pdf",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        _ => "application/octet-stream",
    };

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{nome_arquivo}\""),
        )
        .body(axum::body::Body::from(conteudo))
        .map_err(|_| ApiError::erro_interno())
}

/// POST /relatorios/{id}/compartilhar — cria um link de compartilhamento.
pub async fn criar_link_compartilhamento(
    State(estado): State<EstadoRelatorios>,
    Extension(contexto): Extension<RequestContext>,
    Path(_id): Path<Uuid>,
    Json(requisicao): Json<CriarLinkCompartilhamentoRequest>,
) -> Result<Json<LinkCompartilhamentoDto>, ApiError> {
    let _escritorio_id = contexto.escritorio_id.ok_or_else(ApiError::proibido)?;

    let comando: crate::application::relatorios::use_cases::ComandoCriarLink = requisicao
        .try_into()
        .map_err(|e: String| ApiError::validacao(e))?;

    let link = estado
        .criar_link
        .executar(comando)
        .await
        .map_err(|e| match e {
            crate::domain::relatorios::erros::ErroRelatorio::RelatorioNaoEncontrado => {
                ApiError::nao_encontrado("relatório")
            }
            _ => ApiError::erro_interno(),
        })?;

    Ok(Json(LinkCompartilhamentoDto::from(link)))
}

/// GET /relatorios/compartilhado/{token} — acessa um relatório via link público (somente leitura).
pub async fn obter_relatorio_por_link(
    State(estado): State<EstadoRelatorios>,
    Path(token): Path<String>,
) -> Result<Response, ApiError> {
    let agora = chrono::Utc::now();
    let detalhe = estado
        .acessar_por_link
        .executar(&token, agora)
        .await
        .map_err(|e| match e {
            crate::domain::relatorios::erros::ErroRelatorio::LinkNaoEncontrado => {
                ApiError::nao_encontrado("link")
            }
            crate::domain::relatorios::erros::ErroRelatorio::LinkExpirado => {
                ApiError::nao_encontrado("link expirado")
            }
            _ => ApiError::erro_interno(),
        })?;

    let extensao = detalhe.nome_arquivo.rsplit('.').next().unwrap_or("bin");
    let content_type = match extensao {
        "pdf" => "application/pdf",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        _ => "application/octet-stream",
    };

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", detalhe.nome_arquivo),
        )
        .body(axum::body::Body::from(detalhe.conteudo))
        .map_err(|_| ApiError::erro_interno())
}
