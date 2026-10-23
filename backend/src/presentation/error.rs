use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

use crate::application::admin::ProvisionarError;
use crate::application::clientes::ClienteError;

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

    pub fn proibido() -> Self {
        Self::nova(StatusCode::FORBIDDEN, "PROIBIDO", "Acesso negado")
    }

    pub fn indisponivel() -> Self {
        Self::nova(
            StatusCode::SERVICE_UNAVAILABLE,
            "INDISPONIVEL",
            "Serviço indisponível",
        )
    }

    pub fn conflito(message: impl Into<String>) -> Self {
        Self::nova(StatusCode::CONFLICT, "CONFLITO", message)
    }

    pub fn validacao(message: impl Into<String>) -> Self {
        Self::nova(StatusCode::BAD_REQUEST, "VALIDACAO", message)
    }

    pub fn nao_encontrado(entidade: &str) -> Self {
        Self::nova(
            StatusCode::NOT_FOUND,
            "NAO_ENCONTRADO",
            format!("{} não encontrado", entidade),
        )
    }

    pub fn filtro_invalido() -> Self {
        Self::nova(
            StatusCode::BAD_REQUEST,
            "FILTRO_INVALIDO",
            "Filtro inválido",
        )
    }

    pub fn paginacao_invalida() -> Self {
        Self::nova(
            StatusCode::BAD_REQUEST,
            "PAGINACAO_INVALIDA",
            "Paginação inválida",
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

impl From<ProvisionarError> for ApiError {
    fn from(erro: ProvisionarError) -> Self {
        match erro {
            ProvisionarError::Repositorio(e) => match e {
                crate::application::admin::RepositorioError::Indisponivel => Self::indisponivel(),
                crate::application::admin::RepositorioError::Dados => Self::erro_interno(),
                crate::application::admin::RepositorioError::EscritorioExistente => {
                    Self::conflito("Escritório com este slug já existe")
                }
                crate::application::admin::RepositorioError::UsuarioExistente => {
                    Self::conflito("Usuário com este e-mail já existe neste escritório")
                }
            },
            ProvisionarError::Validacao(msg) => Self::validacao(msg),
            ProvisionarError::Senha => Self::erro_interno(),
        }
    }
}

impl From<ClienteError> for ApiError {
    fn from(erro: ClienteError) -> Self {
        match erro {
            ClienteError::Dominio(e) => Self::validacao(e.to_string()),
            ClienteError::CpfCnpj(e) => Self::validacao(e.to_string()),
            ClienteError::Repositorio(e) => match e {
                crate::application::clientes::RepositorioError::Indisponivel => {
                    Self::indisponivel()
                }
                crate::application::clientes::RepositorioError::Dados => Self::erro_interno(),
                crate::application::clientes::RepositorioError::NaoEncontrado => {
                    Self::nao_encontrado("cliente")
                }
                crate::application::clientes::RepositorioError::Duplicado => {
                    Self::conflito("CPF/CNPJ já cadastrado neste escritório")
                }
            },
            ClienteError::Validacao(msg) => Self::validacao(msg),
            ClienteError::AcessoNegado => Self::proibido(),
        }
    }
}

impl From<crate::application::certificados::CertificadoError> for ApiError {
    fn from(erro: crate::application::certificados::CertificadoError) -> Self {
        match erro {
            crate::application::certificados::CertificadoError::Dominio(e) => {
                Self::validacao(e.to_string())
            }
            crate::application::certificados::CertificadoError::Repositorio(e) => match e {
                crate::application::certificados::RepositorioError::Indisponivel => {
                    Self::indisponivel()
                }
                crate::application::certificados::RepositorioError::Dados => Self::erro_interno(),
                crate::application::certificados::RepositorioError::NaoEncontrado => {
                    Self::nao_encontrado("certificado")
                }
                crate::application::certificados::RepositorioError::FingerprintDuplicado => {
                    Self::conflito("Certificado com este fingerprint já existe")
                }
            },
            crate::application::certificados::CertificadoError::Criptografia => {
                Self::erro_interno()
            }
            crate::application::certificados::CertificadoError::AcessoNegado => Self::proibido(),
            crate::application::certificados::CertificadoError::Fingerprint(e) => {
                Self::validacao(e.to_string())
            }
        }
    }
}
