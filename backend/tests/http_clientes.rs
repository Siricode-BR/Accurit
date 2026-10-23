mod common;

use std::time::Duration;

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
                "escritorio": slug,
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

fn corpo_cliente(tipo: &str, cpf_cnpj: &str, nome: &str, porte_regime: Option<&str>) -> Value {
    json!({
        "tipo": tipo,
        "porteRegime": porte_regime,
        "cpfCnpj": cpf_cnpj,
        "nomeRazaoSocial": nome,
        "nomeFantasia": Value::Null,
        "email": "contato@exemplo.test",
        "telefone": "+5511999999999",
    })
}

#[tokio::test]
async fn crud_de_clientes_isolado_por_escritorio() {
    let ambiente = ambiente(&[]);
    semear_usuario(&ambiente.banco, "alfa", "OPERADOR", true, &cpf_com_dv(1));
    semear_usuario(&ambiente.banco, "beta", "OPERADOR", true, &cpf_com_dv(2));

    let token_alfa = token_de(&entrar_portal(&ambiente.app, "alfa", "OPERADOR").await);
    let token_beta = token_de(&entrar_portal(&ambiente.app, "beta", "OPERADOR").await);

    let resposta = enviar(
        &ambiente.app,
        pedido(
            Method::POST,
            "/api/v1/clientes",
            Some(corpo_cliente(
                "Pf",
                "529.982.247-25",
                "Maria da Silva",
                None,
            )),
            Some(&token_alfa),
        ),
    )
    .await;
    assert_eq!(resposta.status(), StatusCode::CREATED);
    let cliente = corpo_json(resposta).await;
    let cliente_id = cliente["id"].as_str().expect("id do cliente").to_owned();
    assert_eq!(cliente["cpfCnpj"], "52998224725");
    assert_eq!(cliente["nomeRazaoSocial"], "Maria da Silva");
    assert_eq!(cliente["email"], "contato@exemplo.test");

    let duplicado = enviar(
        &ambiente.app,
        pedido(
            Method::POST,
            "/api/v1/clientes",
            Some(corpo_cliente("Pf", "52998224725", "Maria da Silva", None)),
            Some(&token_alfa),
        ),
    )
    .await;
    assert_eq!(duplicado.status(), StatusCode::CONFLICT);
    let corpo = corpo_json(duplicado).await;
    assert_eq!(codigo_de_erro(&corpo), "CONFLITO");

    let mesma_resposta = enviar(
        &ambiente.app,
        pedido(
            Method::POST,
            "/api/v1/clientes",
            Some(corpo_cliente("Pf", "12345678909", "Outro Tenant", None)),
            Some(&token_beta),
        ),
    )
    .await;
    assert_eq!(
        mesma_resposta.status(),
        StatusCode::CREATED,
        "mesmo CPF em outro escritório deve ser permitido"
    );

    let leitura = enviar(
        &ambiente.app,
        pedido(
            Method::GET,
            &format!("/api/v1/clientes/{cliente_id}"),
            None,
            Some(&token_beta),
        ),
    )
    .await;
    assert_eq!(leitura.status(), StatusCode::NOT_FOUND);
    let corpo = corpo_json(leitura).await;
    assert_eq!(codigo_de_erro(&corpo), "NAO_ENCONTRADO");

    let alteracao = enviar(
        &ambiente.app,
        pedido(
            Method::PUT,
            &format!("/api/v1/clientes/{cliente_id}"),
            Some(corpo_cliente(
                "Pf",
                "52998224725",
                "Mudanca no Alheio",
                None,
            )),
            Some(&token_beta),
        ),
    )
    .await;
    assert_eq!(alteracao.status(), StatusCode::NOT_FOUND);

    let remocao = enviar(
        &ambiente.app,
        pedido(
            Method::DELETE,
            &format!("/api/v1/clientes/{cliente_id}"),
            None,
            Some(&token_beta),
        ),
    )
    .await;
    assert_eq!(remocao.status(), StatusCode::NOT_FOUND);

    let lista = enviar(
        &ambiente.app,
        pedido(Method::GET, "/api/v1/clientes", None, Some(&token_beta)),
    )
    .await;
    assert_eq!(lista.status(), StatusCode::OK);
    let corpo = corpo_json(lista).await;
    assert_eq!(corpo["total"], 1);
    assert_eq!(corpo["itens"].as_array().expect("itens").len(), 1);
    assert_ne!(corpo["itens"][0]["id"].as_str().unwrap(), cliente_id);

    let lista_alfa = enviar(
        &ambiente.app,
        pedido(Method::GET, "/api/v1/clientes", None, Some(&token_alfa)),
    )
    .await;
    let corpo = corpo_json(lista_alfa).await;
    assert_eq!(corpo["total"], 1);
    assert_eq!(corpo["itens"][0]["id"].as_str().unwrap(), cliente_id);

    assert_eq!(
        contar(
            &ambiente.banco,
            "SELECT COUNT(*) AS total FROM auditoria_eventos WHERE acao = 'CLIENTE_CRIADO'"
        ),
        2
    );
}

