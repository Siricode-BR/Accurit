use std::convert::Infallible;

use axum::extract::{FromRequestParts, Request};
use axum::http::HeaderValue;
use axum::http::header::HeaderName;
use axum::http::request::Parts;
use axum::middleware::Next;
use axum::response::Response;
use uuid::Uuid;

pub const HEADER_REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");
const TAMANHO_MINIMO: usize = 8;
const TAMANHO_MAXIMO: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestId(String);

impl RequestId {
    pub fn novo(valor: impl Into<String>) -> Self {
        Self(valor.into())
    }

    pub fn como_str(&self) -> &str {
        &self.0
    }
}

impl<S> FromRequestParts<S> for RequestId
where
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(parts
            .extensions
            .get::<Self>()
            .cloned()
            .unwrap_or_else(|| Self::novo(Uuid::now_v7().to_string())))
    }
}

pub async fn middleware(mut request: Request, next: Next) -> Response {
    let recebido = request
        .headers()
        .get(&HEADER_REQUEST_ID)
        .and_then(|valor| valor.to_str().ok())
        .map(str::trim)
        .filter(|valor| valor_valido(valor))
        .map(ToOwned::to_owned);

    let request_id = recebido.unwrap_or_else(|| Uuid::now_v7().to_string());
    request
        .extensions_mut()
        .insert(RequestId::novo(request_id.clone()));

    let mut resposta = next.run(request).await;

    if let Ok(valor) = HeaderValue::from_str(&request_id) {
        resposta.headers_mut().insert(HEADER_REQUEST_ID, valor);
    }

    resposta
}

fn valor_valido(valor: &str) -> bool {
    let tamanho = valor.len();
    (TAMANHO_MINIMO..=TAMANHO_MAXIMO).contains(&tamanho)
        && valor
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aceita_ids_no_formato_esperado() {
        assert!(valor_valido("abc12345"));
        assert!(valor_valido("req_1234-ABCD_5678"));
        assert!(valor_valido(&"a".repeat(64)));
    }

    #[test]
    fn rejeita_ids_curtos_longos_ou_com_caracteres_invalidos() {
        assert!(!valor_valido("curto"));
        assert!(!valor_valido(&"a".repeat(65)));
        assert!(!valor_valido("id com espaco"));
        assert!(!valor_valido("id\ninjection"));
        assert!(!valor_valido("id/../etc"));
    }
}
