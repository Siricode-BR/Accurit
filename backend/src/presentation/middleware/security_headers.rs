use axum::extract::Request;
use axum::http::HeaderValue;
use axum::http::header;
use axum::middleware::Next;
use axum::response::Response;

pub async fn middleware(request: Request, next: Next) -> Response {
    let mut resposta = next.run(request).await;
    let cabecalhos = resposta.headers_mut();
    cabecalhos.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    cabecalhos.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    resposta
}