#[tokio::test]
async fn validacao_de_criacao_retorna_400_com_envelope() {
    let ambiente = ambiente(&[]);
    semear_usuario(&ambiente.banco, "alfa", "OPERADOR", true, &cpf_com_dv(3));
    let token = token_de(&entrar_portal(&ambiente.app, "alfa", "OPERADOR").await);

    for (descricao, corpo) in [
        (
            "cpf com verificador errado",
            corpo_cliente("Pf", "52998224724", "Maria da Silva", None),
        ),
        (
            "cpf com dígitos repetidos",
            corpo_cliente("Pf", "11111111111", "Maria da Silva", None),
        ),
        (
            "cpf com tamanho errado",
            corpo_cliente("Pf", "123456789", "Maria da Silva", None),
        ),
        (
            "pessoa física com porte",
            corpo_cliente("Pf", "52998224725", "Maria da Silva", Some("MEI")),
        ),
        (
            "pessoa jurídica com CPF",
            corpo_cliente("Pj", "52998224725", "Acme", None),
        ),
        (
            "nome vazio",
            corpo_cliente("Pf", "52998224725", "   ", None),
        ),
    ] {
        let resposta = enviar(
            &ambiente.app,
            pedido(Method::POST, "/api/v1/clientes", Some(corpo), Some(&token)),
        )
        .await;
        assert_eq!(resposta.status(), StatusCode::BAD_REQUEST, "{descricao}");
        assert_eq!(
            resposta
                .headers()
                .get(header::CACHE_CONTROL)
                .and_then(|valor| valor.to_str().ok()),
            Some("no-store")
        );
        let corpo = corpo_json(resposta).await;
        assert_eq!(codigo_de_erro(&corpo), "VALIDACAO", "{descricao}");
        assert!(corpo["error"]["requestId"].as_str().is_some());
    }
}

