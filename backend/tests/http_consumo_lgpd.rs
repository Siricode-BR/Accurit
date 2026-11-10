mod common;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use axum::response::Response;
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
    aplicar_migrations_com_lock(banco.conexao(), MIGRATIONS).expect("migrations");
    let config = configuracao_com_extras("", extras);
    let pool = criar_pool(&banco.database_url).expect("pool");
    let auth = AuthState::novo(pool.clone(), &config).expect("auth");
    let estado = AppState::novo(ConsultarSaude::novo(BancoFixo), pool);
    let app = routes::construir(estado, Some(auth), &config);
    Ambiente { app, banco }
}

fn pedido(
    metodo: Method,
    caminho: &str,
    corpo: Option<Value>,
    token: Option<&str>,
) -> Request<Body> {
    let (body, tem_corpo) = match corpo {
        Some(valor) => (Body::from(valor.to_string()), true),
        None => (Body::empty(), false),
    };
    let mut builder = Request::builder().method(metodo).uri(caminho);
    if tem_corpo {
        builder = builder.header(header::CONTENT_TYPE, "application/json");
    }
    if let Some(token) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    builder.body(body).expect("request")
}

async fn enviar(app: &Router, request: Request<Body>) -> Response {
    app.clone().oneshot(request).await.expect("response")
}

async fn json(resposta: Response) -> Value {
    let bytes = to_bytes(resposta.into_body(), 1024 * 1024)
        .await
        .expect("body");
    serde_json::from_slice(&bytes).expect("json")
}

fn token_de(corpo: &Value) -> String {
    corpo["accessToken"].as_str().expect("token").to_owned()
}

fn semear_escritorio(banco: &BancoTemporario, cnpj: &str, nome: &str) -> Uuid {
    let id = Uuid::now_v7();
    sql_query("INSERT INTO escritorios (id, cnpj, nome) VALUES ($1, $2, $3)")
        .bind::<SqlUuid, _>(id)
        .bind::<Text, _>(cnpj)
        .bind::<Text, _>(nome)
        .execute(&mut banco.conexao())
        .expect("escritorio");
    id
}

fn semear_usuario(banco: &BancoTemporario, escritorio: Uuid, email: &str, papel: &str) -> Uuid {
    let config = configuracao_com_extras("", &[]);
    let hash = HashArgon2::novo(&config.auth)
        .expect("argon2")
        .gerar_hash(SENHA)
        .expect("hash");
    let id = Uuid::now_v7();
    sql_query(
        "INSERT INTO usuarios (id, escritorio_id, email, cpf, senha_hash, papel, ativo) \
         VALUES ($1, $2, $3, '52998224725', $4, $5, TRUE)",
    )
    .bind::<SqlUuid, _>(id)
    .bind::<SqlUuid, _>(escritorio)
    .bind::<Text, _>(email)
    .bind::<Text, _>(hash)
    .bind::<Text, _>(papel)
    .execute(&mut banco.conexao())
    .expect("usuario");
    id
}

fn semear_gestor(banco: &BancoTemporario) {
    let config = configuracao_com_extras("", &[]);
    let hash = HashArgon2::novo(&config.auth)
        .expect("argon2")
        .gerar_hash(SENHA)
        .expect("hash");
    sql_query(
        "INSERT INTO administradores (id, nome, email, cpf, senha_hash, ativo) \
         VALUES ($1, 'Gestor', 'gestor@accurit.test', '11144477735', $2, TRUE)",
    )
    .bind::<SqlUuid, _>(Uuid::now_v7())
    .bind::<Text, _>(hash)
    .execute(&mut banco.conexao())
    .expect("gestor");
}

fn semear_plano_e_assinatura(banco: &BancoTemporario, escritorio: Uuid) -> Uuid {
    let plano_id = Uuid::now_v7();
    sql_query(
        "INSERT INTO planos_escritorio \
         (id, nome, chamadas_mes, clientes_limite, usuarios_limite) \
         VALUES ($1, 'Essencial', 10, 50, 5)",
    )
    .bind::<SqlUuid, _>(plano_id)
    .execute(&mut banco.conexao())
    .expect("plano");
    sql_query(
        "INSERT INTO assinaturas_escritorio \
         (id, escritorio_id, plano_id, chamadas_mes_limite, clientes_limite, usuarios_limite) \
         VALUES ($1, $2, $3, 10, 50, 5)",
    )
    .bind::<SqlUuid, _>(Uuid::now_v7())
    .bind::<SqlUuid, _>(escritorio)
    .bind::<SqlUuid, _>(plano_id)
    .execute(&mut banco.conexao())
    .expect("assinatura");
    plano_id
}

fn semear_cliente(banco: &BancoTemporario, escritorio: Uuid) -> Uuid {
    let id = Uuid::now_v7();
    sql_query(
        "INSERT INTO clientes (id, escritorio_id, tipo, cpf_cnpj, nome_razao_social, email) \
         VALUES ($1, $2, 'PF', '52998224725', 'Maria Silva', 'maria@teste.test')",
    )
    .bind::<SqlUuid, _>(id)
    .bind::<SqlUuid, _>(escritorio)
    .execute(&mut banco.conexao())
    .expect("cliente");
    id
}

