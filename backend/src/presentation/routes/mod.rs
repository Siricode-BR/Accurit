use axum::Router;
use axum::http::Request;
use axum::middleware as axum_middleware;
use axum::routing::get;
use tower_http::trace::{DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;

use crate::application::ports::SaudeBanco;
use crate::infrastructure::config::{BURST_RATE_LIMIT, Config, REQUISICOES_POR_SEGUNDO};
use crate::presentation::auth::{AuthState, construir as construir_auth};
use crate::presentation::handlers;
use crate::presentation::middleware::{cors, rate_limit, request_id, security_headers};
use crate::presentation::state::AppState;

pub fn construir<S>(estado: AppState<S>, auth: Option<AuthState>, config: &Config) -> Router
where
    S: SaudeBanco + 'static,
{
    let limite = rate_limit::LimiteRequisicoes::novo(REQUISICOES_POR_SEGUNDO, BURST_RATE_LIMIT);

    let mut api = Router::new()
        .route("/health", get(handlers::saude::<S>))
        .with_state(estado);

    if let Some(auth) = auth {
        api = api.nest("/auth", construir_auth(auth));
    }

    let api = api
        .fallback(handlers::nao_encontrado)
        .method_not_allowed_fallback(handlers::metodo_nao_permitido)
        .layer(axum_middleware::from_fn_with_state(
            limite,
            rate_limit::middleware,
        ))
        .layer(cors::camada(&config.cors_allowed_origins))
        .layer(axum_middleware::from_fn(security_headers::middleware));

    Router::new()
        .nest("/api/v1", api)
        .fallback(handlers::nao_encontrado)
        .method_not_allowed_fallback(handlers::metodo_nao_permitido)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|requisicao: &Request<_>| {
                    let request_id = requisicao
                        .extensions()
                        .get::<request_id::RequestId>()
                        .map(request_id::RequestId::como_str)
                        .unwrap_or("-");

                    tracing::info_span!(
                        "http_request",
                        method = %requisicao.method(),
                        path = %requisicao.uri().path(),
                        request_id = %request_id
                    )
                })
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(
                    DefaultOnResponse::new()
                        .level(Level::INFO)
                        .latency_unit(tower_http::LatencyUnit::Millis),
                ),
        )
        .layer(axum_middleware::from_fn(request_id::middleware))
}
