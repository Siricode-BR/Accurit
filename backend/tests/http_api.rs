mod common;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use axum::response::Response;
use serde_json::Value;
use tower::ServiceExt;
use uuid::Uuid;

use accurit_backend::application::ports::{SaudeBanco, SaudeBancoError};
use accurit_backend::application::saude::ConsultarSaude;
use accurit_backend::infrastructure::config::Config;
use accurit_backend::infrastructure::persistence::pool::DbPool;
use accurit_backend::presentation::routes;
use accurit_backend::presentation::state::AppState;

use common::{configuracao, pool_do_banco};

struct BancoFixo {
    disponivel: bool,
}

impl SaudeBanco for BancoFixo {
    async fn checar(&self) -> Result<(), SaudeBancoError> {
        if self.disponivel {
            Ok(())
        } else {
            Err(SaudeBancoError)
        }
    }
}

fn app(disponivel: bool, config: &Config, pool: DbPool) -> Router {
    routes::construir(
        AppState::novo(ConsultarSaude::novo(BancoFixo { disponivel }), pool),
        None,
        config,
    )
}

fn pedido(metodo: Method, caminho: &str) -> Request<Body> {
    Request::builder()
        .method(metodo)
        .uri(caminho)
        .body(Body::empty())
        .expect("requisição válida")
}

async fn corpo_json(resposta: Response) -> Value {
    let bytes = to_bytes(resposta.into_body(), 1024 * 1024)
        .await
        .expect("corpo deve ser lido");
    serde_json::from_slice(&bytes).expect("corpo deve ser JSON")
}

fn request_id(resposta: &Response) -> String {
    resposta
        .headers()
        .get("x-request-id")
        .expect("x-request-id deve estar presente")
        .to_str()
        .expect("x-request-id deve ser ASCII")
        .to_owned()
}

#[tokio::test]
async fn health_retorna_ok_e_request_id_gerado() {
    let banco = common::BancoTemporario::novo();
    let pool = pool_do_banco(&banco);
    let config = configuracao("");
    let resposta = app(true, &config, pool)
        .oneshot(pedido(Method::GET, "/api/v1/health"))
        .await
        .expect("router deve responder");

    assert_eq!(resposta.status(), StatusCode::OK);
    let id = request_id(&resposta);
    assert!(Uuid::parse_str(&id).is_ok(), "id gerado deve ser UUID");
    assert_eq!(
        resposta.headers().get(header::CACHE_CONTROL).unwrap(),
        "no-store"
    );
    assert_eq!(
        resposta
            .headers()
            .get(header::X_CONTENT_TYPE_OPTIONS)
            .unwrap(),
        "nosniff"
    );

    let corpo = corpo_json(resposta).await;
    assert_eq!(corpo["status"], "ok");
    assert_eq!(corpo["service"], "accurit-backend");
    assert_eq!(corpo["db"], "ok");
}

#[tokio::test]
async fn health_preserva_request_id_valido_e_substitui_invalido() {
    let banco = common::BancoTemporario::novo();
    let pool = pool_do_banco(&banco);
    let config = configuracao("");
    let app = app(true, &config, pool);

    let mut requisicao = pedido(Method::GET, "/api/v1/health");
    requisicao
        .headers_mut()
        .insert("x-request-id", "req-12345678".parse().unwrap());
    let resposta = app
        .clone()
        .oneshot(requisicao)
        .await
        .expect("router deve responder");
    assert_eq!(request_id(&resposta), "req-12345678");

    let mut requisicao = pedido(Method::GET, "/api/v1/health");
    requisicao
        .headers_mut()
        .insert("x-request-id", "curto".parse().unwrap());
    let resposta = app
        .oneshot(requisicao)
        .await
        .expect("router deve responder");
    assert_ne!(request_id(&resposta), "curto");
}

