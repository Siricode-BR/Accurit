use axum::Json;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::Serialize;

use crate::application::admin::ProvisionarError;
use crate::application::clientes::ClienteError;
use crate::application::usuarios::{RepositorioError as UsuarioRepoError, UsuarioError};

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
}

#[derive(Debug, Clone)]
pub struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: String,
    retry_after: Option<HeaderValue>,
}

impl ApiError {
    pub fn nova(status: StatusCode, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
            retry_after: None,
        }
    }

    pub fn com_retry_after(mut self, segundos: &'static str) -> Self {
        self.retry_after = Some(HeaderValue::from_static(segundos));
        self
    }

    /// Detalhe técnico **nunca** vai para a resposta: fica apenas no log.
    pub fn com_detalhe(self, detalhe: impl Into<String>) -> Self {
        tracing::debug!(detalhe = %detalhe.into(), "detalhe interno de erro");
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
            },
        }
    }

    pub fn resposta(&self, request_id: &str) -> Response {
        let mut response = (self.status, Json(self.envelope(request_id))).into_response();
        if let Some(retry_after) = &self.retry_after {
            response
                .headers_mut()
                .insert(header::RETRY_AFTER, retry_after.clone());
        }
        response
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
        let request_id = crate::presentation::middleware::request_id::atual()
            .unwrap_or_else(|| "desconhecido".to_owned());
        self.resposta(&request_id)
    }
}

impl From<axum::extract::rejection::JsonRejection> for ApiError {
    fn from(_: axum::extract::rejection::JsonRejection) -> Self {
        Self::invalid_json()
    }
}

impl From<axum::extract::rejection::PathRejection> for ApiError {
    fn from(_: axum::extract::rejection::PathRejection) -> Self {
        Self::validacao("Parâmetro de rota inválido")
    }
}

impl From<axum::extract::rejection::QueryRejection> for ApiError {
    fn from(_: axum::extract::rejection::QueryRejection) -> Self {
        Self::validacao("Parâmetros de consulta inválidos")
    }
}

impl From<ProvisionarError> for ApiError {
    fn from(erro: ProvisionarError) -> Self {
        match erro {
            ProvisionarError::Repositorio(e) => match e {
                crate::application::admin::RepositorioError::Indisponivel => Self::indisponivel(),
                crate::application::admin::RepositorioError::Dados => Self::erro_interno(),
                crate::application::admin::RepositorioError::EscritorioExistente => {
                    Self::conflito("Escritório com este CNPJ já existe")
                }
                crate::application::admin::RepositorioError::UsuarioExistente => {
                    Self::conflito("Já existe um usuário com este e-mail")
                }
                crate::application::admin::RepositorioError::AutoDesativacao
                | crate::application::admin::RepositorioError::UltimoAdministradorAtivo => {
                    Self::conflito("Operação não permitida para este administrador")
                }
                crate::application::admin::RepositorioError::Validacao(msg) => Self::validacao(msg),
            },
            ProvisionarError::Validacao(msg) => {
                if msg.contains("CNPJ") {
                    Self::nova(StatusCode::UNPROCESSABLE_ENTITY, "VALIDACAO", msg)
                } else {
                    Self::validacao(msg)
                }
            }
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
            ClienteError::FuncaoSemEscrita => Self::proibido(),
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
            crate::application::certificados::CertificadoError::TitularDivergente => Self::nova(
                StatusCode::UNPROCESSABLE_ENTITY,
                "VALIDACAO",
                "O CNPJ do certificado A1 não corresponde ao CNPJ deste escritório",
            ),
            crate::application::certificados::CertificadoError::Fingerprint(e) => {
                Self::validacao(e.to_string())
            }
        }
    }
}

impl From<UsuarioError> for ApiError {
    fn from(erro: UsuarioError) -> Self {
        match erro {
            UsuarioError::Validacao(msg) => Self::validacao(msg),
            UsuarioError::NaoEncontrado => Self::nao_encontrado("usuário"),
            UsuarioError::EscritorioNaoEncontrado => Self::nao_encontrado("escritório"),
            UsuarioError::Conflito(msg) => Self::conflito(msg),
            UsuarioError::Senha => Self::erro_interno(),
            UsuarioError::Repositorio(e) => match e {
                UsuarioRepoError::Indisponivel => Self::indisponivel(),
                UsuarioRepoError::Dados => Self::erro_interno(),
                UsuarioRepoError::EmailExistente => {
                    Self::conflito("Já existe um usuário com este e-mail")
                }
                UsuarioRepoError::CpfExistente => {
                    Self::conflito("CPF já cadastrado neste escritório")
                }
                UsuarioRepoError::EscritorioInexistente => Self::nao_encontrado("escritório"),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn envelope_de_erro_nao_expoe_detalhes_nem_request_id_tecnico() {
        let erro = ApiError::erro_interno();
        let json = serde_json::to_string(&erro.envelope("req-123")).unwrap();
        assert!(json.contains("\"code\":\"ERRO_INTERNO\""));
        assert!(json.contains("\"requestId\":\"req-123\""));
        assert!(json.contains("Erro interno inesperado"));
        assert!(!json.contains("details"));
        for proibido in [
            "diesel",
            "postgres",
            "SELECT",
            "constraint",
            "panicked",
            ".rs",
            "localhost",
            "reqwest",
            "gateway.apiserpro",
        ] {
            assert!(!json.to_lowercase().contains(proibido));
        }
    }

    #[test]
    fn rejeicoes_de_extratores_viram_envelope_amigavel() {
        let erro = ApiError::validacao("Par�metro de rota inv�lido");
        assert_eq!(erro.status(), StatusCode::BAD_REQUEST);
        let json = serde_json::to_string(&erro.envelope("req-1")).unwrap();
        assert!(json.contains("VALIDACAO"));
        assert!(!json.contains("details"));
    }
}
