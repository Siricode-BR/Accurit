mod common;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use axum::response::Response;
use diesel::QueryableByName;
use diesel::RunQueryDsl;
use diesel::sql_query;
use diesel::sql_types::{BigInt, Nullable, Text, Uuid as SqlUuid};
use serde_json::{Value, json};
use tower::ServiceExt;
use uuid::Uuid;

use accurit_backend::application::ports::{SaudeBanco, SaudeBancoError};
use accurit_backend::application::saude::ConsultarSaude;
use accurit_backend::infrastructure::auth::HashArgon2;
use accurit_backend::infrastructure::persistence::migrations::{
    MIGRATIONS, aplicar_migrations_com_lock,
};
use accurit_backend::infrastructure::persistence::pool::criar_pool;
use accurit_backend::presentation::auth::AuthState;
use accurit_backend::presentation::routes;
use accurit_backend::presentation::state::AppState;

use common::{BancoTemporario, configuracao_com_extras};

const SENHA: &str = "senha-forte-123";
const SLUG: &str = "alfa";

struct BancoFixo;

impl SaudeBanco for BancoFixo {
    async fn checar(&self) -> Result<(), SaudeBancoError> {
        Ok(())
    }
}

struct Ambiente {
    app: Router,
    banco: BancoTemporario,
}

fn ambiente(extras: &[(&str, &str)]) -> Ambiente {
    let banco = BancoTemporario::novo();
    aplicar_migrations_com_lock(banco.conexao(), MIGRATIONS).expect("migrations devem aplicar");

    let config = configuracao_com_extras("", extras);
    let pool = criar_pool(&banco.database_url).expect("pool deve ser criado");
    let auth = AuthState::novo(pool, &config).expect("auth deve inicializar");
    let estado = AppState::novo(ConsultarSaude::novo(BancoFixo));
    let app = routes::construir(estado, Some(auth), &config);

    Ambiente { app, banco }
}

