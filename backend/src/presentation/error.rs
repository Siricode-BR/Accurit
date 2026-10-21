use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorEnvelope {
    pub error: ErrorBody,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
    pub request_id: String,
    pub details: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: String,
    details: Vec<String>,
}

impl ApiError {
    pub fn nova(status: StatusCode, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
            details: Vec::new(),
        }
    }

    pub fn com_detalhe(mut self, detalhe: impl Into<String>) -> Self {
        self.details.push(detalhe.into());
        self
    }

    pub const fn status(&self) -> StatusCode {
        self.status
    }

    pub const fn codigo(&self) -> &'static str {
        self.code
    }

    pub fn envelope(&self, request_id: &str) -> ErrorEnvelope {
        ErrorEnvelope {
            error: ErrorBody {
                code: self.code.to_owned(),
                message: self.message.clone(),
                request_id: request_id.to_owned(),
                details: self.details.clone(),
            },
        }
    }

    pub fn resposta(&self, request_id: &str) -> Response {
        (self.status, Json(self.envelope(request_id))).into_response()
    }

    pub fn rota_nao_encontrada() -> Self {
        Self::nova(
            StatusCode::NOT_FOUND,
            "ROTA_NAO_ENCONTRADA",
            "Rota não encontrada",
        )
    }

    pub fn metodo_nao_permitido() -> Self {
        Self::nova(
            StatusCode::METHOD_NOT_ALLOWED,
            "METODO_NAO_PERMITIDO",
            "Método não permitido",
        )
    }

    pub fn muitas_requisicoes() -> Self {
        Self::nova(
            StatusCode::TOO_MANY_REQUESTS,
            "MUITAS_REQUISICOES",
            "Muitas requisições; tente novamente em instantes",
        )
    }

    pub fn banco_indisponivel() -> Self {
        Self::nova(
            StatusCode::SERVICE_UNAVAILABLE,
            "BANCO_INDISPONIVEL",
            "Banco de dados indisponível",
        )
    }

    pub fn invalid_json() -> Self {
        Self::nova(
            StatusCode::BAD_REQUEST,
            "JSON_INVALIDO",
            "Corpo da requisição inválido",
        )
    }

    pub fn nao_autenticado() -> Self {
        Self::nova(
            StatusCode::UNAUTHORIZED,
            "NAO_AUTENTICADO",
            "Autenticação necessária",
        )
    }

    pub fn credenciais_invalidas() -> Self {
        Self::nova(
            StatusCode::UNAUTHORIZED,
            "AUTH_INVALID_CREDENTIALS",
            "Credenciais inválidas",
        )
    }

    pub fn sessao_expirada() -> Self {
        Self::nova(
            StatusCode::UNAUTHORIZED,
            "AUTH_SESSION_EXPIRED",
            "Sessão expirada; faça login novamente",
        )
    }

    pub fn audiencia_incompativel() -> Self {
        Self::nova(
            StatusCode::UNAUTHORIZED,
            "AUTH_AUDIENCE_MISMATCH",
            "Token não pertence a esta aplicação",
        )
    }

    pub fn csrf_ausente() -> Self {
        Self::nova(
            StatusCode::BAD_REQUEST,
            "AUTH_CSRF_HEADER_MISSING",
            "Cabeçalho de proteção CSRF ausente ou inválido",
        )
    }

    pub fn erro_interno() -> Self {
        Self::nova(
            StatusCode::INTERNAL_SERVER_ERROR,
            "ERRO_INTERNO",
            "Erro interno inesperado",
        )
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let request_id = "desconhecido";
        (self.status, Json(self.envelope(request_id))).into_response()
    }
}

impl From<axum::extract::rejection::JsonRejection> for ApiError {
    fn from(_: axum::extract::rejection::JsonRejection) -> Self {
        Self::invalid_json()
    }
}
