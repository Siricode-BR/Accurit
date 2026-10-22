use std::net::IpAddr;
use std::sync::Arc;

use axum::extract::{Request, State};
use axum::http::header;
use axum::middleware::Next;
use axum::response::Response;
use uuid::Uuid;

use crate::application::auth::{TokenError, TokenService};
use crate::domain::auth::{Audiencia, Papel};
use crate::presentation::error::ApiError;
use crate::presentation::middleware::request_id::RequestId;

#[derive(Debug, Clone, Copy)]
pub struct RequestContext {
    /// Do portal: id do usuário de `usuarios`. Do backoffice: id de `administradores`.
    pub usuario_id: Uuid,
    /// `None` no backoffice — o gestor de tenants não pertence a nenhum escritório
    /// (ADR-0010, D3); ações sobre tenants exigem `escritorio_id` explícito.
    pub escritorio_id: Option<Uuid>,
    pub papel: Papel,
    pub audiencia: Audiencia,
    pub jti: Uuid,
    pub ip: Option<IpAddr>,
}

#[derive(Clone)]
pub struct ExigirAudiencia {
    tokens: Arc<dyn TokenService>,
    audiencia: Audiencia,
}

impl ExigirAudiencia {
    pub fn novo(tokens: Arc<dyn TokenService>, audiencia: Audiencia) -> Self {
        Self { tokens, audiencia }
    }
}

pub async fn exigir(
    State(estado): State<ExigirAudiencia>,
    mut request: Request,
    next: Next,
) -> Response {
    let request_id = request
        .extensions()
        .get::<RequestId>()
        .cloned()
        .unwrap_or_else(|| RequestId::novo("desconhecido"));

    let Some(token) = token_bearer(request.headers()) else {
        return ApiError::nao_autenticado().resposta(request_id.como_str());
    };

    let resultado = if estado.audiencia == Audiencia::Admin {
        estado
            .tokens
            .validar_admin(token, estado.audiencia)
            .map(|claims| RequestContext {
                usuario_id: claims.administrador_id,
                escritorio_id: None,
                papel: Papel::Admin,
                audiencia: claims.audiencia,
                jti: claims.jti,
                ip: ip_da_requisicao(request.headers()),
            })
    } else {
        estado
            .tokens
            .validar(token, estado.audiencia)
            .map(|claims| RequestContext {
                usuario_id: claims.usuario_id,
                escritorio_id: Some(claims.escritorio_id),
                papel: claims.papel,
                audiencia: claims.audiencia,
                jti: claims.jti,
                ip: ip_da_requisicao(request.headers()),
            })
    };

    match resultado {
        Ok(contexto) => {
            request.extensions_mut().insert(contexto);
            next.run(request).await
        }
        Err(TokenError::Audiencia) => {
            ApiError::audiencia_incompativel().resposta(request_id.como_str())
        }
        Err(_) => {
            let outro_uso = estado
                .tokens
                .ler_audiencia(token)
                .is_some_and(|audiencia_token| audiencia_token != estado.audiencia);
            if outro_uso {
                ApiError::audiencia_incompativel().resposta(request_id.como_str())
            } else {
                ApiError::sessao_expirada().resposta(request_id.como_str())
            }
        }
    }
}

pub async fn exigir_extension(
    State(estado): State<ExigirAudiencia>,
    mut request: Request,
    next: Next,
) -> Response {
    let request_id = request
        .extensions()
        .get::<RequestId>()
        .cloned()
        .unwrap_or_else(|| RequestId::novo("desconhecido"));

    let Some(token) = token_bearer(request.headers()) else {
        return ApiError::nao_autenticado().resposta(request_id.como_str());
    };

    let resultado = if estado.audiencia == Audiencia::Admin {
        estado
            .tokens
            .validar_admin(token, estado.audiencia)
            .map(|claims| RequestContext {
                usuario_id: claims.administrador_id,
                escritorio_id: None,
                papel: Papel::Admin,
                audiencia: claims.audiencia,
                jti: claims.jti,
                ip: ip_da_requisicao(request.headers()),
            })
    } else {
        estado
            .tokens
            .validar(token, estado.audiencia)
            .map(|claims| RequestContext {
                usuario_id: claims.usuario_id,
                escritorio_id: Some(claims.escritorio_id),
                papel: claims.papel,
                audiencia: claims.audiencia,
                jti: claims.jti,
                ip: ip_da_requisicao(request.headers()),
            })
    };

    match resultado {
        Ok(contexto) => {
            request.extensions_mut().insert(contexto);
            next.run(request).await
        }
        Err(TokenError::Audiencia) => {
            ApiError::audiencia_incompativel().resposta(request_id.como_str())
        }
        Err(_) => {
            let outro_uso = estado
                .tokens
                .ler_audiencia(token)
                .is_some_and(|audiencia_token| audiencia_token != estado.audiencia);
            if outro_uso {
                ApiError::audiencia_incompativel().resposta(request_id.como_str())
            } else {
                ApiError::sessao_expirada().resposta(request_id.como_str())
            }
        }
    }
}

fn token_bearer(cabecalhos: &axum::http::HeaderMap) -> Option<&str> {
    let valor = cabecalhos.get(header::AUTHORIZATION)?.to_str().ok()?;
    let token = valor.strip_prefix("Bearer ").map(str::trim)?;
    if token.is_empty() { None } else { Some(token) }
}

fn ip_da_requisicao(cabecalhos: &axum::http::HeaderMap) -> Option<IpAddr> {
    cabecalhos
        .get("x-real-ip")
        .and_then(|valor| valor.to_str().ok())
        .map(str::trim)
        .and_then(|valor| valor.parse::<IpAddr>().ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn le_token_bearer() {
        let mut cabecalhos = axum::http::HeaderMap::new();
        cabecalhos.insert(
            header::AUTHORIZATION,
            HeaderValue::from_static("Bearer abc.def.ghi"),
        );
        assert_eq!(token_bearer(&cabecalhos), Some("abc.def.ghi"));
    }

    #[test]
    fn rejeita_formato_sem_bearer() {
        let mut cabecalhos = axum::http::HeaderMap::new();
        cabecalhos.insert(header::AUTHORIZATION, HeaderValue::from_static("abc"));
        assert_eq!(token_bearer(&cabecalhos), None);
    }
}
