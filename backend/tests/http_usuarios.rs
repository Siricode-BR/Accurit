#![allow(dead_code)]
mod common;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use axum::response::Response;
use diesel::QueryableByName;
use diesel::RunQueryDsl;
use diesel::sql_query;
use diesel::sql_types::{BigInt, Text, Uuid as SqlUuid};
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
const EMAIL_GESTOR: &str = "gestor@accurit.test";

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
    let auth = AuthState::novo(pool.clone(), &config).expect("auth deve inicializar");
    let estado = AppState::novo(ConsultarSaude::novo(BancoFixo), pool);
    let app = routes::construir(estado, Some(auth), &config);

    Ambiente { app, banco }
}

#[derive(QueryableByName)]
struct IdRow {
    #[diesel(sql_type = SqlUuid)]
    id: Uuid,
}

#[derive(QueryableByName)]
struct Contagem {
    #[diesel(sql_type = BigInt)]
    total: i64,
}

fn cpf_com_dv(seed: u64) -> String {
    let base = 876_543_210_u64 + (seed % 100_000_000);
    let parte = format!("{base:0>9}");
    let digitos: Vec<u32> = parte.bytes().map(|b| u32::from(b - b'0')).collect();
    let calc = |parcela: &[u32], peso: u32| {
        let soma: u32 = parcela
            .iter()
            .enumerate()
            .map(|(indice, digito)| digito * (peso - indice as u32))
            .sum();
        let resto = (soma * 10) % 11;
        if resto == 10 { 0 } else { resto }
    };
    let primeiro = calc(&digitos[..9], 10);
    let mut com_primeiro = digitos.clone();
    com_primeiro.push(primeiro);
    let segundo = calc(&com_primeiro[..10], 11);
    format!("{parte}{primeiro}{segundo}")
}

fn semear_escritorio(banco: &BancoTemporario, slug: &str) -> Uuid {
    let mut conexao = banco.conexao();
    let id = Uuid::now_v7();
    sql_query("INSERT INTO escritorios (id, slug, nome) VALUES ($1, $2, $3)")
        .bind::<SqlUuid, _>(id)
        .bind::<Text, _>(slug)
        .bind::<Text, _>(format!("Escritório {slug}"))
        .execute(&mut conexao)
        .expect("escritório deve ser inserido");
    id
}

fn id_do_escritorio(banco: &BancoTemporario, slug: &str) -> Option<Uuid> {
    sql_query("SELECT id FROM escritorios WHERE slug = $1")
        .bind::<Text, _>(slug)
        .get_result::<IdRow>(&mut banco.conexao())
        .ok()
        .map(|linha| linha.id)
}