fn semear_chamadas(banco: &BancoTemporario, escritorio: Uuid, cliente: Uuid, total: i32) {
    for _ in 0..total {
        sql_query(
            "INSERT INTO chamadas_serpro \
             (id, escritorio_id, produto, verbo, servico, ambiente, cliente_id, origem, resultado, status_http, duracao_ms, request_id) \
             VALUES ($1, $2, 'DIVIDA_ATIVA', 'consultar_devedor', 'devedor', 'TRIAL', $3, 'manual', 'sucesso', 200, 120, 'req-teste')",
        )
        .bind::<SqlUuid, _>(Uuid::now_v7())
        .bind::<SqlUuid, _>(escritorio)
        .bind::<SqlUuid, _>(cliente)
        .execute(&mut banco.conexao())
        .expect("chamada");
    }
}

async fn entrar(app: &Router, caminho: &str, email: &str) -> Value {
    let resposta = enviar(
        app,
        pedido(
            Method::POST,
            caminho,
            Some(json!({ "email": email, "senha": SENHA })),
            None,
        ),
    )
    .await;
    assert_eq!(resposta.status(), StatusCode::OK, "login {email}");
    json(resposta).await
}

fn contar(banco: &BancoTemporario, consulta: &str) -> i64 {
    #[derive(diesel::QueryableByName)]
    struct Total {
        #[diesel(sql_type = BigInt)]
        total: i64,
    }
    sql_query(consulta)
        .get_result::<Total>(&mut banco.conexao())
        .expect("contagem")
        .total
}

