//! Etapa 2 — testes HTTP de gestão de usuários do escritório (portal) e
//! isolamento entre painéis/escritórios.
//!
//! Cobre CRUD pelo titular, paginação/busca no servidor, e-mail único,
//! senha mínima, ativação/desativação, redefinição de senha com revogação
//! de sessões, proteção do último titular ativo, auditoria, funções e
//! carteira, além da separação de audiências admin/client e IDOR.

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
const CNPJ_A: &str = "11222333000181";
const CNPJ_B: &str = "11444777000161";
const CPF_TITULAR: &str = "12345678062";
const CPF_TITULAR_B: &str = "12345678143";
const CPF_OPERADOR: &str = "39053344705";
const CPF_OPERADOR2: &str = "86288366757";
const CPF_ADMIN: &str = "11144477735";

struct BancoFixo;

impl SaudeBanco for BancoFixo {
    async fn checar(&self) -> Result<(), SaudeBancoError> {
        Ok(())
    }
}

struct Ambiente {
    app: Router,
    banco: BancoTemporario,
    escritorio_a: Uuid,
    escritorio_b: Uuid,
    titular_a: Uuid,
}

fn ambiente() -> Ambiente {
    let banco = BancoTemporario::novo();
    aplicar_migrations_com_lock(banco.conexao(), MIGRATIONS).expect("migrations");
    let config = configuracao_com_extras("", &[]);
    let pool = criar_pool(&banco.database_url).expect("pool");
    let auth = AuthState::novo(pool.clone(), &config).expect("auth");
    let estado = AppState::novo(ConsultarSaude::novo(BancoFixo), pool);
    let app = routes::construir(estado, Some(auth), &config);

    let escritorio_a = semear_escritorio(&banco, CNPJ_A, "Escritório A");
    let escritorio_b = semear_escritorio(&banco, CNPJ_B, "Escritório B");
    let titular_a = semear_usuario(
        &banco,
        escritorio_a,
        "titular@a.test",
        CPF_TITULAR,
        "CLIENTE",
        SENHA,
    );
    semear_usuario(
        &banco,
        escritorio_b,
        "titular@b.test",
        CPF_TITULAR_B,
        "CLIENTE",
        SENHA,
    );
    semear_plano_e_assinatura(&banco, escritorio_a);

    Ambiente {
        app,
        banco,
        escritorio_a,
        escritorio_b,
        titular_a,
    }
}

