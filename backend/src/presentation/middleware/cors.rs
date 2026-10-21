use axum::http::{HeaderName, HeaderValue, Method, header};
use tower_http::cors::CorsLayer;

pub fn camada(origens: &[String]) -> CorsLayer {
    let mut camada = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::ACCEPT,
            header::COOKIE,
            HeaderName::from_static("x-request-id"),
            HeaderName::from_static("x-requested-with"),
        ])
        .allow_credentials(true);

    let valores: Vec<HeaderValue> = origens
        .iter()
        .filter_map(|origem| HeaderValue::from_str(origem).ok())
        .collect();

    if !valores.is_empty() {
        camada = camada.allow_origin(valores);
    }

    camada
}