#[tokio::test]
async fn consumo_cliente_admin_csv_e_planos() {
    let ambiente = ambiente(&[]);
    let escritorio = semear_escritorio(&ambiente.banco, "11222333000181", "Escritório Alfa");
    semear_usuario(
        &ambiente.banco,
        escritorio,
        "operador@alfa.test",
        "OPERADOR",
    );
    let cliente = semear_cliente(&ambiente.banco, escritorio);
    semear_chamadas(&ambiente.banco, escritorio, cliente, 2);
    let plano_id = semear_plano_e_assinatura(&ambiente.banco, escritorio);
    semear_gestor(&ambiente.banco);

    let token_client = token_de(
        &entrar(
            &ambiente.app,
            "/api/v1/auth/client/login",
            "operador@alfa.test",
        )
        .await,
    );
    let consumo = enviar(
        &ambiente.app,
        pedido(Method::GET, "/api/v1/consumo", None, Some(&token_client)),
    )
    .await;
    assert_eq!(consumo.status(), StatusCode::OK);
    let corpo = json(consumo).await;
    assert_eq!(corpo["chamadasUsadasMes"], 2);
    assert_eq!(corpo["chamadasMesLimite"], 10);
    assert_eq!(corpo["clientesLimite"], 50);
    assert_eq!(corpo["usuariosLimite"], 5);
    assert_eq!(corpo["chamadasRecentes"].as_array().unwrap().len(), 2);

    let token_admin = token_de(
        &entrar(
            &ambiente.app,
            "/api/v1/auth/admin/login",
            "gestor@accurit.test",
        )
        .await,
    );
    let admin_consumo = enviar(
        &ambiente.app,
        pedido(
            Method::GET,
            "/api/v1/admin/consumo",
            None,
            Some(&token_admin),
        ),
    )
    .await;
    assert_eq!(admin_consumo.status(), StatusCode::OK);
    let corpo = json(admin_consumo).await;
    assert_eq!(corpo["totalChamadas"], 2);
    assert_eq!(corpo["escritorios"].as_array().unwrap().len(), 1);
    assert_eq!(corpo["escritorios"][0]["planoNome"], "Essencial");

    let csv = enviar(
        &ambiente.app,
        pedido(
            Method::GET,
            "/api/v1/admin/consumo/export.csv",
            None,
            Some(&token_admin),
        ),
    )
    .await;
    assert_eq!(csv.status(), StatusCode::OK);
    assert_eq!(
        csv.headers().get(header::CONTENT_TYPE).unwrap(),
        "text/csv; charset=utf-8"
    );

    let planos = enviar(
        &ambiente.app,
        pedido(
            Method::GET,
            "/api/v1/admin/planos",
            None,
            Some(&token_admin),
        ),
    )
    .await;
    assert_eq!(planos.status(), StatusCode::OK);
    assert_eq!(json(planos).await.as_array().unwrap().len(), 1);

    let novo = enviar(
        &ambiente.app,
        pedido(
            Method::POST,
            "/api/v1/admin/planos",
            Some(json!({
                "nome": "Pro",
                "chamadasMes": 1000,
                "clientesLimite": 500,
                "usuariosLimite": 50,
                "precoMensalCents": 9900
            })),
            Some(&token_admin),
        ),
    )
    .await;
    assert_eq!(novo.status(), StatusCode::CREATED);
    let novo = json(novo).await;
    assert_eq!(novo["nome"], "Pro");

    let atribuir = enviar(
        &ambiente.app,
        pedido(
            Method::PUT,
            &format!("/api/v1/admin/planos/escritorios/{escritorio}"),
            Some(json!({
                "planoId": novo["id"],
                "chamadasMesLimite": 500,
                "bloqueioAoExceder": true
            })),
            Some(&token_admin),
        ),
    )
    .await;
    assert_eq!(atribuir.status(), StatusCode::OK);
    assert_eq!(json(atribuir).await["chamadasMesLimite"], 500);
    assert_ne!(plano_id, Uuid::nil());

    // Token de escritório não acessa o backoffice.
    let negado = enviar(
        &ambiente.app,
        pedido(
            Method::GET,
            "/api/v1/admin/consumo",
            None,
            Some(&token_client),
        ),
    )
    .await;
    assert_eq!(negado.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn lgpd_exporta_anonimiza_consentimento_e_retencao() {
    let ambiente = ambiente(&[]);
    let escritorio = semear_escritorio(&ambiente.banco, "11222333000181", "Escritório Alfa");
    semear_usuario(
        &ambiente.banco,
        escritorio,
        "operador@alfa.test",
        "OPERADOR",
    );
    let cliente = semear_cliente(&ambiente.banco, escritorio);
    let token = token_de(
        &entrar(
            &ambiente.app,
            "/api/v1/auth/client/login",
            "operador@alfa.test",
        )
        .await,
    );

    let export = enviar(
        &ambiente.app,
        pedido(
            Method::GET,
            &format!("/api/v1/lgpd/clientes/{cliente}/export"),
            None,
            Some(&token),
        ),
    )
    .await;
    assert_eq!(export.status(), StatusCode::OK);
    let corpo = json(export).await;
    assert_eq!(corpo["cliente"]["nomeRazaoSocial"], "Maria Silva");
    assert_eq!(corpo["cliente"]["cpfCnpj"], "52998224725");

    let consentimento = enviar(
        &ambiente.app,
        pedido(
            Method::POST,
            &format!("/api/v1/lgpd/clientes/{cliente}/consentimentos"),
            Some(json!({ "tipo": "tratamento_dados", "versao": "1.0" })),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(consentimento.status(), StatusCode::CREATED);

    let lista = enviar(
        &ambiente.app,
        pedido(
            Method::GET,
            &format!("/api/v1/lgpd/clientes/{cliente}/consentimentos"),
            None,
            Some(&token),
        ),
    )
    .await;
    assert_eq!(json(lista).await.as_array().unwrap().len(), 1);

    let retencao = enviar(
        &ambiente.app,
        pedido(
            Method::PUT,
            "/api/v1/lgpd/retencao",
            Some(json!({
                "consultasDias": 120,
                "relatoriosDias": 45,
                "auditoriaDias": 500
            })),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(retencao.status(), StatusCode::OK);
    assert_eq!(json(retencao).await["consultasDias"], 120);

    let anonimizar = enviar(
        &ambiente.app,
        pedido(
            Method::DELETE,
            &format!("/api/v1/lgpd/clientes/{cliente}"),
            None,
            Some(&token),
        ),
    )
    .await;
    assert_eq!(anonimizar.status(), StatusCode::NO_CONTENT);

    let export_pos = enviar(
        &ambiente.app,
        pedido(
            Method::GET,
            &format!("/api/v1/lgpd/clientes/{cliente}/export"),
            None,
            Some(&token),
        ),
    )
    .await;
    let corpo = json(export_pos).await;
    assert!(
        corpo["cliente"]["nomeRazaoSocial"]
            .as_str()
            .unwrap()
            .contains("anonimizado")
    );
    assert!(corpo["cliente"]["email"].is_null());
    assert_eq!(
        contar(
            &ambiente.banco,
            "SELECT COUNT(*) AS total FROM auditoria_eventos WHERE acao = 'LGPD_CLIENTE_ANONIMIZADO'"
        ),
        1
    );
}

#[tokio::test]
async fn metrics_exige_token_e_health_detalhado_responde() {
    let ambiente = ambiente(&[("METRICS_TOKEN", "segredo-metricas")]);
    let sem_token = enviar(
        &ambiente.app,
        pedido(Method::GET, "/api/v1/metrics", None, None),
    )
    .await;
    assert_eq!(sem_token.status(), StatusCode::FORBIDDEN);

    let com_token = enviar(
        &ambiente.app,
        pedido(
            Method::GET,
            "/api/v1/metrics",
            None,
            Some("segredo-metricas"),
        ),
    )
    .await;
    assert_eq!(com_token.status(), StatusCode::OK);
    let bytes = to_bytes(com_token.into_body(), 1024 * 1024).await.unwrap();
    let texto = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(texto.contains("accurit_http_requests_total"));
    assert!(texto.contains("accurit_info"));

    let health = enviar(
        &ambiente.app,
        pedido(Method::GET, "/api/v1/health/detalhado", None, None),
    )
    .await;
    assert_eq!(health.status(), StatusCode::OK);
    let corpo = json(health).await;
    assert_eq!(corpo["status"], "ok");
    assert!(corpo["uptimeSegundos"].as_u64().is_some());
}