#[derive(QueryableByName)]
struct Contagem {
    #[diesel(sql_type = BigInt)]
    total: i64,
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

fn hash_senha(senha: &str) -> String {
    let config = configuracao_com_extras("", &[]);
    HashArgon2::novo(&config.auth)
        .expect("argon2")
        .gerar_hash(senha)
        .expect("hash")
}

fn semear_usuario(
    banco: &BancoTemporario,
    escritorio: Uuid,
    email: &str,
    cpf: &str,
    papel: &str,
    senha: &str,
) -> Uuid {
    let id = Uuid::now_v7();
    sql_query(
        "INSERT INTO usuarios (id, escritorio_id, email, cpf, senha_hash, papel, ativo) \
         VALUES ($1, $2, $3, $4, $5, $6, TRUE)",
    )
    .bind::<SqlUuid, _>(id)
    .bind::<SqlUuid, _>(escritorio)
    .bind::<Text, _>(email)
    .bind::<Text, _>(cpf)
    .bind::<Text, _>(hash_senha(senha))
    .bind::<Text, _>(papel)
    .execute(&mut banco.conexao())
    .expect("usuario");
    id
}

fn semear_plano_e_assinatura(banco: &BancoTemporario, escritorio: Uuid) {
    let plano = Uuid::now_v7();
    sql_query(
        "INSERT INTO planos_escritorio (id, nome, chamadas_mes, clientes_limite, usuarios_limite) \
         VALUES ($1, 'Plano', 100, 100, 100)",
    )
    .bind::<SqlUuid, _>(plano)
    .execute(&mut banco.conexao())
    .expect("plano");
    sql_query(
        "INSERT INTO assinaturas_escritorio \
         (id, escritorio_id, plano_id, chamadas_mes_limite, clientes_limite, usuarios_limite) \
         VALUES ($1, $2, $3, 100, 100, 100)",
    )
    .bind::<SqlUuid, _>(Uuid::now_v7())
    .bind::<SqlUuid, _>(escritorio)
    .bind::<SqlUuid, _>(plano)
    .execute(&mut banco.conexao())
    .expect("assinatura");
}

fn semear_administrador(banco: &BancoTemporario, email: &str) {
    sql_query(
        "INSERT INTO administradores (id, nome, email, cpf, senha_hash, ativo) \
         VALUES ($1, 'Gestor', $2, $3, $4, TRUE)",
    )
    .bind::<SqlUuid, _>(Uuid::now_v7())
    .bind::<Text, _>(email)
    .bind::<Text, _>(CPF_ADMIN)
    .bind::<Text, _>(hash_senha(SENHA))
    .execute(&mut banco.conexao())
    .expect("administrador");
}

fn contar(ambiente: &Ambiente, sql: &str, bind: Option<Uuid>) -> i64 {
    let mut conexao = ambiente.banco.conexao();
    let consulta = sql_query(sql);
    let resultado = match bind {
        Some(id) => consulta
            .bind::<SqlUuid, _>(id)
            .get_result::<Contagem>(&mut conexao),
        None => consulta.get_result::<Contagem>(&mut conexao),
    };
    resultado.expect("contagem").total
}

fn pedido(
    metodo: Method,
    caminho: &str,
    token: Option<&str>,
    corpo: Option<Value>,
) -> Request<Body> {
    let mut construtor = Request::builder().method(metodo).uri(caminho);
    if let Some(token) = token {
        construtor = construtor.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    match corpo {
        Some(corpo) => construtor
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(corpo.to_string()))
            .expect("request"),
        None => construtor.body(Body::empty()).expect("request"),
    }
}

async fn enviar(app: &Router, request: Request<Body>) -> Response {
    app.clone().oneshot(request).await.expect("response")
}

async fn json(resposta: Response) -> Value {
    let bytes = to_bytes(resposta.into_body(), 4 * 1024 * 1024)
        .await
        .expect("body");
    serde_json::from_slice(&bytes).unwrap_or(Value::Null)
}

async fn login(app: &Router, email: &str, senha: &str) -> String {
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/auth/client/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "email": email, "senha": senha }).to_string(),
        ))
        .expect("request");
    let resposta = enviar(app, request).await;
    assert_eq!(resposta.status(), StatusCode::OK, "login {email}");
    json(resposta).await["accessToken"]
        .as_str()
        .expect("accessToken")
        .to_owned()
}

async fn login_admin(app: &Router, email: &str) -> String {
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/auth/admin/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "email": email, "senha": SENHA }).to_string(),
        ))
        .expect("request");
    let resposta = enviar(app, request).await;
    assert_eq!(resposta.status(), StatusCode::OK, "login admin");
    json(resposta).await["accessToken"]
        .as_str()
        .expect("accessToken")
        .to_owned()
}

fn requisicao_criar(email: &str, cpf: &str, papel: &str, senha: &str) -> Value {
    json!({
        "nome": "Usuário Novo",
        "email": email,
        "cpf": cpf,
        "papel": papel,
        "senha": senha
    })
}