fn semear(banco: &BancoTemporario, papel: &str, ativo: bool, cpf: Option<&str>) {
    let config = configuracao_com_extras("", &[]);
    let argon2 = HashArgon2::novo(&config.auth).expect("argon2 deve inicializar");
    let hash = argon2.gerar_hash(SENHA).expect("hash deve ser gerado");

    let mut conexao = banco.conexao();
    let escritorio_id = Uuid::now_v7();
    sql_query("INSERT INTO escritorios (id, slug, nome) VALUES ($1, $2, $3)")
        .bind::<SqlUuid, _>(escritorio_id)
        .bind::<Text, _>(SLUG)
        .bind::<Text, _>("Escritório de teste")
        .execute(&mut conexao)
        .expect("escritório deve ser inserido");

    sql_query(
        "INSERT INTO usuarios (id, escritorio_id, email, cpf, senha_hash, papel, ativo) \
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind::<SqlUuid, _>(Uuid::now_v7())
    .bind::<SqlUuid, _>(escritorio_id)
    .bind::<Text, _>(format!("{papel}@alfa.test").to_lowercase())
    .bind::<Nullable<Text>, _>(cpf)
    .bind::<Text, _>(hash)
    .bind::<Text, _>(papel)
    .bind::<diesel::sql_types::Bool, _>(ativo)
    .execute(&mut conexao)
    .expect("usuário deve ser inserido");
}

fn pedido(
    metodo: Method,
    caminho: &str,
    corpo: Option<Value>,
    extras: &[(&str, &str)],
) -> Request<Body> {
    let (corpo, tem_corpo) = match corpo {
        Some(valor) => (Body::from(valor.to_string()), true),
        None => (Body::empty(), false),
    };

    let mut construtor = Request::builder().method(metodo).uri(caminho);
    if tem_corpo {
        construtor = construtor.header(header::CONTENT_TYPE, "application/json");
    }
    for (chave, valor) in extras {
        construtor = construtor.header(*chave, *valor);
    }
    construtor.body(corpo).expect("requisição válida")
}

async fn enviar(app: &Router, pedido: Request<Body>) -> Response {
    app.clone()
        .oneshot(pedido)
        .await
        .expect("resposta deve existir")
}

async fn corpo_json(resposta: Response) -> Value {
    let bytes = to_bytes(resposta.into_body(), 1024 * 1024)
        .await
        .expect("corpo deve ser lido");
    serde_json::from_slice(&bytes).expect("corpo deve ser JSON")
}

fn cookie_da_resposta(resposta: &Response, nome: &str) -> Option<String> {
    let cabecalhos = resposta.headers().get_all(header::SET_COOKIE);
    for cabecalho in cabecalhos {
        let valor = cabecalho.to_str().ok()?;
        let Some(par) = valor.split(';').next() else {
            continue;
        };
        let Some((chave, token)) = par.split_once('=') else {
            continue;
        };
        if chave == nome {
            return Some(token.to_owned());
        }
    }
    None
}

fn cabecalho_cookie(resposta: &Response, nome: &str) -> Option<String> {
    resposta
        .headers()
        .get_all(header::SET_COOKIE)
        .into_iter()
        .filter_map(|cabecalho| cabecalho.to_str().ok())
        .find(|valor| valor.starts_with(&format!("{nome}=")))
        .map(str::to_owned)
}

fn login(audiencia: &str, senha: &str, papel: &str) -> Request<Body> {
    pedido(
        Method::POST,
        &format!("/api/v1/auth/{audiencia}/login"),
        Some(json!({
            "escritorio": SLUG,
            "email": format!("{papel}@alfa.test").to_lowercase(),
            "senha": senha,
        })),
        &[],
    )
}

#[derive(QueryableByName)]
struct Contagem {
    #[diesel(sql_type = BigInt)]
    total: i64,
}

fn contar(banco: &BancoTemporario, consulta: &str) -> i64 {
    sql_query(consulta)
        .get_result::<Contagem>(&mut banco.conexao())
        .expect("consulta deve funcionar")
        .total
}

#[tokio::test]
async fn login_admin_emite_token_e_cookie_seguro() {
    let ambiente = ambiente(&[]);
    semear(&ambiente.banco, "ADMIN", true, Some("12345678901"));

    let resposta = enviar(&ambiente.app, login("admin", SENHA, "ADMIN")).await;
    assert_eq!(resposta.status(), StatusCode::OK);
    assert_eq!(
        resposta
            .headers()
            .get(header::CACHE_CONTROL)
            .and_then(|valor| valor.to_str().ok()),
        Some("no-store")
    );

    let cabecalho = cabecalho_cookie(&resposta, "accurit_admin_rt").expect("cookie deve existir");
    assert!(cabecalho.contains("HttpOnly"));
    assert!(cabecalho.contains("SameSite=Strict"));
    assert!(cabecalho.contains("Path=/api/v1/auth/admin"));
    assert!(cabecalho.contains("Max-Age=28800"));
    assert!(cookie_da_resposta(&resposta, "accurit_client_rt").is_none());

    let corpo = corpo_json(resposta).await;
    assert_eq!(corpo["usuario"]["papel"], "ADMIN");
    assert!(corpo["accessToken"].as_str().is_some_and(|v| !v.is_empty()));
    assert_eq!(corpo["expiresIn"], 900);
    assert!(corpo["refreshToken"].is_null());
}

#[tokio::test]
async fn login_com_senha_errada_audita_falha_sem_cookie() {
    let ambiente = ambiente(&[]);
    semear(&ambiente.banco, "ADMIN", true, Some("12345678901"));

    let resposta = enviar(&ambiente.app, login("admin", "senha-errada", "ADMIN")).await;
    assert_eq!(resposta.status(), StatusCode::UNAUTHORIZED);
    assert!(cabecalho_cookie(&resposta, "accurit_admin_rt").is_none());

    let corpo = corpo_json(resposta).await;
    assert_eq!(corpo["error"]["code"], "AUTH_INVALID_CREDENTIALS");

    assert_eq!(
        contar(
            &ambiente.banco,
            "SELECT COUNT(*) AS total FROM auditoria_eventos WHERE acao = 'AUTH_LOGIN_FALHA'"
        ),
        1
    );
}

#[tokio::test]
async fn token_de_uma_audiencia_nao_vale_na_outra() {
    let ambiente = ambiente(&[]);
    semear(&ambiente.banco, "ADMIN", true, Some("12345678901"));

    let resposta = enviar(&ambiente.app, login("admin", SENHA, "ADMIN")).await;
    let corpo = corpo_json(resposta).await;
    let token = corpo["accessToken"].as_str().expect("token").to_owned();

    let autorizacao_header = format!("Bearer {token}");
    let autorizacao = [("authorization", autorizacao_header.as_str())];
    let admin = enviar(
        &ambiente.app,
        pedido(Method::GET, "/api/v1/auth/admin/me", None, &autorizacao),
    )
    .await;
    assert_eq!(admin.status(), StatusCode::OK);
    let corpo = corpo_json(admin).await;
    assert_eq!(corpo["usuario"]["papel"], "ADMIN");

    let client = enviar(
        &ambiente.app,
        pedido(Method::GET, "/api/v1/auth/client/me", None, &autorizacao),
    )
    .await;
    assert_eq!(client.status(), StatusCode::UNAUTHORIZED);
    let corpo = corpo_json(client).await;
    assert_eq!(corpo["error"]["code"], "AUTH_AUDIENCE_MISMATCH");

    let sem_token = enviar(
        &ambiente.app,
        pedido(Method::GET, "/api/v1/auth/admin/me", None, &[]),
    )
    .await;
    assert_eq!(sem_token.status(), StatusCode::UNAUTHORIZED);
    let corpo = corpo_json(sem_token).await;
    assert_eq!(corpo["error"]["code"], "NAO_AUTENTICADO");
}

#[tokio::test]
async fn refresh_rotaciona_e_reuso_revoga_a_familia() {
    let ambiente = ambiente(&[]);
    semear(&ambiente.banco, "ADMIN", true, Some("12345678901"));

    let resposta = enviar(&ambiente.app, login("admin", SENHA, "ADMIN")).await;
    let cookie_antigo = cabecalho_cookie(&resposta, "accurit_admin_rt").expect("cookie de login");
    let cookie_header = format!(
        "accurit_admin_rt={}",
        cookie_da_resposta(&resposta, "accurit_admin_rt").expect("token")
    );

    let refresh = enviar(
        &ambiente.app,
        pedido(
            Method::POST,
            "/api/v1/auth/admin/refresh",
            None,
            &[
                (header::CONTENT_TYPE.as_str(), "application/json"),
                (header::COOKIE.as_str(), cookie_header.as_str()),
                ("x-requested-with", "accurit"),
            ],
        ),
    )
    .await;
    assert_eq!(refresh.status(), StatusCode::OK);
    let cookie_novo = cabecalho_cookie(&refresh, "accurit_admin_rt").expect("novo cookie");
    assert_ne!(cookie_novo, cookie_antigo);

    let reuso = enviar(
        &ambiente.app,
        pedido(
            Method::POST,
            "/api/v1/auth/admin/refresh",
            None,
            &[
                (header::CONTENT_TYPE.as_str(), "application/json"),
                (header::COOKIE.as_str(), cookie_header.as_str()),
                ("x-requested-with", "accurit"),
            ],
        ),
    )
    .await;
    assert_eq!(reuso.status(), StatusCode::UNAUTHORIZED);
    let corpo = corpo_json(reuso).await;
    assert_eq!(corpo["error"]["code"], "AUTH_SESSION_EXPIRED");

    assert_eq!(
        contar(
            &ambiente.banco,
            "SELECT COUNT(*) AS total FROM sessoes WHERE revogada_em IS NULL"
        ),
        0
    );
    assert_eq!(
        contar(
            &ambiente.banco,
            "SELECT COUNT(*) AS total FROM auditoria_eventos WHERE acao = 'AUTH_REFRESH_REUSO'"
        ),
        1
    );
}

#[tokio::test]
async fn refresh_exige_cabecalho_csrf() {
    let ambiente = ambiente(&[]);
    semear(&ambiente.banco, "ADMIN", true, Some("12345678901"));

    let resposta = enviar(&ambiente.app, login("admin", SENHA, "ADMIN")).await;
    let cookie_header = format!(
        "accurit_admin_rt={}",
        cookie_da_resposta(&resposta, "accurit_admin_rt").expect("token")
    );

    let sem_csrf = enviar(
        &ambiente.app,
        pedido(
            Method::POST,
            "/api/v1/auth/admin/refresh",
            None,
            &[(header::COOKIE.as_str(), cookie_header.as_str())],
        ),
    )
    .await;
    assert_eq!(sem_csrf.status(), StatusCode::BAD_REQUEST);
    let corpo = corpo_json(sem_csrf).await;
    assert_eq!(corpo["error"]["code"], "AUTH_CSRF_HEADER_MISSING");
}

#[tokio::test]
async fn logout_revoga_sessao_e_limpa_cookie() {
    let ambiente = ambiente(&[]);
    semear(&ambiente.banco, "ADMIN", true, Some("12345678901"));

    let resposta = enviar(&ambiente.app, login("admin", SENHA, "ADMIN")).await;
    let cookie_header = format!(
        "accurit_admin_rt={}",
        cookie_da_resposta(&resposta, "accurit_admin_rt").expect("token")
    );

    let logout = enviar(
        &ambiente.app,
        pedido(
            Method::POST,
            "/api/v1/auth/admin/logout",
            None,
            &[
                (header::CONTENT_TYPE.as_str(), "application/json"),
                (header::COOKIE.as_str(), cookie_header.as_str()),
                ("x-requested-with", "accurit"),
            ],
        ),
    )
    .await;
    assert_eq!(logout.status(), StatusCode::NO_CONTENT);
    let limpo = cabecalho_cookie(&logout, "accurit_admin_rt").expect("cookie de limpeza");
    assert!(limpo.contains("Max-Age=0"));

    assert_eq!(
        contar(
            &ambiente.banco,
            "SELECT COUNT(*) AS total FROM sessoes WHERE revogada_em IS NULL"
        ),
        0
    );
    assert_eq!(
        contar(
            &ambiente.banco,
            "SELECT COUNT(*) AS total FROM auditoria_eventos WHERE acao = 'AUTH_LOGOUT'"
        ),
        1
    );
}

#[tokio::test]
async fn bloqueia_usuario_apos_falhas_consecutivas() {
    let ambiente = ambiente(&[("AUTH_MAX_FAILED_ATTEMPTS", "2")]);
    semear(&ambiente.banco, "OPERADOR", true, Some("12345678901"));

    for _ in 0..2 {
        let resposta = enviar(&ambiente.app, login("admin", "senha-errada", "OPERADOR")).await;
        assert_eq!(resposta.status(), StatusCode::UNAUTHORIZED);
    }

    let bloqueado = enviar(&ambiente.app, login("admin", SENHA, "OPERADOR")).await;
    assert_eq!(bloqueado.status(), StatusCode::UNAUTHORIZED);
    let corpo = corpo_json(bloqueado).await;
    assert_eq!(corpo["error"]["code"], "AUTH_INVALID_CREDENTIALS");

    assert_eq!(
        contar(
            &ambiente.banco,
            "SELECT COUNT(*) AS total FROM auditoria_eventos WHERE acao = 'AUTH_LOGIN_FALHA' AND valor_novo->>'motivo' = 'USUARIO_BLOQUEADO'"
        ),
        1
    );
}

#[tokio::test]
async fn usuario_inativo_nao_faz_login() {
    let ambiente = ambiente(&[]);
    semear(&ambiente.banco, "ADMIN", false, Some("12345678901"));

    let resposta = enviar(&ambiente.app, login("admin", SENHA, "ADMIN")).await;
    assert_eq!(resposta.status(), StatusCode::UNAUTHORIZED);

    assert_eq!(
        contar(
            &ambiente.banco,
            "SELECT COUNT(*) AS total FROM auditoria_eventos WHERE acao = 'AUTH_LOGIN_FALHA' AND valor_novo->>'motivo' = 'USUARIO_INATIVO'"
        ),
        1
    );
}

#[tokio::test]
async fn cliente_faz_login_apenas_no_portal_do_cliente() {
    let ambiente = ambiente(&[]);
    semear(&ambiente.banco, "CLIENTE", true, None);

    let portal = enviar(&ambiente.app, login("client", SENHA, "CLIENTE")).await;
    assert_eq!(portal.status(), StatusCode::OK);
    assert!(
        cabecalho_cookie(&portal, "accurit_client_rt").is_some(),
        "portal do cliente deve receber o cookie de cliente"
    );

    let painel = enviar(&ambiente.app, login("admin", SENHA, "CLIENTE")).await;
    assert_eq!(painel.status(), StatusCode::UNAUTHORIZED);
}
