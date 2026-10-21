use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

use crate::application::ports::SaudeBanco;
use crate::application::saude::ConsultarSaude;
use crate::presentation::dto::SaudeResponse;
use crate::presentation::error::{ApiError, ErrorBody};
use crate::presentation::middleware::request_id::RequestId;
use crate::presentation::state::AppState;

#[derive(Debug, Serialize)]
struct SaudeIndisponivelResponse {
    error: ErrorBody,
    status: &'static str,
    service: &'static str,
    db: &'static str,
}

pub async fn saude<S>(State(state): State<AppState<S>>, request_id: RequestId) -> Response
where
    S: SaudeBanco,
{
    let consultar: &ConsultarSaude<S> = state.consultar_saude.as_ref();

    match consultar.executar().await {
        Ok(()) => (StatusCode::OK, Json(SaudeResponse::ok())).into_response(),
        Err(_) => {
            tracing::warn!(
                request_id = request_id.como_str(),
                "health check indisponível"
            );

            let erro = ApiError::banco_indisponivel();
            let corpo = SaudeIndisponivelResponse {
                error: erro.envelope(request_id.como_str()).error,
                status: "unavailable",
                service: "accurit-backend",
                db: "unavailable",
            };

            (StatusCode::SERVICE_UNAVAILABLE, Json(corpo)).into_response()
        }
    }
}
