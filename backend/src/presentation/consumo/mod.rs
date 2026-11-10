//! Fase G — apresentação de consumo, planos e assinaturas.

use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    middleware as axum_middleware,
    response::{IntoResponse, Response},
    routing::{get, put},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::application::serpro::medicao::{
    ConsumoAdminDto, ConsumoAdminUseCase, CriarAtualizarAssinaturaUseCase,
    CriarPlanoEscritorioUseCase, ListarConsumoEscritorioUseCase, ListarPlanosUseCase,
};
use crate::domain::serpro::medicao::entidades::PlanoEscritorio;
use crate::domain::serpro::medicao::erros::ErroMedicao;
use crate::presentation::auth::{AuthState, ExigirAudiencia, RequestContext, exigir};
use crate::presentation::error::ApiError;

#[derive(Clone)]
pub struct EstadoConsumo {
    pub consumo_cliente: Arc<ListarConsumoEscritorioUseCase>,
    pub consumo_admin: Arc<ConsumoAdminUseCase>,
    pub planos_listar: Arc<ListarPlanosUseCase>,
    pub planos_criar: Arc<CriarPlanoEscritorioUseCase>,
    pub assinatura_atribuir: Arc<CriarAtualizarAssinaturaUseCase>,
}

fn erro_medicao(erro: ErroMedicao) -> ApiError {
    match erro {
        ErroMedicao::RepositorioIndisponivel => ApiError::indisponivel(),
        ErroMedicao::AssinaturaNaoEncontrada | ErroMedicao::PlanoNaoEncontrado => {
            ApiError::nao_encontrado("plano")
        }
        ErroMedicao::LimiteChamadasExcedido => ApiError::nova(
            StatusCode::TOO_MANY_REQUESTS,
            "LIMITE_SERPRO",
            "Limite mensal de chamadas SERPRO atingido",
        ),
        ErroMedicao::Interno(mensagem) => ApiError::validacao(mensagem),
    }
}

