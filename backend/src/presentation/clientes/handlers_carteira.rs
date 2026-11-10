//! Fase E — Handlers de sócios/contatos, filiais, tags, carteira e importação.
//!
//! Toda escrita checa `guardas::exigir_escrita`; toda leitura específica de
//! cliente checa `guardas::exigir_ver`. Os casos de uso da carteira são
//! síncronos.

use axum::{
    Json,
    extract::{Extension, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::application::clientes::{
    NovaClienteTagCommand, NovoClienteFilialCommand, NovoSocioContatoCommand,
};
use crate::domain::clientes::PapelSocioContato;
use crate::presentation::auth::RequestContext;
use crate::presentation::clientes::dto::{
    AtualizarCarteiraRequest, CarteiraClienteResponse, CriarTagRequest, FilialRequest,
    FilialResponse, ImportacaoRequest, SocioContatoRequest, SocioContatoResponse, TagResponse,
    VincularTagRequest,
};
use crate::presentation::clientes::guardas;
use crate::presentation::clientes::router::ClientesRouterState;
use crate::presentation::error::ApiError;

fn escritorio(contexto: &RequestContext) -> Result<Uuid, ApiError> {
    contexto.escritorio_id.ok_or_else(ApiError::proibido)
}

// ===== Sócios e contatos =====

pub async fn criar_socio_contato(
    State(state): State<ClientesRouterState>,
    Extension(contexto): Extension<RequestContext>,
    Path(cliente_id): Path<Uuid>,
    Json(payload): Json<SocioContatoRequest>,
) -> Result<Response, ApiError> {
    let escritorio_id = escritorio(&contexto)?;
    let acesso = guardas::acesso_do_contexto(&state.verificar, &contexto)?;
    guardas::exigir_escrita(&acesso, Some(cliente_id))?;

    let comando = NovoSocioContatoCommand {
        escritorio_id,
        cliente_id,
        papel: PapelSocioContato::de_str(&payload.papel)
            .ok_or_else(|| ApiError::validacao("papel deve ser SOCIO ou CONTATO"))?,
        nome: payload.nome,
        documento: payload.documento,
        email: payload.email,
        telefone: payload.telefone,
        representante_principal: payload.representante_principal,
        usuario_id: contexto.usuario_id,
        ip: contexto.ip,
    };
    let criado = state.socios.criar(&comando)?;
    Ok((
        StatusCode::CREATED,
        Json(SocioContatoResponse::from(criado)),
    )
        .into_response())
}

pub async fn listar_socios_contatos(
    State(state): State<ClientesRouterState>,
    Extension(contexto): Extension<RequestContext>,
    Path(cliente_id): Path<Uuid>,
) -> Result<Response, ApiError> {
    let acesso = guardas::acesso_do_contexto(&state.verificar, &contexto)?;
    guardas::exigir_ver(&acesso, Some(cliente_id))?;

    let itens: Vec<SocioContatoResponse> = state
        .socios
        .listar(escritorio(&contexto)?, cliente_id)?
        .into_iter()
        .map(Into::into)
        .collect();
    Ok(Json(serde_json::json!({ "itens": itens })).into_response())
}

pub async fn remover_socio_contato(
    State(state): State<ClientesRouterState>,
    Extension(contexto): Extension<RequestContext>,
    Path((cliente_id, id)): Path<(Uuid, Uuid)>,
) -> Result<Response, ApiError> {
    let escritorio_id = escritorio(&contexto)?;
    let acesso = guardas::acesso_do_contexto(&state.verificar, &contexto)?;
    guardas::exigir_escrita(&acesso, Some(cliente_id))?;

    state.socios.remover(
        escritorio_id,
        cliente_id,
        id,
        contexto.usuario_id,
        contexto.ip,
    )?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

// ===== Filiais =====

pub async fn criar_filial(
    State(state): State<ClientesRouterState>,
    Extension(contexto): Extension<RequestContext>,
    Path(cliente_id): Path<Uuid>,
    Json(payload): Json<FilialRequest>,
) -> Result<Response, ApiError> {
    let escritorio_id = escritorio(&contexto)?;
    let acesso = guardas::acesso_do_contexto(&state.verificar, &contexto)?;
    guardas::exigir_escrita(&acesso, Some(cliente_id))?;

    let comando = NovoClienteFilialCommand {
        escritorio_id,
        cliente_id,
        cnpj: payload.cnpj,
        nome_unidade: payload.nome_unidade,
        endereco: payload.endereco,
        telefone: payload.telefone,
        usuario_id: contexto.usuario_id,
        ip: contexto.ip,
    };
    let criado = state.filiais.criar(&comando)?;
    Ok((StatusCode::CREATED, Json(FilialResponse::from(criado))).into_response())
}

pub async fn listar_filiais(
    State(state): State<ClientesRouterState>,
    Extension(contexto): Extension<RequestContext>,
    Path(cliente_id): Path<Uuid>,
) -> Result<Response, ApiError> {
    let acesso = guardas::acesso_do_contexto(&state.verificar, &contexto)?;
    guardas::exigir_ver(&acesso, Some(cliente_id))?;

    let itens: Vec<FilialResponse> = state
        .filiais
        .listar(escritorio(&contexto)?, cliente_id)?
        .into_iter()
        .map(Into::into)
        .collect();
    Ok(Json(serde_json::json!({ "itens": itens })).into_response())
}

pub async fn remover_filial(
    State(state): State<ClientesRouterState>,
    Extension(contexto): Extension<RequestContext>,
    Path((cliente_id, id)): Path<(Uuid, Uuid)>,
) -> Result<Response, ApiError> {
    let escritorio_id = escritorio(&contexto)?;
    let acesso = guardas::acesso_do_contexto(&state.verificar, &contexto)?;
    guardas::exigir_escrita(&acesso, Some(cliente_id))?;

    state.filiais.remover(
        escritorio_id,
        cliente_id,
        id,
        contexto.usuario_id,
        contexto.ip,
    )?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

// ===== Tags =====

pub async fn listar_tags(
    State(state): State<ClientesRouterState>,
    Extension(contexto): Extension<RequestContext>,
) -> Result<Response, ApiError> {
    guardas::acesso_do_contexto(&state.verificar, &contexto)?;
    let itens: Vec<TagResponse> = state
        .tags
        .listar_tags(escritorio(&contexto)?)?
        .into_iter()
        .map(Into::into)
        .collect();
    Ok(Json(serde_json::json!({ "itens": itens })).into_response())
}

pub async fn criar_tag(
    State(state): State<ClientesRouterState>,
    Extension(contexto): Extension<RequestContext>,
    Json(payload): Json<CriarTagRequest>,
) -> Result<Response, ApiError> {
    let acesso = guardas::acesso_do_contexto(&state.verificar, &contexto)?;
    guardas::exigir_escrita(&acesso, None)?;

    let comando = NovaClienteTagCommand {
        escritorio_id: escritorio(&contexto)?,
        nome: payload.nome,
        cor: payload.cor,
        usuario_id: contexto.usuario_id,
        ip: contexto.ip,
    };
    let criada = state.tags.criar_tag(&comando)?;
    Ok((StatusCode::CREATED, Json(TagResponse::from(criada))).into_response())
}

pub async fn listar_tags_do_cliente(
    State(state): State<ClientesRouterState>,
    Extension(contexto): Extension<RequestContext>,
    Path(cliente_id): Path<Uuid>,
) -> Result<Response, ApiError> {
    let acesso = guardas::acesso_do_contexto(&state.verificar, &contexto)?;
    guardas::exigir_ver(&acesso, Some(cliente_id))?;
    let itens: Vec<TagResponse> = state
        .tags
        .tags_do_cliente(escritorio(&contexto)?, cliente_id)?
        .into_iter()
        .map(Into::into)
        .collect();
    Ok(Json(serde_json::json!({ "itens": itens })).into_response())
}

pub async fn vincular_tag(
    State(state): State<ClientesRouterState>,
    Extension(contexto): Extension<RequestContext>,
    Path(cliente_id): Path<Uuid>,
    Json(payload): Json<VincularTagRequest>,
) -> Result<Response, ApiError> {
    let escritorio_id = escritorio(&contexto)?;
    let acesso = guardas::acesso_do_contexto(&state.verificar, &contexto)?;
    guardas::exigir_escrita(&acesso, Some(cliente_id))?;
    state.tags.vincular(
        escritorio_id,
        cliente_id,
        payload.tag_id,
        contexto.usuario_id,
        contexto.ip,
    )?;
    Ok(StatusCode::CREATED.into_response())
}

pub async fn desvincular_tag(
    State(state): State<ClientesRouterState>,
    Extension(contexto): Extension<RequestContext>,
    Path((cliente_id, tag_id)): Path<(Uuid, Uuid)>,
) -> Result<Response, ApiError> {
    let escritorio_id = escritorio(&contexto)?;
    let acesso = guardas::acesso_do_contexto(&state.verificar, &contexto)?;
    guardas::exigir_escrita(&acesso, Some(cliente_id))?;
    state.tags.desvincular(
        escritorio_id,
        cliente_id,
        tag_id,
        contexto.usuario_id,
        contexto.ip,
    )?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

// ===== Carteira (responsável/observações) =====

pub async fn obter_carteira(
    State(state): State<ClientesRouterState>,
    Extension(contexto): Extension<RequestContext>,
    Path(cliente_id): Path<Uuid>,
) -> Result<Response, ApiError> {
    let acesso = guardas::acesso_do_contexto(&state.verificar, &contexto)?;
    guardas::exigir_ver(&acesso, Some(cliente_id))?;
    let carteira = state.carteira.obter(escritorio(&contexto)?, cliente_id)?;
    let resposta = match carteira {
        Some(c) => CarteiraClienteResponse::from(c),
        None => CarteiraClienteResponse {
            cliente_id,
            responsavel_usuario_id: None,
            observacoes: None,
            criado_em: chrono::Utc::now(),
            atualizado_em: chrono::Utc::now(),
        },
    };
    Ok(Json(resposta).into_response())
}

pub async fn definir_carteira(
    State(state): State<ClientesRouterState>,
    Extension(contexto): Extension<RequestContext>,
    Path(cliente_id): Path<Uuid>,
    Json(payload): Json<AtualizarCarteiraRequest>,
) -> Result<Response, ApiError> {
    let escritorio_id = escritorio(&contexto)?;
    let acesso = guardas::acesso_do_contexto(&state.verificar, &contexto)?;
    guardas::exigir_escrita(&acesso, Some(cliente_id))?;
    let carteira = state.carteira.definir(
        escritorio_id,
        cliente_id,
        payload.responsavel_usuario_id,
        payload.limpar_responsavel,
        payload.observacoes,
        payload.limpar_observacoes,
        contexto.usuario_id,
        contexto.ip,
    )?;
    Ok(Json(CarteiraClienteResponse::from(carteira)).into_response())
}

// ===== Importação em massa (CSV/XLSX) =====

fn decodificar_requisicao(
    payload: ImportacaoRequest,
) -> Result<
    (
        Vec<u8>,
        crate::application::clientes::importar::FormatoImportacao,
    ),
    ApiError,
> {
    use base64::Engine;

    use crate::application::clientes::importar::FormatoImportacao;

    let formato = match payload.formato.trim().to_ascii_uppercase().as_str() {
        "CSV" => FormatoImportacao::Csv,
        "XLSX" => FormatoImportacao::Xlsx,
        _ => return Err(ApiError::validacao("formato deve ser CSV ou XLSX")),
    };
    let conteudo = base64::engine::general_purpose::STANDARD
        .decode(payload.conteudo_base64.trim())
        .map_err(|_| ApiError::validacao("conteúdo base64 inválido"))?;
    Ok((conteudo, formato))
}

/// Etapa 1: prévia. Nada persiste.
pub async fn importar_analise(
    State(state): State<ClientesRouterState>,
    Extension(contexto): Extension<RequestContext>,
    Json(payload): Json<ImportacaoRequest>,
) -> Result<Response, ApiError> {
    let acesso = guardas::acesso_do_contexto(&state.verificar, &contexto)?;
    guardas::exigir_escrita(&acesso, None)?;

    let (conteudo, formato) = decodificar_requisicao(payload)?;
    let analise = state.importar.analisar(&conteudo, formato)?;
    Ok(Json(analise).into_response())
}

/// Etapa 2: confirmar — cria apenas as linhas válidas não duplicadas.
pub async fn importar_confirmar(
    State(state): State<ClientesRouterState>,
    Extension(contexto): Extension<RequestContext>,
    Json(payload): Json<ImportacaoRequest>,
) -> Result<Response, ApiError> {
    let acesso = guardas::acesso_do_contexto(&state.verificar, &contexto)?;
    guardas::exigir_escrita(&acesso, None)?;

    let (conteudo, formato) = decodificar_requisicao(payload)?;
    let escritorio_id = escritorio(&contexto)?;
    let analise = state.importar.analisar(&conteudo, formato)?;
    let relatorio = state.importar.confirmar(
        escritorio_id,
        contexto.usuario_id,
        &analise.validas,
        contexto.ip,
    )?;
    Ok(Json(relatorio).into_response())
}

/// Acesso do usuário ao portal (para o frontend saber o que mostra, Fase E).
pub async fn meu_acesso(
    State(state): State<ClientesRouterState>,
    Extension(contexto): Extension<RequestContext>,
) -> Result<Response, ApiError> {
    let acesso = guardas::acesso_do_contexto(&state.verificar, &contexto)?;
    Ok(Json(serde_json::json!({
        "funcao": acesso.funcao.como_str(),
        "podeEscrever": acesso.funcao.pode_escrever_clientes(),
        "administraUsuarios": acesso.funcao.administra_usuarios(),
        "clientesPermitidos": acesso.clientes_permitidos,
    }))
    .into_response())
}
