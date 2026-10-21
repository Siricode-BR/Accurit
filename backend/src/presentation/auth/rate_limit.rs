use std::num::NonZeroU32;
use std::sync::Arc;

use axum::body::{Body, to_bytes};
use axum::extract::{Request, State};
use axum::http::{HeaderMap, HeaderValue, header};
use axum::middleware::Next;
use axum::response::Response;
use governor::clock::DefaultClock;
use governor::state::keyed::DefaultKeyedStateStore;
use governor::{Quota, RateLimiter};
use serde_json::Value;

use crate::presentation::error::ApiError;
use crate::presentation::middleware::request_id::RequestId;

const REQUISICOES_POR_MINUTO: u32 = 10;
const BURST_LOGIN: u32 = 5;
const RETRY_AFTER: &str = "60";
const LIMITE_CORPO_BYTES: usize = 64 * 1024;

type Limitador = RateLimiter<String, DefaultKeyedStateStore<String>, DefaultClock>;

#[derive(Clone)]
pub struct LimiteLogin {
    limitador: Arc<Limitador>,
}

impl LimiteLogin {
    pub fn novo() -> Self {
        let por_minuto = NonZeroU32::new(REQUISICOES_POR_MINUTO).expect("valor positivo");
        let burst = NonZeroU32::new(BURST_LOGIN).expect("valor positivo");
        Self {
            limitador: Arc::new(RateLimiter::dashmap(
                Quota::per_minute(por_minuto).allow_burst(burst),
            )),
        }
    }

    pub fn verificar(&self, chave: &str) -> bool {
        self.limitador.check_key(&chave.to_owned()).is_ok()
    }
}

pub async fn middleware(
    State(limite): State<LimiteLogin>,
    request: Request,
    next: Next,
) -> Response {
    let (partes, corpo) = request.into_parts();
    let bytes = to_bytes(corpo, LIMITE_CORPO_BYTES)
        .await
        .unwrap_or_default();

    let slug = serde_json::from_slice::<Value>(&bytes)
        .ok()
        .and_then(|valor| {
            valor
                .get("escritorio")
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
        .unwrap_or_else(|| "-".to_owned());

    let ip = ip_da_requisicao(&partes.headers);
    let caminho = partes.uri.path().to_owned();
    let chave = format!("{ip}|{caminho}|{}", slug.to_ascii_lowercase());

    let request_id = partes
        .extensions
        .get::<RequestId>()
        .cloned()
        .unwrap_or_else(|| RequestId::novo("desconhecido"));

    if !limite.verificar(&chave) {
        tracing::warn!(
            request_id = request_id.como_str(),
            caminho = %caminho,
            "limite de login excedido"
        );
        let mut resposta = ApiError::muitas_requisicoes().resposta(request_id.como_str());
        resposta
            .headers_mut()
            .insert(header::RETRY_AFTER, HeaderValue::from_static(RETRY_AFTER));
        return resposta;
    }

    let request = Request::from_parts(partes, Body::from(bytes));
    next.run(request).await
}

fn ip_da_requisicao(cabecalhos: &HeaderMap) -> String {
    cabecalhos
        .get("x-real-ip")
        .and_then(|valor| valor.to_str().ok())
        .map(str::trim)
        .filter(|valor| !valor.is_empty() && valor.len() <= 64)
        .unwrap_or("desconhecido")
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bloqueia_apos_burst_por_chave() {
        let limite = LimiteLogin::novo();
        for _ in 0..BURST_LOGIN {
            assert!(limite.verificar("ip|rota|slug"));
        }
        assert!(!limite.verificar("ip|rota|slug"));
        assert!(limite.verificar("outro-ip|rota|slug"));
    }
}