// ===== DTOs =====

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsumoClienteResponse {
    pub escritorio_id: Uuid,
    pub plano_id: Uuid,
    pub chamadas_mes_limite: i64,
    pub clientes_limite: i64,
    pub usuarios_limite: i64,
    pub bloqueio_ao_exceder: bool,
    pub chamadas_usadas_mes: i64,
    pub percentual_utilizacao: f64,
    pub aviso_consumo: Option<String>,
    pub agregados: Vec<AgregadoResponse>,
    pub chamadas_recentes: Vec<ChamadaResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgregadoResponse {
    pub produto: String,
    pub ambiente: String,
    pub periodo: String,
    pub referencia_periodo: DateTime<Utc>,
    pub total_chamadas: i64,
    pub chamadas_sucesso: i64,
    pub chamadas_4xx: i64,
    pub chamadas_5xx: i64,
    pub chamadas_timeout: i64,
    pub chamadas_indeterminado: i64,
    pub duracao_total_ms: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChamadaResponse {
    pub id: Uuid,
    pub produto: String,
    pub verbo: String,
    pub servico: Option<String>,
    pub ambiente: String,
    pub cliente_id: Option<Uuid>,
    pub resultado: String,
    pub status_http: Option<u16>,
    pub duracao_ms: u64,
    pub criado_em: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsumoAdminResponse {
    pub inicio: DateTime<Utc>,
    pub fim: DateTime<Utc>,
    pub total_chamadas: i64,
    pub total_erros: i64,
    pub taxa_erro: f64,
    pub duracao_media_ms: i64,
    pub escritorios: Vec<ConsumoAdminEscritorioResponse>,
    pub ranking: Vec<RankingResponse>,
    pub serie: Vec<SerieResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsumoAdminEscritorioResponse {
    pub escritorio_id: Uuid,
    pub nome: String,
    pub cnpj: String,
    pub plano_nome: Option<String>,
    pub chamadas_limite: Option<i64>,
    pub chamadas_usadas: i64,
    pub chamadas_sucesso: i64,
    pub chamadas_erro: i64,
    pub taxa_erro: f64,
    pub duracao_media_ms: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RankingResponse {
    pub escritorio_id: Uuid,
    pub nome: String,
    pub chamadas: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SerieResponse {
    pub dia: String,
    pub chamadas: i64,
    pub erros: i64,
    pub duracao_media_ms: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanoResponse {
    pub id: Uuid,
    pub nome: String,
    pub descricao: Option<String>,
    pub chamadas_mes: i64,
    pub clientes_limite: i64,
    pub usuarios_limite: i64,
    pub preco_mensal_cents: i64,
    pub ativo: bool,
    pub criado_em: DateTime<Utc>,
    pub atualizado_em: DateTime<Utc>,
}

impl From<PlanoEscritorio> for PlanoResponse {
    fn from(p: PlanoEscritorio) -> Self {
        Self {
            id: p.id,
            nome: p.nome,
            descricao: p.descricao,
            chamadas_mes: p.chamadas_mes,
            clientes_limite: p.clientes_limite,
            usuarios_limite: p.usuarios_limite,
            preco_mensal_cents: p.preco_mensal_cents,
            ativo: p.ativo,
            criado_em: p.criado_em,
            atualizado_em: p.atualizado_em,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CriarPlanoRequest {
    pub nome: String,
    #[serde(default)]
    pub descricao: Option<String>,
    pub chamadas_mes: i64,
    pub clientes_limite: i64,
    pub usuarios_limite: i64,
    #[serde(default)]
    pub preco_mensal_cents: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtribuirPlanoRequest {
    pub plano_id: Uuid,
    pub chamadas_mes_limite: i64,
    #[serde(default = "padrao_bloqueio")]
    pub bloqueio_ao_exceder: bool,
}

const fn padrao_bloqueio() -> bool {
    true
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsumoAdminQuery {
    #[serde(default)]
    pub inicio: Option<DateTime<Utc>>,
    #[serde(default)]
    pub fim: Option<DateTime<Utc>>,
    #[serde(default)]
    pub produto: Option<String>,
    #[serde(default)]
    pub ambiente: Option<String>,
    #[serde(default)]
    pub escritorio_id: Option<Uuid>,
}

// ===== Handlers =====

fn exigir_escritorio(contexto: &RequestContext) -> Result<Uuid, ApiError> {
    contexto.escritorio_id.ok_or_else(ApiError::proibido)
}

/// GET /api/v1/consumo — consumo do mês corrente do escritório do token.
pub async fn consumo_cliente(
    State(estado): State<EstadoConsumo>,
    Extension(contexto): Extension<RequestContext>,
) -> Result<Json<ConsumoClienteResponse>, ApiError> {
    let escritorio_id = exigir_escritorio(&contexto)?;
    let dto = estado
        .consumo_cliente
        .executar(escritorio_id, None, None, None)
        .await
        .map_err(erro_medicao)?;

    Ok(Json(ConsumoClienteResponse {
        escritorio_id: dto.escritorio_id,
        plano_id: dto.plano_id,
        chamadas_mes_limite: dto.chamadas_mes_limite,
        clientes_limite: dto.clientes_limite,
        usuarios_limite: dto.usuarios_limite,
        bloqueio_ao_exceder: dto.bloqueio_ao_exceder,
        chamadas_usadas_mes: dto.chamadas_usadas_mes,
        percentual_utilizacao: dto.percentual_utilizacao,
        aviso_consumo: dto.aviso_consumo,
        agregados: dto
            .agregados
            .into_iter()
            .map(|a| AgregadoResponse {
                produto: a.produto,
                ambiente: a.ambiente,
                periodo: a.periodo,
                referencia_periodo: a.referencia_periodo,
                total_chamadas: a.total_chamadas,
                chamadas_sucesso: a.chamadas_sucesso,
                chamadas_4xx: a.chamadas_4xx,
                chamadas_5xx: a.chamadas_5xx,
                chamadas_timeout: a.chamadas_timeout,
                chamadas_indeterminado: a.chamadas_indeterminado,
                duracao_total_ms: a.duracao_total_ms,
            })
            .collect(),
        chamadas_recentes: dto
            .chamadas_recentes
            .into_iter()
            .map(|c| ChamadaResponse {
                id: c.id,
                produto: c.produto,
                verbo: c.verbo,
                servico: c.servico,
                ambiente: c.ambiente,
                cliente_id: c.cliente_id,
                resultado: c.resultado,
                status_http: c.status_http,
                duracao_ms: c.duracao_ms,
                criado_em: c.criado_em,
            })
            .collect(),
    }))
}

fn periodo_padrao() -> (DateTime<Utc>, DateTime<Utc>) {
    let agora = Utc::now();
    let inicio = crate::application::serpro::medicao::use_cases::inicio_mes(agora);
    (inicio, agora)
}

async fn calcular_admin(
    estado: &EstadoConsumo,
    filtros: &ConsumoAdminQuery,
) -> Result<ConsumoAdminDto, ApiError> {
    let (inicio_padrao, fim_padrao) = periodo_padrao();
    estado
        .consumo_admin
        .executar(
            filtros.inicio.unwrap_or(inicio_padrao),
            filtros.fim.unwrap_or(fim_padrao),
            filtros.produto.clone(),
            filtros.ambiente.clone(),
            filtros.escritorio_id,
        )
        .await
        .map_err(erro_medicao)
}

fn montar_admin(dto: ConsumoAdminDto) -> ConsumoAdminResponse {
    ConsumoAdminResponse {
        inicio: dto.inicio,
        fim: dto.fim,
        total_chamadas: dto.total_chamadas,
        total_erros: dto.total_erros,
        taxa_erro: dto.taxa_erro,
        duracao_media_ms: dto.duracao_media_ms,
        escritorios: dto
            .escritorios
            .into_iter()
            .map(|e| ConsumoAdminEscritorioResponse {
                escritorio_id: e.escritorio_id,
                nome: e.nome,
                cnpj: e.cnpj,
                plano_nome: e.plano_nome,
                chamadas_limite: e.chamadas_limite,
                chamadas_usadas: e.chamadas_usadas,
                chamadas_sucesso: e.chamadas_sucesso,
                chamadas_erro: e.chamadas_erro,
                taxa_erro: e.taxa_erro,
                duracao_media_ms: e.duracao_media_ms,
            })
            .collect(),
        ranking: dto
            .ranking
            .into_iter()
            .map(|r| RankingResponse {
                escritorio_id: r.escritorio_id,
                nome: r.nome,
                chamadas: r.chamadas,
            })
            .collect(),
        serie: dto
            .serie
            .into_iter()
            .map(|s| SerieResponse {
                dia: s.dia,
                chamadas: s.chamadas,
                erros: s.erros,
                duracao_media_ms: s.duracao_media_ms,
            })
            .collect(),
    }
}

/// GET /api/v1/admin/consumo
pub async fn consumo_admin(
    State(estado): State<EstadoConsumo>,
    Extension(_contexto): Extension<RequestContext>,
    Query(filtros): Query<ConsumoAdminQuery>,
) -> Result<Json<ConsumoAdminResponse>, ApiError> {
    Ok(Json(montar_admin(calcular_admin(&estado, &filtros).await?)))
}

/// GET /api/v1/admin/consumo/export.csv
pub async fn exportar_consumo_csv(
    State(estado): State<EstadoConsumo>,
    Extension(_contexto): Extension<RequestContext>,
    Query(filtros): Query<ConsumoAdminQuery>,
) -> Result<Response, ApiError> {
    let dto = montar_admin(calcular_admin(&estado, &filtros).await?);
    let mut csv = String::from(
        "escritorio_id,nome,cnpj,plano,chamadas_limite,chamadas_usadas,sucesso,erros,taxa_erro,duracao_media_ms\n",
    );
    for e in &dto.escritorios {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{:.4},{}\n",
            e.escritorio_id,
            e.nome.replace(',', " "),
            e.cnpj,
            e.plano_nome.clone().unwrap_or_default().replace(',', " "),
            e.chamadas_limite.map(|v| v.to_string()).unwrap_or_default(),
            e.chamadas_usadas,
            e.chamadas_sucesso,
            e.chamadas_erro,
            e.taxa_erro,
            e.duracao_media_ms
        ));
    }

    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "text/csv; charset=utf-8")
        .header(
            "content-disposition",
            "attachment; filename=\"consumo-serpro.csv\"",
        )
        .body(axum::body::Body::from(csv))
        .map_err(|_| ApiError::erro_interno())
}

/// GET /api/v1/admin/planos
pub async fn listar_planos(
    State(estado): State<EstadoConsumo>,
    Extension(_contexto): Extension<RequestContext>,
) -> Result<Json<Vec<PlanoResponse>>, ApiError> {
    let planos = estado
        .planos_listar
        .executar_todos()
        .await
        .map_err(erro_medicao)?;
    Ok(Json(planos.into_iter().map(PlanoResponse::from).collect()))
}

/// POST /api/v1/admin/planos
pub async fn criar_plano(
    State(estado): State<EstadoConsumo>,
    Extension(_contexto): Extension<RequestContext>,
    Json(payload): Json<CriarPlanoRequest>,
) -> Result<Response, ApiError> {
    let plano = estado
        .planos_criar
        .executar(
            payload.nome,
            payload.descricao,
            payload.chamadas_mes,
            payload.clientes_limite,
            payload.usuarios_limite,
            payload.preco_mensal_cents,
        )
        .await
        .map_err(erro_medicao)?;
    Ok((StatusCode::CREATED, Json(PlanoResponse::from(plano))).into_response())
}

/// PUT /api/v1/admin/planos/escritorios/{id} — atribui plano/limite ao escritório.
pub async fn atribuir_plano(
    State(estado): State<EstadoConsumo>,
    Extension(_contexto): Extension<RequestContext>,
    Path(escritorio_id): Path<Uuid>,
    Json(payload): Json<AtribuirPlanoRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let assinatura = estado
        .assinatura_atribuir
        .executar(
            escritorio_id,
            payload.plano_id,
            payload.chamadas_mes_limite,
            payload.bloqueio_ao_exceder,
        )
        .await
        .map_err(erro_medicao)?;
    Ok(Json(serde_json::json!({
        "id": assinatura.id,
        "escritorioId": assinatura.escritorio_id,
        "planoId": assinatura.plano_id,
        "chamadasMesLimite": assinatura.chamadas_mes_limite,
        "clientesLimite": assinatura.clientes_limite,
        "usuariosLimite": assinatura.usuarios_limite,
        "bloqueioAoExceder": assinatura.bloqueio_ao_exceder,
        "ativo": assinatura.ativo,
        "atualizadoEm": assinatura.atualizado_em,
    })))
}

pub fn construir_cliente(estado: EstadoConsumo, auth: AuthState) -> Router {
    Router::new()
        .route("/consumo", get(consumo_cliente))
        .route_layer(axum_middleware::from_fn_with_state(
            ExigirAudiencia::novo(auth.tokens.clone(), crate::domain::auth::Audiencia::Client),
            exigir,
        ))
        .with_state(estado)
}

pub fn construir_admin(estado: EstadoConsumo, auth: AuthState) -> Router {
    Router::new()
        .route("/admin/consumo", get(consumo_admin))
        .route("/admin/consumo/export.csv", get(exportar_consumo_csv))
        .route("/admin/planos", get(listar_planos).post(criar_plano))
        .route("/admin/planos/escritorios/{id}", put(atribuir_plano))
        .route_layer(axum_middleware::from_fn_with_state(
            ExigirAudiencia::novo(auth.tokens.clone(), crate::domain::auth::Audiencia::Admin),
            exigir,
        ))
        .with_state(estado)
}

pub fn rotas_admin() -> Router<EstadoConsumo> {
    Router::new()
        .route("/admin/consumo", get(consumo_admin))
        .route("/admin/consumo/export.csv", get(exportar_consumo_csv))
        .route("/admin/planos", get(listar_planos).post(criar_plano))
        .route("/admin/planos/escritorios/{id}", put(atribuir_plano))
}

pub fn rotas_cliente() -> Router<EstadoConsumo> {
    Router::new().route("/consumo", get(consumo_cliente))
}