#[tokio::test]
async fn health_retorna_503_com_envelope_quando_banco_falha() {
    let banco = common::BancoTemporario::novo();
    let pool = pool_do_banco(&banco);
    let config = configuracao("");
    let resposta = app(false, &config, pool)
        .oneshot(pedido(Method::GET, "/api/v1/health"))
        .await
        .expect("router deve responder");

    assert_eq!(resposta.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(
        resposta.headers().get(header::CACHE_CONTROL).unwrap(),
        "no-store"
    );
    let id = request_id(&resposta);

    let corpo = corpo_json(resposta).await;
    assert_eq!(corpo["error"]["code"], "BANCO_INDISPONIVEL");
    assert_eq!(corpo["error"]["requestId"], id);
    assert!(corpo["error"]["details"].is_array());
    assert_eq!(corpo["status"], "unavailable");
    assert_eq!(corpo["db"], "unavailable");
}

#[tokio::test]
async fn rotas_inexistentes_e_metodos_invalidos_retornam_envelope() {
    let banco = common::BancoTemporario::novo();
    let pool = pool_do_banco(&banco);
    let config = configuracao("");
    let app = app(true, &config, pool);

    let resposta = app
        .clone()
        .oneshot(pedido(Method::GET, "/api/v1/nao-existe"))
        .await
        .expect("router deve responder");
    assert_eq!(resposta.status(), StatusCode::NOT_FOUND);
    let id = request_id(&resposta);
    let corpo = corpo_json(resposta).await;
    assert_eq!(corpo["error"]["code"], "ROTA_NAO_ENCONTRADA");
    assert_eq!(corpo["error"]["requestId"], id);

    let resposta = app
        .clone()
        .oneshot(pedido(Method::POST, "/api/v1/health"))
        .await
        .expect("router deve responder");
    assert_eq!(resposta.status(), StatusCode::METHOD_NOT_ALLOWED);
    let corpo = corpo_json(resposta).await;
    assert_eq!(corpo["error"]["code"], "METODO_NAO_PERMITIDO");

    let resposta = app
        .oneshot(pedido(Method::GET, "/fora-da-api"))
        .await
        .expect("router deve responder");
    assert_eq!(resposta.status(), StatusCode::NOT_FOUND);
    let corpo = corpo_json(resposta).await;
    assert_eq!(corpo["error"]["code"], "ROTA_NAO_ENCONTRADA");
}

#[tokio::test]
async fn cors_so_permite_origens_configuradas() {
    let banco = common::BancoTemporario::novo();
    let pool = pool_do_banco(&banco);
    let config = configuracao("http://localhost:4200");
    let app = app(true, &config, pool);

    let mut requisicao = pedido(Method::GET, "/api/v1/health");
    requisicao
        .headers_mut()
        .insert(header::ORIGIN, "http://localhost:4200".parse().unwrap());
    let resposta = app
        .clone()
        .oneshot(requisicao)
        .await
        .expect("router deve responder");
    assert_eq!(
        resposta
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .unwrap(),
        "http://localhost:4200"
    );

    let mut requisicao = pedido(Method::GET, "/api/v1/health");
    requisicao
        .headers_mut()
        .insert(header::ORIGIN, "http://malicioso.test".parse().unwrap());
    let resposta = app
        .oneshot(requisicao)
        .await
        .expect("router deve responder");
    assert!(
        resposta
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .is_none()
    );
}

#[tokio::test]
async fn limite_de_requisicoes_retorna_429_e_health_e_isento() {
    let banco = common::BancoTemporario::novo();
    let pool = pool_do_banco(&banco);
    let config = configuracao("");
    let app = app(true, &config, pool);

    let mut status_final = StatusCode::OK;
    for _ in 0..61 {
        let mut requisicao = pedido(Method::GET, "/api/v1/nao-existe");
        requisicao
            .headers_mut()
            .insert("x-real-ip", "198.51.100.7".parse().unwrap());
        let resposta = app
            .clone()
            .oneshot(requisicao)
            .await
            .expect("router deve responder");
        status_final = resposta.status();
    }

    assert_eq!(status_final, StatusCode::TOO_MANY_REQUESTS);

    let mut requisicao = pedido(Method::GET, "/api/v1/nao-existe");
    requisicao
        .headers_mut()
        .insert("x-real-ip", "198.51.100.7".parse().unwrap());
    let resposta = app
        .clone()
        .oneshot(requisicao)
        .await
        .expect("router deve responder");
    assert_eq!(resposta.status(), StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(resposta.headers().get(header::RETRY_AFTER).unwrap(), "1");
    let corpo = corpo_json(resposta).await;
    assert_eq!(corpo["error"]["code"], "MUITAS_REQUISICOES");

    let mut requisicao = pedido(Method::GET, "/api/v1/health");
    requisicao
        .headers_mut()
        .insert("x-real-ip", "198.51.100.7".parse().unwrap());
    let resposta = app
        .oneshot(requisicao)
        .await
        .expect("router deve responder");
    assert_eq!(resposta.status(), StatusCode::OK);
}