#[tokio::test]
async fn titular_cria_lista_busca_e_edita_usuario() {
    let ambiente = ambiente();
    let token = login(&ambiente.app, "titular@a.test", SENHA).await;

    // Lista inicial: só o titular.
    let resposta = enviar(
        &ambiente.app,
        pedido(Method::GET, "/api/v1/usuarios", Some(&token), None),
    )
    .await;
    assert_eq!(resposta.status(), StatusCode::OK);
    let corpo = json(resposta).await;
    assert_eq!(corpo["total"], 1);

    // Criação.
    let resposta = enviar(
        &ambiente.app,
        pedido(
            Method::POST,
            "/api/v1/usuarios",
            Some(&token),
            Some(requisicao_criar(
                "operador@a.test",
                CPF_OPERADOR,
                "OPERADOR",
                SENHA,
            )),
        ),
    )
    .await;
    assert_eq!(resposta.status(), StatusCode::CREATED);
    let criado = json(resposta).await;
    let id = criado["id"].as_str().expect("id").to_owned();
    assert_eq!(criado["email"], "operador@a.test");
    assert_eq!(criado["papel"], "OPERADOR");
    assert_eq!(criado["escritorioId"], ambiente.escritorio_a.to_string());

    // Busca por id.
    let resposta = enviar(
        &ambiente.app,
        pedido(
            Method::GET,
            &format!("/api/v1/usuarios/{id}"),
            Some(&token),
            None,
        ),
    )
    .await;
    assert_eq!(resposta.status(), StatusCode::OK);

    // Paginação e busca no servidor.
    for (email, cpf) in [
        ("bia@a.test", CPF_OPERADOR2),
        ("carla@a.test", "95835384068"),
    ] {
        let resposta = enviar(
            &ambiente.app,
            pedido(
                Method::POST,
                "/api/v1/usuarios",
                Some(&token),
                Some(requisicao_criar(email, cpf, "OPERADOR", SENHA)),
            ),
        )
        .await;
        assert_eq!(resposta.status(), StatusCode::CREATED);
    }

    let resposta = enviar(
        &ambiente.app,
        pedido(
            Method::GET,
            "/api/v1/usuarios?pagina=2&tamanho=2",
            Some(&token),
            None,
        ),
    )
    .await;
    assert_eq!(resposta.status(), StatusCode::OK);
    let corpo = json(resposta).await;
    assert_eq!(corpo["total"], 4);
    assert_eq!(corpo["pagina"], 2);
    assert_eq!(corpo["tamanho"], 2);
    assert_eq!(corpo["itens"].as_array().expect("itens").len(), 2);

    let resposta = enviar(
        &ambiente.app,
        pedido(
            Method::GET,
            "/api/v1/usuarios?busca=bia",
            Some(&token),
            None,
        ),
    )
    .await;
    let corpo = json(resposta).await;
    assert_eq!(corpo["total"], 1);
    assert_eq!(corpo["itens"][0]["email"], "bia@a.test");

    // Edição de nome e desativação.
    let resposta = enviar(
        &ambiente.app,
        pedido(
            Method::PATCH,
            &format!("/api/v1/usuarios/{id}"),
            Some(&token),
            Some(json!({ "nome": "Usuário Editado", "ativo": false })),
        ),
    )
    .await;
    assert_eq!(resposta.status(), StatusCode::OK);
    let corpo = json(resposta).await;
    assert_eq!(corpo["nome"], "Usuário Editado");
    assert_eq!(corpo["ativo"], false);

    // Auditoria das ações do titular.
    let eventos = contar(
        &ambiente,
        "SELECT COUNT(*) AS total FROM auditoria_eventos WHERE acao = 'USUARIO_CRIADO'",
        None,
    );
    assert!(eventos >= 3, "criações devem ser auditadas");
    let desativacoes = contar(
        &ambiente,
        "SELECT COUNT(*) AS total FROM auditoria_eventos WHERE acao = 'USUARIO_DESATIVADO'",
        None,
    );
    assert_eq!(desativacoes, 1);
}

#[tokio::test]
async fn email_unico_e_senha_minima_sao_validados() {
    let ambiente = ambiente();
    let token = login(&ambiente.app, "titular@a.test", SENHA).await;

    let resposta = enviar(
        &ambiente.app,
        pedido(
            Method::POST,
            "/api/v1/usuarios",
            Some(&token),
            Some(requisicao_criar(
                "operador@a.test",
                CPF_OPERADOR,
                "OPERADOR",
                SENHA,
            )),
        ),
    )
    .await;
    assert_eq!(resposta.status(), StatusCode::CREATED);

    // E-mail repetido na plataforma (mesmo com CPF diferente).
    let resposta = enviar(
        &ambiente.app,
        pedido(
            Method::POST,
            "/api/v1/usuarios",
            Some(&token),
            Some(requisicao_criar(
                "operador@a.test",
                CPF_OPERADOR2,
                "OPERADOR",
                SENHA,
            )),
        ),
    )
    .await;
    assert_eq!(resposta.status(), StatusCode::CONFLICT);
    assert_eq!(json(resposta).await["error"]["code"], "CONFLITO");

    // Senha com menos de 8 caracteres.
    let resposta = enviar(
        &ambiente.app,
        pedido(
            Method::POST,
            "/api/v1/usuarios",
            Some(&token),
            Some(requisicao_criar(
                "curta@a.test",
                "95835384068",
                "OPERADOR",
                "123",
            )),
        ),
    )
    .await;
    assert_eq!(resposta.status(), StatusCode::BAD_REQUEST);
    assert_eq!(json(resposta).await["error"]["code"], "VALIDACAO");
}

