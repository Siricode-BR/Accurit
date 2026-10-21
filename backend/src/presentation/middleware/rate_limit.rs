use std::num::NonZeroU32;
use std::sync::Arc;

use axum::extract::{Request, State};
use axum::http::{HeaderMap, HeaderValue, header};
use axum::middleware::Next;
use axum::response::Response;
use governor::clock::DefaultClock;
use governor::state::keyed::DefaultKeyedStateStore;
use governor::{Quota, RateLimiter};

use crate::presentation::error::ApiError;
use crate::presentation::middleware::request_id::RequestId;

const CAMINHOS_HEALTH: [&str; 2] = ["/api/v1/health", "/health"];
const RETRY_AFTER_SEGUNDOS: &str = "1";

type Limitador = RateLimiter<String, DefaultKeyedStateStore<String>, DefaultClock>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RateLimitKey {
    Ip(String),
}

impl RateLimitKey {
    pub fn da_requisicao(cabecalhos: &HeaderMap) -> Self {
        let ip = cabecalhos
            .get("x-real-ip")
            .and_then(|valor| valor.to_str().ok())
            .map(str::trim)
            .filter(|valor| !valor.is_empty() && valor.len() <= 64)
            .unwrap_or("desconhecido");

        Self::Ip(ip.to_owned())
    }

    pub fn como_str(&self) -> &str {
        match self {
            Self::Ip(valor) => valor,
        }
    }
}

#[derive(Clone)]
pub struct LimiteRequisicoes {
    limitador: Arc<Limitador>,
}

impl LimiteRequisicoes {
    pub fn novo(por_segundo: u32, burst: u32) -> Self {
        let por_segundo = NonZeroU32::new(por_segundo.max(1)).expect("valor máximo de 1");
        let burst = NonZeroU32::new(burst.max(1)).expect("valor máximo de 1");
        let quota = Quota::per_second(por_segundo).allow_burst(burst);

        Self {
            limitador: Arc::new(RateLimiter::dashmap(quota)),
        }
    }

    pub fn verificar(&self, chave: &RateLimitKey) -> bool {
        self.limitador
            .check_key(&chave.como_str().to_owned())
            .is_ok()
    }
}

pub async fn middleware(
    State(limite): State<LimiteRequisicoes>,
    request: Request,
    next: Next,
) -> Response {
    if CAMINHOS_HEALTH.contains(&request.uri().path()) {
        return next.run(request).await;
    }

    let chave = RateLimitKey::da_requisicao(request.headers());

    if limite.verificar(&chave) {
        return next.run(request).await;
    }

    let request_id = request
        .extensions()
        .get::<RequestId>()
        .cloned()
        .unwrap_or_else(|| RequestId::novo("desconhecido"));

    tracing::warn!(
        request_id = request_id.como_str(),
        caminho = request.uri().path(),
        "limite de requisições excedido"
    );

    let mut resposta = ApiError::muitas_requisicoes().resposta(request_id.como_str());
    resposta.headers_mut().insert(
        header::RETRY_AFTER,
        HeaderValue::from_static(RETRY_AFTER_SEGUNDOS),
    );
    resposta
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn usa_x_real_ip_como_chave() {
        let mut cabecalhos = HeaderMap::new();
        cabecalhos.insert("x-real-ip", HeaderValue::from_static("203.0.113.10"));
        assert_eq!(
            RateLimitKey::da_requisicao(&cabecalhos),
            RateLimitKey::Ip("203.0.113.10".to_owned())
        );
    }

    #[test]
    fn usa_chave_desconhecida_sem_x_real_ip() {
        assert_eq!(
            RateLimitKey::da_requisicao(&HeaderMap::new()),
            RateLimitKey::Ip("desconhecido".to_owned())
        );
    }

    #[test]
    fn ignora_x_real_ip_muito_longo() {
        let mut cabecalhos = HeaderMap::new();
        let valor = HeaderValue::from_str(&"a".repeat(65)).expect("valor válido");
        cabecalhos.insert("x-real-ip", valor);
        assert_eq!(
            RateLimitKey::da_requisicao(&cabecalhos),
            RateLimitKey::Ip("desconhecido".to_owned())
        );
    }

    #[test]
    fn bloqueia_apos_esgotar_o_burst() {
        let limite = LimiteRequisicoes::novo(1, 1);
        let chave = RateLimitKey::Ip("203.0.113.10".to_owned());
        assert!(limite.verificar(&chave));
        assert!(!limite.verificar(&chave));
    }
}