#[tokio::test]
async fn atualiza_e_exclui_cliente_com_auditoria() {
    let ambiente = ambiente(&[]);
    semear_usuario(&ambiente.banco, "alfa", "OPERADOR", true, &cpf_com_dv(4));
    let token = token_de(&entrar_portal(&ambiente.app, "alfa", "OPERADOR").await);

    let criacao = enviar(
        &ambiente.app,
        pedido(
            Method::POST,
            "/api/v1/clientes",
            Some(corpo_cliente(
                "Pj",
                "11444777000161",
                "Acme Serviços Ltda",
                Some("SIMPLES_NACIONAL"),
            )),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(criacao.status(), StatusCode::CREATED);
    let cliente = corpo_json(criacao).await;
    let cliente_id = cliente["id"].as_str().expect("id").to_owned();
    assert_eq!(cliente["porteRegime"], "SIMPLES_NACIONAL");

    let atualizacao = enviar(
        &ambiente.app,
        pedido(
            Method::PUT,
            &format!("/api/v1/clientes/{cliente_id}"),
            Some(json!({
                "nomeRazaoSocial": "Acme Serviços Ltda Atualizado",
                "nomeFantasia": "Acme",
                "email": "novo@exemplo.test",
                "telefone": null,
            })),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(atualizacao.status(), StatusCode::OK);
    let atualizado = corpo_json(atualizacao).await;
    assert_eq!(
        atualizado["nomeRazaoSocial"],
        "Acme Serviços Ltda Atualizado"
    );
    assert_eq!(atualizado["nomeFantasia"], "Acme");
    assert_eq!(atualizado["email"], "novo@exemplo.test");
    assert!(
        atualizado["telefone"].is_null(),
        "telefone limpo deve permanecer nulo"
    );

    let sem_validacao = enviar(
        &ambiente.app,
        pedido(
            Method::PUT,
            &format!("/api/v1/clientes/{cliente_id}"),
            Some(json!({ "nomeRazaoSocial": "" })),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(sem_validacao.status(), StatusCode::BAD_REQUEST);
    let corpo = corpo_json(sem_validacao).await;
    assert_eq!(codigo_de_erro(&corpo), "VALIDACAO");

    let remocao = enviar(
        &ambiente.app,
        pedido(
            Method::DELETE,
            &format!("/api/v1/clientes/{cliente_id}"),
            None,
            Some(&token),
        ),
    )
    .await;
    assert_eq!(remocao.status(), StatusCode::NO_CONTENT);

    let leitura = enviar(
        &ambiente.app,
        pedido(
            Method::GET,
            &format!("/api/v1/clientes/{cliente_id}"),
            None,
            Some(&token),
        ),
    )
    .await;
    assert_eq!(leitura.status(), StatusCode::NOT_FOUND);

    let remocao_repetida = enviar(
        &ambiente.app,
        pedido(
            Method::DELETE,
            &format!("/api/v1/clientes/{cliente_id}"),
            None,
            Some(&token),
        ),
    )
    .await;
    assert_eq!(remocao_repetida.status(), StatusCode::NOT_FOUND);

    assert_eq!(
        contar(
            &ambiente.banco,
            "SELECT COUNT(*) AS total FROM auditoria_eventos WHERE acao = 'CLIENTE_CRIADO'"
        ),
        1
    );
    assert_eq!(
        contar(
            &ambiente.banco,
            "SELECT COUNT(*) AS total FROM auditoria_eventos WHERE acao = 'CLIENTE_ATUALIZADO'"
        ),
        1
    );
    assert_eq!(
        contar(
            &ambiente.banco,
            "SELECT COUNT(*) AS total FROM auditoria_eventos WHERE acao = 'CLIENTE_EXCLUIDO'"
        ),
        1
    );
    assert_eq!(
        contar(
            &ambiente.banco,
            "SELECT COUNT(*) AS total FROM auditoria_eventos WHERE acao = 'CLIENTE_ATUALIZADO' AND valor_anterior IS NOT NULL AND valor_novo IS NOT NULL"
        ),
        1
    );
}

#[tokio::test]
async fn filtros_paginacao_e_erros_invalidos() {
    let ambiente = ambiente(&[]);
    semear_usuario(&ambiente.banco, "alfa", "OPERADOR", true, &cpf_com_dv(5));
    let token = token_de(&entrar_portal(&ambiente.app, "alfa", "OPERADOR").await);

    for (cpf, tipo, porte) in [
        ("52998224725", "Pf", None),
        ("12345678909", "Pf", None),
        ("16899535009", "Pf", None),
        ("11222333000181", "Pj", Some("MEI")),
    ] {
        let resposta = enviar(
            &ambiente.app,
            pedido(
                Method::POST,
                "/api/v1/clientes",
                Some(corpo_cliente(tipo, cpf, &format!("Cliente {cpf}"), porte)),
                Some(&token),
            ),
        )
        .await;
        assert_eq!(resposta.status(), StatusCode::CREATED, "cliente {cpf}");
    }

    let lista = enviar(
        &ambiente.app,
        pedido(Method::GET, "/api/v1/clientes", None, Some(&token)),
    )
    .await;
    let corpo = corpo_json(lista).await;
    assert_eq!(corpo["total"], 4);
    assert_eq!(corpo["pagina"], 1);
    assert_eq!(corpo["tamanho"], 20);

    let filtrada = enviar(
        &ambiente.app,
        pedido(
            Method::GET,
            "/api/v1/clientes?filtro=cpf",
            None,
            Some(&token),
        ),
    )
    .await;
    let corpo = corpo_json(filtrada).await;
    assert_eq!(corpo["total"], 3);
    for item in corpo["itens"].as_array().expect("itens") {
        assert_eq!(item["tipo"], "Pf");
    }

    let mey = enviar(
        &ambiente.app,
        pedido(
            Method::GET,
            "/api/v1/clientes?filtro=cnpj_mei",
            None,
            Some(&token),
        ),
    )
    .await;
    let corpo = corpo_json(mey).await;
    assert_eq!(corpo["total"], 1);
    assert_eq!(corpo["itens"][0]["tipo"], "Pj");
    assert_eq!(corpo["itens"][0]["porteRegime"], "MEI");

    let paginada = enviar(
        &ambiente.app,
        pedido(
            Method::GET,
            "/api/v1/clientes?pagina=2&tamanho=3",
            None,
            Some(&token),
        ),
    )
    .await;
    let corpo = corpo_json(paginada).await;
    assert_eq!(corpo["total"], 4);
    assert_eq!(corpo["pagina"], 2);
    assert_eq!(corpo["tamanho"], 3);
    assert_eq!(corpo["itens"].as_array().expect("itens").len(), 1);

    for (descricao, consulta, codigo) in [
        (
            "filtro desconhecido",
            "/api/v1/clientes?filtro=rural",
            "FILTRO_INVALIDO",
        ),
        (
            "pagina zero",
            "/api/v1/clientes?pagina=0",
            "PAGINACAO_INVALIDA",
        ),
        (
            "tamanho acima do limite",
            "/api/v1/clientes?tamanho=101",
            "PAGINACAO_INVALIDA",
        ),
    ] {
        let resposta = enviar(
            &ambiente.app,
            pedido(Method::GET, consulta, None, Some(&token)),
        )
        .await;
        assert_eq!(resposta.status(), StatusCode::BAD_REQUEST, "{descricao}");
        let corpo = corpo_json(resposta).await;
        assert_eq!(codigo_de_erro(&corpo), codigo, "{descricao}");
    }
}

#[tokio::test]
async fn papeis_de_clientes_restringem_operacoes() {
    let ambiente = ambiente(&[]);
    semear_usuario(&ambiente.banco, "alfa", "CLIENTE", true, &cpf_com_dv(11));
    semear_usuario(&ambiente.banco, "alfa", "OPERADOR", true, &cpf_com_dv(12));

    let token_cliente = token_de(&entrar_portal(&ambiente.app, "alfa", "CLIENTE").await);
    let token_operador = token_de(&entrar_portal(&ambiente.app, "alfa", "OPERADOR").await);

    let criacao = enviar(
        &ambiente.app,
        pedido(
            Method::POST,
            "/api/v1/clientes",
            Some(corpo_cliente("Pf", "52998224725", "Maria da Silva", None)),
            Some(&token_cliente),
        ),
    )
    .await;
    assert_eq!(criacao.status(), StatusCode::CREATED);
    let cliente = corpo_json(criacao).await;
    let cliente_id = cliente["id"].as_str().expect("id").to_owned();

    let sem_papel = enviar(
        &ambiente.app,
        pedido(
            Method::PUT,
            &format!("/api/v1/clientes/{cliente_id}"),
            Some(json!({ "nomeRazaoSocial": "Novo Nome" })),
            Some(&token_cliente),
        ),
    )
    .await;
    assert_eq!(sem_papel.status(), StatusCode::FORBIDDEN);
    let corpo = corpo_json(sem_papel).await;
    assert_eq!(codigo_de_erro(&corpo), "PROIBIDO");

    let exclusao_cliente = enviar(
        &ambiente.app,
        pedido(
            Method::DELETE,
            &format!("/api/v1/clientes/{cliente_id}"),
            None,
            Some(&token_cliente),
        ),
    )
    .await;
    assert_eq!(exclusao_cliente.status(), StatusCode::FORBIDDEN);

    let alteracao = enviar(
        &ambiente.app,
        pedido(
            Method::PUT,
            &format!("/api/v1/clientes/{cliente_id}"),
            Some(json!({ "nomeRazaoSocial": "Papel Operador" })),
            Some(&token_operador),
        ),
    )
    .await;
    assert_eq!(alteracao.status(), StatusCode::OK);

    let exclusao = enviar(
        &ambiente.app,
        pedido(
            Method::DELETE,
            &format!("/api/v1/clientes/{cliente_id}"),
            None,
            Some(&token_operador),
        ),
    )
    .await;
    assert_eq!(exclusao.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn rotas_de_clientes_exigem_token_e_audiencia_client() {
    let ambiente = ambiente(&[]);
    semear_usuario(&ambiente.banco, "alfa", "OPERADOR", true, &cpf_com_dv(6));
    semear_gestor(&ambiente.banco, true);

    let sem_token = enviar(
        &ambiente.app,
        pedido(Method::GET, "/api/v1/clientes", None, None),
    )
    .await;
    assert_eq!(sem_token.status(), StatusCode::UNAUTHORIZED);
    let corpo = corpo_json(sem_token).await;
    assert_eq!(codigo_de_erro(&corpo), "NAO_AUTENTICADO");

    let respostas = [
        pedido(Method::GET, "/api/v1/clientes", None, None),
        pedido(
            Method::POST,
            "/api/v1/clientes",
            Some(corpo_cliente("Pf", "52998224725", "Maria", None)),
            None,
        ),
        pedido(
            Method::GET,
            &format!("/api/v1/clientes/{}", Uuid::now_v7()),
            None,
            None,
        ),
        pedido(
            Method::PUT,
            &format!("/api/v1/clientes/{}", Uuid::now_v7()),
            Some(json!({})),
            None,
        ),
        pedido(
            Method::DELETE,
            &format!("/api/v1/clientes/{}", Uuid::now_v7()),
            None,
            None,
        ),
    ];
    for requisicao in respostas {
        let resposta = enviar(&ambiente.app, requisicao).await;
        assert_eq!(resposta.status(), StatusCode::UNAUTHORIZED);
        let corpo = corpo_json(resposta).await;
        assert_eq!(codigo_de_erro(&corpo), "NAO_AUTENTICADO");
    }

    let corpo = entrar_gestor(&ambiente.app).await;
    let token_gestor = token_de(&corpo);

    let metodos = [
        (Method::GET, "/api/v1/clientes", None),
        (
            Method::POST,
            "/api/v1/clientes",
            Some(corpo_cliente("Pf", "52998224725", "Maria", None)),
        ),
        (
            Method::GET,
            &*format!("/api/v1/clientes/{}", Uuid::now_v7()),
            None,
        ),
        (
            Method::PUT,
            &*format!("/api/v1/clientes/{}", Uuid::now_v7()),
            Some(json!({})),
        ),
        (
            Method::DELETE,
            &*format!("/api/v1/clientes/{}", Uuid::now_v7()),
            None,
        ),
    ];
    for (metodo, caminho, corpo) in metodos {
        let resposta = enviar(
            &ambiente.app,
            pedido(metodo, caminho, corpo, Some(&token_gestor)),
        )
        .await;
        assert_eq!(
            resposta.status(),
            StatusCode::UNAUTHORIZED,
            "gestor em {caminho}"
        );
        let corpo = corpo_json(resposta).await;
        assert_eq!(
            codigo_de_erro(&corpo),
            "AUTH_AUDIENCE_MISMATCH",
            "gestor em {caminho}"
        );
    }
}

#[tokio::test]
async fn token_expirado_retorna_sessao_expirada() {
    let ambiente = ambiente(&[("AUTH_ACCESS_TTL_SECS", "1")]);
    semear_usuario(&ambiente.banco, "alfa", "OPERADOR", true, &cpf_com_dv(7));

    let corpo = entrar_portal(&ambiente.app, "alfa", "OPERADOR").await;
    let token = token_de(&corpo);

    let enquanto_valido = enviar(
        &ambiente.app,
        pedido(Method::GET, "/api/v1/clientes", None, Some(&token)),
    )
    .await;
    assert_eq!(enquanto_valido.status(), StatusCode::OK);

    std::thread::sleep(Duration::from_secs(3));

    let expirado = enviar(
        &ambiente.app,
        pedido(Method::GET, "/api/v1/clientes", None, Some(&token)),
    )
    .await;
    assert_eq!(expirado.status(), StatusCode::UNAUTHORIZED);
    let corpo = corpo_json(expirado).await;
    assert_eq!(codigo_de_erro(&corpo), "AUTH_SESSION_EXPIRED");
}

fn corpo_provisionamento(slug: &str, email: &str, cpf: &str, senha: &str) -> Value {
    json!({
        "slug": slug,
        "nome": "Novo Tenant Ltda",
        "emailTitular": email,
        "cpfTitular": cpf,
        "senhaTitular": senha,
    })
}

#[tokio::test]
async fn gestor_provisiona_tenant_e_titular_faz_login() {
    let ambiente = ambiente(&[]);
    semear_gestor(&ambiente.banco, true);

    let corpo = entrar_gestor(&ambiente.app).await;
    let token_gestor = token_de(&corpo);

    let criacao = enviar(
        &ambiente.app,
        pedido(
            Method::POST,
            "/api/v1/admin/escritorios",
            Some(corpo_provisionamento(
                "novo-tenant",
                "titular@novo-tenant.test",
                "12345678909",
                "senha-titular-9",
            )),
            Some(&token_gestor),
        ),
    )
    .await;
    assert_eq!(criacao.status(), StatusCode::CREATED);
    let corpo = corpo_json(criacao).await;
    assert_eq!(corpo["slug"], "novo-tenant");
    assert_eq!(corpo["emailTitular"], "titular@novo-tenant.test");
    assert!(corpo["escritorioId"].as_str().is_some());
    assert!(corpo["titularId"].as_str().is_some());

    assert_eq!(
        contar(
            &ambiente.banco,
            "SELECT COUNT(*) AS total FROM escritorios WHERE slug = 'novo-tenant'"
        ),
        1
    );
    assert_eq!(
        contar(
            &ambiente.banco,
            "SELECT COUNT(*) AS total FROM usuarios WHERE email = 'titular@novo-tenant.test' \
             AND papel = 'CLIENTE' AND ativo IS TRUE"
        ),
        1
    );
    assert_eq!(
        contar(
            &ambiente.banco,
            "SELECT COUNT(*) AS total FROM auditoria_eventos WHERE acao = 'TENANT_PROVISIONADO' \
             AND administrador_id IS NOT NULL AND usuario_id IS NULL"
        ),
        1
    );

    let login_titular = enviar(
        &ambiente.app,
        pedido(
            Method::POST,
            "/api/v1/auth/client/login",
            Some(json!({
                "escritorio": "novo-tenant",
                "email": "titular@novo-tenant.test",
                "senha": "senha-titular-9",
            })),
            None,
        ),
    )
    .await;
    assert_eq!(login_titular.status(), StatusCode::OK);
    let corpo = corpo_json(login_titular).await;
    assert_eq!(corpo["usuario"]["papel"], "CLIENTE");

    let duplicado = enviar(
        &ambiente.app,
        pedido(
            Method::POST,
            "/api/v1/admin/escritorios",
            Some(corpo_provisionamento(
                "novo-tenant",
                "titular@novo-tenant.test",
                "98765432100",
                "outra-senha-12",
            )),
            Some(&token_gestor),
        ),
    )
    .await;
    assert_eq!(duplicado.status(), StatusCode::CONFLICT);
    let corpo = corpo_json(duplicado).await;
    assert_eq!(
        corpo["error"]["message"],
        "Usuário com este e-mail já existe neste escritório"
    );

    assert_eq!(
        contar(&ambiente.banco, "SELECT COUNT(*) AS total FROM escritorios"),
        1
    );
    assert_eq!(
        contar(&ambiente.banco, "SELECT COUNT(*) AS total FROM usuarios"),
        1
    );
}

#[tokio::test]
async fn provisionamento_exige_audiencia_admin_e_payload_valido() {
    let ambiente = ambiente(&[]);
    semear_gestor(&ambiente.banco, true);
    semear_usuario(&ambiente.banco, "alfa", "OPERADOR", true, &cpf_com_dv(8));

    let sem_token = enviar(
        &ambiente.app,
        pedido(
            Method::POST,
            "/api/v1/admin/escritorios",
            Some(corpo_provisionamento(
                "novo-tenant",
                "titular@novo-tenant.test",
                "12345678909",
                "senha-titular-9",
            )),
            None,
        ),
    )
    .await;
    assert_eq!(sem_token.status(), StatusCode::UNAUTHORIZED);
    let corpo = corpo_json(sem_token).await;
    assert_eq!(codigo_de_erro(&corpo), "NAO_AUTENTICADO");

    let corpo = entrar_portal(&ambiente.app, "alfa", "OPERADOR").await;
    let token_operador = token_de(&corpo);

    let usuario_comum = enviar(
        &ambiente.app,
        pedido(
            Method::POST,
            "/api/v1/admin/escritorios",
            Some(corpo_provisionamento(
                "novo-tenant",
                "titular@novo-tenant.test",
                "12345678909",
                "senha-titular-9",
            )),
            Some(&token_operador),
        ),
    )
    .await;
    assert_eq!(usuario_comum.status(), StatusCode::UNAUTHORIZED);
    let corpo = corpo_json(usuario_comum).await;
    assert_eq!(codigo_de_erro(&corpo), "AUTH_AUDIENCE_MISMATCH");

    let corpo = entrar_gestor(&ambiente.app).await;
    let token_gestor = token_de(&corpo);

    for (descricao, corpo) in [
        (
            "slug com maiúsculas e símbolos",
            corpo_provisionamento(
                "Tenant_A!",
                "titular@novo-tenant.test",
                "12345678909",
                "senha-titular-9",
            ),
        ),
        (
            "cpf do titular inválido",
            corpo_provisionamento(
                "novo-tenant",
                "titular@novo-tenant.test",
                "12345678900",
                "senha-titular-9",
            ),
        ),
        (
            "senha do titular curta",
            corpo_provisionamento(
                "novo-tenant",
                "titular@novo-tenant.test",
                "12345678909",
                "curta",
            ),
        ),
        (
            "email do titular inválido",
            corpo_provisionamento(
                "novo-tenant",
                "sem-arroba",
                "12345678909",
                "senha-titular-9",
            ),
        ),
    ] {
        let resposta = enviar(
            &ambiente.app,
            pedido(
                Method::POST,
                "/api/v1/admin/escritorios",
                Some(corpo),
                Some(&token_gestor),
            ),
        )
        .await;
        assert_eq!(resposta.status(), StatusCode::BAD_REQUEST, "{descricao}");
        let corpo = corpo_json(resposta).await;
        assert_eq!(codigo_de_erro(&corpo), "VALIDACAO", "{descricao}");
    }

    assert_eq!(
        contar(
            &ambiente.banco,
            "SELECT COUNT(*) AS total FROM escritorios WHERE slug = 'novo-tenant'"
        ),
        0
    );
    assert_eq!(
        contar(
            &ambiente.banco,
            "SELECT COUNT(*) AS total FROM usuarios WHERE email = 'titular@novo-tenant.test'"
        ),
        0
    );
}