#[tokio::test]
async fn protege_ultimo_titular_e_auto_desativacao() {
    let ambiente = ambiente();
    let token = login(&ambiente.app, "titular@a.test", SENHA).await;

    // O único titular ativo não pode ser rebaixado a OPERADOR.
    let resposta = enviar(
        &ambiente.app,
        pedido(
            Method::PATCH,
            &format!("/api/v1/usuarios/{}", ambiente.titular_a),
            Some(&token),
            Some(json!({ "papel": "OPERADOR" })),
        ),
    )
    .await;
    assert_eq!(resposta.status(), StatusCode::CONFLICT);
    assert_eq!(json(resposta).await["error"]["code"], "CONFLITO");

    // Nem desativar a própria conta.
    let resposta = enviar(
        &ambiente.app,
        pedido(
            Method::PATCH,
            &format!("/api/v1/usuarios/{}", ambiente.titular_a),
            Some(&token),
            Some(json!({ "ativo": false })),
        ),
    )
    .await;
    assert_eq!(resposta.status(), StatusCode::CONFLICT);

    // Com um segundo titular ativo, o rebaixamento do primeiro é permitido.
    let resposta = enviar(
        &ambiente.app,
        pedido(
            Method::POST,
            "/api/v1/usuarios",
            Some(&token),
            Some(requisicao_criar(
                "titular2@a.test",
                CPF_OPERADOR2,
                "CLIENTE",
                SENHA,
            )),
        ),
    )
    .await;
    assert_eq!(resposta.status(), StatusCode::CREATED);

    let resposta = enviar(
        &ambiente.app,
        pedido(
            Method::PATCH,
            &format!("/api/v1/usuarios/{}", ambiente.titular_a),
            Some(&token),
            Some(json!({ "papel": "OPERADOR" })),
        ),
    )
    .await;
    assert_eq!(
        resposta.status(),
        StatusCode::OK,
        "com outro titular ativo o rebaixamento é permitido"
    );
}

#[tokio::test]
async fn redefinir_senha_troca_login_e_revoga_sessoes() {
    let ambiente = ambiente();
    let token = login(&ambiente.app, "titular@a.test", SENHA).await;
    let operador = semear_usuario(
        &ambiente.banco,
        ambiente.escritorio_a,
        "operador@a.test",
        CPF_OPERADOR,
        "OPERADOR",
        SENHA,
    );
    let token_operador = login(&ambiente.app, "operador@a.test", SENHA).await;

    let resposta = enviar(
        &ambiente.app,
        pedido(
            Method::POST,
            &format!("/api/v1/usuarios/{operador}/senha"),
            Some(&token),
            Some(json!({ "senha": "nova-senha-987" })),
        ),
    )
    .await;
    assert_eq!(resposta.status(), StatusCode::NO_CONTENT);

    // Sessões do usuário foram revogadas: o token antigo deixa de valer.
    let resposta = enviar(
        &ambiente.app,
        pedido(Method::GET, "/api/v1/usuarios", Some(&token_operador), None),
    )
    .await;
    assert_eq!(resposta.status(), StatusCode::UNAUTHORIZED);

    // A senha antiga não autentica mais; a nova sim.
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/auth/client/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "email": "operador@a.test", "senha": SENHA }).to_string(),
        ))
        .expect("request");
    let resposta = enviar(&ambiente.app, request).await;
    assert_eq!(resposta.status(), StatusCode::UNAUTHORIZED);

    let novo_token = login(&ambiente.app, "operador@a.test", "nova-senha-987").await;
    let resposta = enviar(
        &ambiente.app,
        pedido(Method::GET, "/api/v1/usuarios", Some(&novo_token), None),
    )
    .await;
    // OPERADOR não gerencia usuários, mas o login em si foi aceito.
    assert_eq!(resposta.status(), StatusCode::FORBIDDEN);

    let auditoria = contar(
        &ambiente,
        "SELECT COUNT(*) AS total FROM auditoria_eventos WHERE acao = 'USUARIO_SENHA_REDEFINIDA'",
        None,
    );
    assert_eq!(auditoria, 1);
}

