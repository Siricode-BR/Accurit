use std::env;

use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};

async fn health() -> impl IntoResponse {
    (
        StatusCode::OK,
        [("content-type", "application/json")],
        r#"{"status":"ok","service":"accurit-backend"}"#,
    )
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port = env::var("PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(8080);

    let app = Router::new().route("/api/v1/health", get(health));

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    println!("accurit-backend listening on 0.0.0.0:{port}");

    axum::serve(listener, app).await?;

    Ok(())
}
