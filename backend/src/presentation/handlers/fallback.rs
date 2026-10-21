use axum::response::Response;

use crate::presentation::error::ApiError;
use crate::presentation::middleware::request_id::RequestId;

pub async fn nao_encontrado(request_id: RequestId) -> Response {
    ApiError::rota_nao_encontrada().resposta(request_id.como_str())
}

pub async fn metodo_nao_permitido(request_id: RequestId) -> Response {
    ApiError::metodo_nao_permitido().resposta(request_id.como_str())
}