#[tokio::test]
async fn desativar_usuario_revoga_sessoes() {
    let ambiente = ambiente();
    let token = login(&ambiente.app, "titular@a.test", SENHA).await;
    let operador = semear_usuario(
        &ambiente.banco,
        ambiente.escritorio_a,
        "operador@a.test",
        CPF_OPERADOR,
        "OPERADOR",
        SENHA,
    );
    let token_operador = login(&ambiente.app, "operador@a.test", SENHA).await;

    let resposta = enviar(
        &ambiente.app,
        pedido(
            Method::PATCH,
            &format!("/api/v1/usuarios/{operador}"),
            Some(&token),
            Some(json!({ "ativo": false })),
        ),
    )
    .await;
    assert_eq!(resposta.status(), StatusCode::OK);

    let resposta = enviar(
        &ambiente.app,
        pedido(Method::GET, "/api/v1/usuarios", Some(&token_operador), None),
    )
    .await;
    assert_eq!(resposta.status(), StatusCode::UNAUTHORIZED);

    // Login de usuário desativado também falha.
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/auth/client/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "email": "operador@a.test", "senha": SENHA }).to_string(),
        ))
        .expect("request");
    let resposta = enviar(&ambiente.app, request).await;
    assert_eq!(resposta.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn funcao_e_carteira_do_usuario() {
    let ambiente = ambiente();
    let token = login(&ambiente.app, "titular@a.test", SENHA).await;

    let resposta = enviar(
        &ambiente.app,
        pedido(
            Method::POST,
            "/api/v1/usuarios",
            Some(&token),
            Some(requisicao_criar(
                "analista@a.test",
                CPF_OPERADOR,
                "OPERADOR",
                SENHA,
            )),
        ),
    )
    .await;
    assert_eq!(resposta.status(), StatusCode::CREATED);
    let id = json(resposta).await["id"].as_str().expect("id").to_owned();

    let resposta = enviar(
        &ambiente.app,
        pedido(
            Method::PUT,
            &format!("/api/v1/usuarios/{id}/acesso"),
            Some(&token),
            Some(json!({ "funcao": "ANALISTA_FISCAL" })),
        ),
    )
    .await;
    assert_eq!(resposta.status(), StatusCode::OK);
    let corpo = json(resposta).await;
    assert_eq!(corpo["funcao"], "ANALISTA_FISCAL");
    assert_eq!(corpo["podeAdministrarUsuarios"], false);

    let cliente = Uuid::now_v7();
    sql_query(
        "INSERT INTO clientes (id, escritorio_id, tipo, cpf_cnpj, nome_razao_social) \
         VALUES ($1, $2, 'PF', '12345678909', 'Cliente da Carteira')",
    )
    .bind::<SqlUuid, _>(cliente)
    .bind::<SqlUuid, _>(ambiente.escritorio_a)
    .execute(&mut ambiente.banco.conexao())
    .expect("cliente");

    let resposta = enviar(
        &ambiente.app,
        pedido(
            Method::PUT,
            &format!("/api/v1/usuarios/{id}/carteira"),
            Some(&token),
            Some(json!({ "clientes": [cliente] })),
        ),
    )
    .await;
    assert_eq!(resposta.status(), StatusCode::OK);
    let corpo = json(resposta).await;
    assert_eq!(corpo["clientesPermitidos"], json!([cliente.to_string()]));

    // Operador sem função de gestor não acessa a configuração de acesso.
    let operador_token = login(&ambiente.app, "analista@a.test", SENHA).await;
    let resposta = enviar(
        &ambiente.app,
        pedido(
            Method::GET,
            &format!("/api/v1/usuarios/{id}/acesso"),
            Some(&operador_token),
            None,
        ),
    )
    .await;
    assert_eq!(resposta.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn admin_nao_gerencia_usuarios_do_escritorio() {
    let ambiente = ambiente();
    semear_administrador(&ambiente.banco, "gestor@accurit.test");
    let admin_token = login_admin(&ambiente.app, "gestor@accurit.test").await;

    // A rota /usuarios é da audiência client: token admin recebe 401.
    for caminho in ["/api/v1/usuarios", "/api/v1/usuarios/"] {
        let resposta = enviar(
            &ambiente.app,
            pedido(Method::GET, caminho, Some(&admin_token), None),
        )
        .await;
        assert_eq!(resposta.status(), StatusCode::UNAUTHORIZED, "{caminho}");
        assert_eq!(
            json(resposta).await["error"]["code"],
            "AUTH_AUDIENCE_MISMATCH"
        );
    }

    // E o token client não acessa o backoffice.
    let token = login(&ambiente.app, "titular@a.test", SENHA).await;
    for caminho in [
        "/api/v1/admin/administradores",
        "/api/v1/admin/consumo",
        "/api/v1/admin/escritorios",
    ] {
        let resposta = enviar(
            &ambiente.app,
            pedido(Method::GET, caminho, Some(&token), None),
        )
        .await;
        assert!(
            resposta.status() == StatusCode::UNAUTHORIZED
                || resposta.status() == StatusCode::FORBIDDEN,
            "{caminho} deveria negar token client, veio {}",
            resposta.status()
        );
    }
}

#[tokio::test]
async fn idor_entre_escritorios_responde_404_sem_vazar() {
    let ambiente = ambiente();
    let token_b = login(&ambiente.app, "titular@b.test", SENHA).await;

    let caminhos = [
        (
            Method::GET,
            format!("/api/v1/usuarios/{}", ambiente.titular_a),
        ),
        (
            Method::PATCH,
            format!("/api/v1/usuarios/{}", ambiente.titular_a),
        ),
        (
            Method::POST,
            format!("/api/v1/usuarios/{}/senha", ambiente.titular_a),
        ),
        (
            Method::GET,
            format!("/api/v1/usuarios/{}/acesso", ambiente.titular_a),
        ),
    ];

    for (metodo, caminho) in caminhos {
        let corpo = match metodo {
            Method::PATCH => Some(json!({ "nome": "Invasor" })),
            Method::POST => Some(json!({ "senha": "senha-invasor-1" })),
            _ => None,
        };
        let resposta = enviar(
            &ambiente.app,
            pedido(metodo, &caminho, Some(&token_b), corpo),
        )
        .await;
        assert_eq!(resposta.status(), StatusCode::NOT_FOUND, "{caminho}");
        let corpo = json(resposta).await;
        let texto = corpo.to_string();
        assert!(!texto.contains("titular@a.test"));
        assert!(!texto.contains(&ambiente.escritorio_a.to_string()));
        for vazamento in [
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
            assert!(
                !texto.to_lowercase().contains(&vazamento.to_lowercase()),
                "resposta vazou {vazamento}: {texto}"
            );
        }
    }

    // A listagem de B nunca contém usuários de A.
    let resposta = enviar(
        &ambiente.app,
        pedido(Method::GET, "/api/v1/usuarios", Some(&token_b), None),
    )
    .await;
    let corpo = json(resposta).await;
    assert_eq!(corpo["total"], 1);
    assert!(
        !corpo.to_string().contains(&ambiente.titular_a.to_string()),
        "listagem vazou usuário de outro escritório"
    );
    assert_eq!(
        corpo["itens"][0]["escritorioId"],
        ambiente.escritorio_b.to_string()
    );
}

#[tokio::test]
async fn erros_nao_vazam_detalhes_internos() {
    let ambiente = ambiente();
    let token = login(&ambiente.app, "titular@a.test", SENHA).await;

    // UUID inválido na rota.
    let resposta = enviar(
        &ambiente.app,
        pedido(
            Method::GET,
            "/api/v1/usuarios/nao-e-uuid",
            Some(&token),
            None,
        ),
    )
    .await;
    assert!(resposta.status().is_client_error());
    let corpo = json(resposta).await;
    let texto = corpo.to_string().to_lowercase();
    for vazamento in [
        "diesel",
        "postgres",
        "select",
        "constraint",
        "panicked",
        ".rs",
        "localhost",
        "reqwest",
        "gateway.apiserpro",
    ] {
        assert!(
            !texto.contains(vazamento),
            "erro vazou {vazamento}: {texto}"
        );
    }
    assert!(corpo["error"]["requestId"].is_string());
}

/// Evita aviso de campo não usado quando o teste de carteira não roda.
#[allow(dead_code)]
fn _tipos(ambiente: &Ambiente) {
    let _ = (ambiente.escritorio_b, CPF_TITULAR_B);
}