fn semear_usuario(banco: &BancoTemporario, slug: &str, papel: &str, ativo: bool, cpf: &str) {
    let escritorio_id =
        id_do_escritorio(banco, slug).unwrap_or_else(|| semear_escritorio(banco, slug));

    let mut conexao = banco.conexao();
    let config = configuracao_com_extras("", &[]);
    let argon2 = HashArgon2::novo(&config.auth).expect("argon2 deve inicializar");
    let hash = argon2.gerar_hash(SENHA).expect("hash deve ser gerado");

    sql_query(
        "INSERT INTO usuarios (id, escritorio_id, email, cpf, senha_hash, papel, ativo) \
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind::<SqlUuid, _>(Uuid::now_v7())
    .bind::<SqlUuid, _>(escritorio_id)
    .bind::<Text, _>(format!("{papel}@{slug}.test").to_lowercase())
    .bind::<Text, _>(cpf.to_owned())
    .bind::<Text, _>(hash)
    .bind::<Text, _>(papel)
    .bind::<diesel::sql_types::Bool, _>(ativo)
    .execute(&mut conexao)
    .expect("usuário deve ser inserido");
}

fn semear_gestor(banco: &BancoTemporario, ativo: bool) {
    let mut conexao = banco.conexao();
    let config = configuracao_com_extras("", &[]);
    let argon2 = HashArgon2::novo(&config.auth).expect("argon2 deve inicializar");
    let hash = argon2.gerar_hash(SENHA).expect("hash deve ser gerado");

    sql_query(
        "INSERT INTO administradores (id, email, cpf, senha_hash, ativo) \
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind::<SqlUuid, _>(Uuid::now_v7())
    .bind::<Text, _>(EMAIL_GESTOR)
    .bind::<Text, _>("52998224725")
    .bind::<Text, _>(hash)
    .bind::<diesel::sql_types::Bool, _>(ativo)
    .execute(&mut conexao)
    .expect("administrador deve ser inserido");
}

fn pedido(
    metodo: Method,
    caminho: &str,
    corpo: Option<Value>,
    token: Option<&str>,
) -> Request<Body> {
    let (corpo, tem_corpo) = match corpo {
        Some(valor) => (Body::from(valor.to_string()), true),
        None => (Body::empty(), false),
    };

    let mut construtor = Request::builder().method(metodo).uri(caminho);
    if tem_corpo {
        construtor = construtor.header(header::CONTENT_TYPE, "application/json");
    }
    if let Some(token) = token {
        construtor = construtor.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    construtor
        .header("x-real-ip", "10.9.0.1")
        .body(corpo)
        .expect("requisição válida")
}

async fn enviar(app: &Router, requisicao: Request<Body>) -> Response {
    app.clone()
        .oneshot(requisicao)
        .await
        .expect("resposta deve existir")
}

async fn corpo_json(resposta: Response) -> Value {
    let bytes = to_bytes(resposta.into_body(), 1024 * 1024)
        .await
        .expect("corpo deve ser lido");
    serde_json::from_slice(&bytes).expect("corpo deve ser JSON")
}

fn contar(banco: &BancoTemporario, consulta: &str) -> i64 {
    sql_query(consulta)
        .get_result::<Contagem>(&mut banco.conexao())
        .expect("consulta deve funcionar")
        .total
}

fn token_de(corpo: &Value) -> String {
    corpo["accessToken"]
        .as_str()
        .expect("accessToken presente")
        .to_owned()
}

fn codigo_de_erro(corpo: &Value) -> &str {
    corpo["error"]["code"].as_str().expect("envelope de erro")
}

async fn entrar_portal(app: &Router, slug: &str, papel: &str) -> Value {
    let resposta = enviar(
        app,
        pedido(
            Method::POST,
            "/api/v1/auth/client/login",
            Some(json!({
                "email": format!("{papel}@{slug}.test"),
                "senha": SENHA,
            })),
            None,
        ),
    )
    .await;
    assert_eq!(
        resposta.status(),
        StatusCode::OK,
        "login de {papel} deve funcionar no escritório {slug}"
    );
    corpo_json(resposta).await
}

async fn entrar_gestor(app: &Router) -> Value {
    let resposta = enviar(
        app,
        pedido(
            Method::POST,
            "/api/v1/auth/admin/login",
            Some(json!({
                "email": EMAIL_GESTOR,
                "senha": SENHA,
            })),
            None,
        ),
    )
    .await;
    assert_eq!(resposta.status(), StatusCode::OK, "login do gestor");
    corpo_json(resposta).await
}

const U: &str = "/api/v1/usuarios";
const A: &str = "/api/v1/admin/usuarios";

fn novo_usuario(email: &str, seed: u64, papel: &str) -> Value {
    json!({"nome": "Fulano de Tal", "email": email, "cpf": cpf_com_dv(seed), "papel": papel, "senha": "outra-senha-123"})
}

async fn chamar(
    app: &Router,
    m: Method,
    c: &str,
    corpo: Option<Value>,
    t: &str,
) -> (StatusCode, Value) {
    let r = enviar(app, pedido(m, c, corpo, Some(t))).await;
    let s = r.status();
    let b = if s == StatusCode::NO_CONTENT {
        Value::Null
    } else {
        corpo_json(r).await
    };
    (s, b)
}

#[tokio::test]
async fn titular_gerencia_usuarios_do_proprio_escritorio() {
    let a = ambiente(&[]);
    semear_usuario(&a.banco, "alfa", "CLIENTE", true, &cpf_com_dv(1));
    semear_usuario(&a.banco, "alfa", "OPERADOR", true, &cpf_com_dv(2));
    semear_usuario(&a.banco, "beta", "CLIENTE", true, &cpf_com_dv(3));
    let login = entrar_portal(&a.app, "alfa", "CLIENTE").await;
    assert!(login["usuario"]["nome"].is_string());
    let t = token_de(&login);
    let tb = token_de(&entrar_portal(&a.app, "beta", "CLIENTE").await);

    let (s, criado) = chamar(
        &a.app,
        Method::POST,
        U,
        Some(novo_usuario("Novo@Alfa.test", 10, "OPERADOR")),
        &t,
    )
    .await;
    assert_eq!(s, StatusCode::CREATED);
    assert_eq!(criado["email"], "novo@alfa.test");
    assert_eq!(criado["escritorioSlug"], "alfa");
    assert!(criado.get("senhaHash").is_none());
    let id = criado["id"].as_str().unwrap().to_owned();

    let (_, lista) = chamar(&a.app, Method::GET, U, None, &t).await;
    assert_eq!(lista["total"], 3);
    let (_, lista) = chamar(
        &a.app,
        Method::GET,
        &format!("{U}?busca=novo%40alfa&ativo=true"),
        None,
        &t,
    )
    .await;
    assert_eq!(lista["total"], 1);

    // Isolamento: outro tenant não enxerga nem altera.
    let (s, _) = chamar(&a.app, Method::GET, &format!("{U}/{id}"), None, &tb).await;
    assert_eq!(s, StatusCode::NOT_FOUND);
    let (s, _) = chamar(
        &a.app,
        Method::PATCH,
        &format!("{U}/{id}"),
        Some(json!({"ativo": false})),
        &tb,
    )
    .await;
    assert_eq!(s, StatusCode::NOT_FOUND);
    let (_, lb) = chamar(&a.app, Method::GET, U, None, &tb).await;
    assert_eq!(lb["total"], 1);

    // E-mail global duplicado (outro escritório) e CPF duplicado no mesmo escritório.
    let (s, _) = chamar(
        &a.app,
        Method::POST,
        U,
        Some(novo_usuario("cliente@beta.test", 11, "OPERADOR")),
        &t,
    )
    .await;
    assert_eq!(s, StatusCode::CONFLICT);
    let (s, _) = chamar(
        &a.app,
        Method::POST,
        U,
        Some(novo_usuario("x@alfa.test", 10, "OPERADOR")),
        &t,
    )
    .await;
    assert_eq!(s, StatusCode::CONFLICT);
    let (s, _) = chamar(&a.app, Method::POST, U, Some(json!({"nome":"X","email":"y@alfa.test","cpf":"11111111111","papel":"OPERADOR","senha":"outra-senha-123"})), &t).await;
    assert_eq!(s, StatusCode::BAD_REQUEST);

    // Atualização + desativação revoga sessões; login deixa de funcionar.
    let (s, atu) = chamar(
        &a.app,
        Method::PATCH,
        &format!("{U}/{id}"),
        Some(json!({"nome": "Renomeado"})),
        &t,
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(atu["nome"], "Renomeado");
    let novo = entrar_portal_email(&a.app, "novo@alfa.test", "outra-senha-123").await;
    assert_eq!(novo, StatusCode::OK);
    let (s, d) = chamar(
        &a.app,
        Method::PATCH,
        &format!("{U}/{id}"),
        Some(json!({"ativo": false})),
        &t,
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(d["ativo"], false);
    assert_ne!(
        entrar_portal_email(&a.app, "novo@alfa.test", "outra-senha-123").await,
        StatusCode::OK
    );
    assert_eq!(
        contar(
            &a.banco,
            "SELECT count(*) AS total FROM sessoes s JOIN usuarios u ON u.id=s.usuario_id WHERE u.email='novo@alfa.test' AND s.revogada_em IS NULL"
        ),
        0
    );

    // Redefinir senha.
    let (s, _) = chamar(
        &a.app,
        Method::PATCH,
        &format!("{U}/{id}"),
        Some(json!({"ativo": true})),
        &t,
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    let (s, _) = chamar(
        &a.app,
        Method::POST,
        &format!("{U}/{id}/senha"),
        Some(json!({"senha": "curta"})),
        &t,
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
    let (s, _) = chamar(
        &a.app,
        Method::POST,
        &format!("{U}/{id}/senha"),
        Some(json!({"senha": "senha-nova-456"})),
        &t,
    )
    .await;
    assert_eq!(s, StatusCode::NO_CONTENT);
    assert_ne!(
        entrar_portal_email(&a.app, "novo@alfa.test", "outra-senha-123").await,
        StatusCode::OK
    );
    assert_eq!(
        entrar_portal_email(&a.app, "novo@alfa.test", "senha-nova-456").await,
        StatusCode::OK
    );

    assert!(
        contar(
            &a.banco,
            "SELECT count(*) AS total FROM auditoria_eventos WHERE entidade='usuario' AND usuario_id IS NOT NULL"
        ) >= 5
    );
    assert_eq!(
        contar(
            &a.banco,
            "SELECT count(*) AS total FROM auditoria_eventos WHERE valor_novo::text ILIKE '%senha-nova%'"
        ),
        0
    );
}

async fn entrar_portal_email(app: &Router, email: &str, senha: &str) -> StatusCode {
    enviar(
        app,
        pedido(
            Method::POST,
            "/api/v1/auth/client/login",
            Some(json!({"email": email, "senha": senha})),
            None,
        ),
    )
    .await
    .status()
}

#[tokio::test]
async fn operador_nao_gerencia_e_titular_tem_travas() {
    let a = ambiente(&[]);
    semear_usuario(&a.banco, "alfa", "CLIENTE", true, &cpf_com_dv(1));
    semear_usuario(&a.banco, "alfa", "OPERADOR", true, &cpf_com_dv(2));
    let to = token_de(&entrar_portal(&a.app, "alfa", "OPERADOR").await);
    let login = entrar_portal(&a.app, "alfa", "CLIENTE").await;
    let t = token_de(&login);
    let meu_id = login["usuario"]["id"].as_str().unwrap().to_owned();

    let (s, _) = chamar(&a.app, Method::GET, U, None, &to).await;
    assert_eq!(s, StatusCode::FORBIDDEN);
    let (s, _) = chamar(
        &a.app,
        Method::POST,
        U,
        Some(novo_usuario("z@alfa.test", 20, "OPERADOR")),
        &to,
    )
    .await;
    assert_eq!(s, StatusCode::FORBIDDEN);

    let (s, _) = chamar(
        &a.app,
        Method::PATCH,
        &format!("{U}/{meu_id}"),
        Some(json!({"ativo": false})),
        &t,
    )
    .await;
    assert_eq!(s, StatusCode::CONFLICT);
    let (s, _) = chamar(
        &a.app,
        Method::PATCH,
        &format!("{U}/{meu_id}"),
        Some(json!({"papel": "OPERADOR"})),
        &t,
    )
    .await;
    assert_eq!(s, StatusCode::CONFLICT);
    let (s, _) = chamar(
        &a.app,
        Method::PATCH,
        &format!("{U}/{meu_id}"),
        Some(json!({})),
        &t,
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn admin_gerencia_usuarios_globalmente() {
    let a = ambiente(&[]);
    semear_gestor(&a.banco, true);
    semear_usuario(&a.banco, "alfa", "CLIENTE", true, &cpf_com_dv(1));
    semear_usuario(&a.banco, "beta", "CLIENTE", true, &cpf_com_dv(2));
    let tc = token_de(&entrar_portal(&a.app, "alfa", "CLIENTE").await);
    let t = token_de(&entrar_gestor(&a.app).await);

    // Audiências cruzadas são recusadas.
    let (s, _) = chamar(&a.app, Method::GET, A, None, &tc).await;
    assert_ne!(s, StatusCode::OK);
    let (s, _) = chamar(&a.app, Method::GET, U, None, &t).await;
    assert_ne!(s, StatusCode::OK);

    let (_, todos) = chamar(&a.app, Method::GET, A, None, &t).await;
    assert_eq!(todos["total"], 2);
    let esc = id_do_escritorio(&a.banco, "beta").unwrap();
    let (_, so_beta) = chamar(
        &a.app,
        Method::GET,
        &format!("{A}?escritorioId={esc}"),
        None,
        &t,
    )
    .await;
    assert_eq!(so_beta["total"], 1);
    assert_eq!(so_beta["itens"][0]["escritorioSlug"], "beta");

    let mut corpo = novo_usuario("op@beta.test", 30, "OPERADOR");
    let (s, _) = chamar(&a.app, Method::POST, A, Some(corpo.clone()), &t).await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
    corpo["escritorioId"] = json!(esc);
    let (s, criado) = chamar(&a.app, Method::POST, A, Some(corpo.clone()), &t).await;
    assert_eq!(s, StatusCode::CREATED);
    let (s, _) = chamar(&a.app, Method::POST, A, Some(corpo), &t).await;
    assert_eq!(s, StatusCode::CONFLICT);
    let id = criado["id"].as_str().unwrap();

    let (s, d) = chamar(
        &a.app,
        Method::PATCH,
        &format!("{A}/{id}"),
        Some(json!({"ativo": false, "papel": "OPERADOR"})),
        &t,
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(d["ativo"], false);
    let (s, _) = chamar(
        &a.app,
        Method::POST,
        &format!("{A}/{id}/senha"),
        Some(json!({"senha": "senha-nova-456"})),
        &t,
    )
    .await;
    assert_eq!(s, StatusCode::NO_CONTENT);
    assert!(
        contar(
            &a.banco,
            "SELECT count(*) AS total FROM auditoria_eventos WHERE administrador_id IS NOT NULL AND entidade='usuario'"
        ) >= 3
    );

    // Último titular ativo não pode ser desativado, nem pelo admin.
    let alfa = chamar(
        &a.app,
        Method::GET,
        &format!("{A}?busca=cliente%40alfa"),
        None,
        &t,
    )
    .await
    .1;
    let alfa_id = alfa["itens"][0]["id"].as_str().unwrap();
    let (s, _) = chamar(
        &a.app,
        Method::PATCH,
        &format!("{A}/{alfa_id}"),
        Some(json!({"ativo": false})),
        &t,
    )
    .await;
    assert_eq!(s, StatusCode::CONFLICT);
}
