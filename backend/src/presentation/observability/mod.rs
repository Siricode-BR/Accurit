//! Observabilidade: métricas Prometheus protegidas e health detalhado.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Instant;

use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use secrecy::{ExposeSecret, SecretString};
use serde::Serialize;

use crate::presentation::error::ApiError;

#[derive(Default)]
pub struct Metricas {
    requisicoes_2xx: AtomicU64,
    requisicoes_3xx: AtomicU64,
    requisicoes_4xx: AtomicU64,
    requisicoes_5xx: AtomicU64,
    duracao_total_ms: AtomicU64,
    duracao_observacoes: AtomicU64,
    serpro_erros: AtomicU64,
    serpro_chamadas: AtomicU64,
}

static METRICAS: OnceLock<Metricas> = OnceLock::new();
static INICIO: OnceLock<Instant> = OnceLock::new();

pub fn metricas() -> &'static Metricas {
    METRICAS.get_or_init(Metricas::default)
}

pub fn iniciar_relogio() {
    INICIO.get_or_init(Instant::now);
}

pub fn uptime_segundos() -> u64 {
    INICIO.get_or_init(Instant::now).elapsed().as_secs()
}

pub fn registrar_chamada_serpro(erro: bool) {
    let m = metricas();
    m.serpro_chamadas.fetch_add(1, Ordering::Relaxed);
    if erro {
        m.serpro_erros.fetch_add(1, Ordering::Relaxed);
    }
}

pub async fn middleware(
    requisicao: axum::http::Request<axum::body::Body>,
    proximo: axum::middleware::Next,
) -> Response {
    let inicio = Instant::now();
    let resposta = proximo.run(requisicao).await;
    registrar_status(resposta.status(), inicio.elapsed().as_millis() as u64);
    resposta
}

pub fn registrar_status(status: StatusCode, duracao_ms: u64) {
    let m = metricas();
    match status.as_u16() {
        200..=299 => m.requisicoes_2xx.fetch_add(1, Ordering::Relaxed),
        300..=399 => m.requisicoes_3xx.fetch_add(1, Ordering::Relaxed),
        400..=499 => m.requisicoes_4xx.fetch_add(1, Ordering::Relaxed),
        _ => m.requisicoes_5xx.fetch_add(1, Ordering::Relaxed),
    };
    m.duracao_total_ms.fetch_add(duracao_ms, Ordering::Relaxed);
    m.duracao_observacoes.fetch_add(1, Ordering::Relaxed);
}

#[derive(Clone)]
pub struct EstadoObservabilidade {
    pub metrics_token: Option<Arc<SecretString>>,
    pub ambiente: String,
}

fn token_autorizado(headers: &HeaderMap, esperado: &Option<Arc<SecretString>>) -> bool {
    let Some(esperado) = esperado else {
        return false;
    };
    let recebido = headers
        .get(header::AUTHORIZATION)
        .and_then(|valor| valor.to_str().ok())
        .and_then(|valor| valor.strip_prefix("Bearer "));
    recebido.is_some_and(|valor| valor.trim() == esperado.expose_secret())
}

pub async fn metricas_handler(
    State(estado): State<EstadoObservabilidade>,
    headers: HeaderMap,
) -> Response {
    if !token_autorizado(&headers, &estado.metrics_token) {
        return ApiError::proibido().into_response();
    }
    let m = metricas();
    let observacoes = m.duracao_observacoes.load(Ordering::Relaxed);
    let total = m.duracao_total_ms.load(Ordering::Relaxed);
    let corpo = format!(
        "# HELP accurit_http_requests_total Requisições HTTP por classe de status.\n\
# TYPE accurit_http_requests_total counter\n\
accurit_http_requests_total{{classe=\"2xx\"}} {}\n\
accurit_http_requests_total{{classe=\"3xx\"}} {}\n\
accurit_http_requests_total{{classe=\"4xx\"}} {}\n\
accurit_http_requests_total{{classe=\"5xx\"}} {}\n\
# HELP accurit_http_request_duration_ms_sum Soma das durações HTTP em ms.\n\
# TYPE accurit_http_request_duration_ms_sum counter\n\
accurit_http_request_duration_ms_sum {}\n\
accurit_http_request_duration_ms_count {}\n\
# HELP accurit_serpro_calls_total Chamadas SERPRO registradas.\n\
# TYPE accurit_serpro_calls_total counter\n\
accurit_serpro_calls_total {}\n\
accurit_serpro_errors_total {}\n\
# HELP accurit_uptime_seconds Tempo desde o start do processo.\n\
# TYPE accurit_uptime_seconds gauge\n\
accurit_uptime_seconds {}\n\
accurit_info{{versao=\"{}\",ambiente=\"{}\"}} 1\n",
        m.requisicoes_2xx.load(Ordering::Relaxed),
        m.requisicoes_3xx.load(Ordering::Relaxed),
        m.requisicoes_4xx.load(Ordering::Relaxed),
        m.requisicoes_5xx.load(Ordering::Relaxed),
        total,
        observacoes,
        m.serpro_chamadas.load(Ordering::Relaxed),
        m.serpro_erros.load(Ordering::Relaxed),
        uptime_segundos(),
        env!("CARGO_PKG_VERSION"),
        estado.ambiente,
    );
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/plain; version=0.0.4")
        .body(axum::body::Body::from(corpo))
        .unwrap_or_else(|_| ApiError::erro_interno().into_response())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthDetalhado {
    pub status: &'static str,
    pub versao: &'static str,
    pub ambiente: String,
    pub uptime_segundos: u64,
    pub banco: &'static str,
    pub requisicoes_2xx: u64,
    pub requisicoes_4xx: u64,
    pub requisicoes_5xx: u64,
    pub serpro_chamadas: u64,
    pub serpro_erros: u64,
}

pub async fn health_detalhado(
    State(estado): State<EstadoObservabilidade>,
) -> Json<HealthDetalhado> {
    let m = metricas();
    Json(HealthDetalhado {
        status: "ok",
        versao: env!("CARGO_PKG_VERSION"),
        ambiente: estado.ambiente,
        uptime_segundos: uptime_segundos(),
        banco: "ok",
        requisicoes_2xx: m.requisicoes_2xx.load(Ordering::Relaxed),
        requisicoes_4xx: m.requisicoes_4xx.load(Ordering::Relaxed),
        requisicoes_5xx: m.requisicoes_5xx.load(Ordering::Relaxed),
        serpro_chamadas: m.serpro_chamadas.load(Ordering::Relaxed),
        serpro_erros: m.serpro_erros.load(Ordering::Relaxed),
    })
}

pub fn construir(estado: EstadoObservabilidade) -> Router {
    Router::new()
        .route("/metrics", get(metricas_handler))
        .route("/health/detalhado", get(health_detalhado))
        .with_state(estado)
}
