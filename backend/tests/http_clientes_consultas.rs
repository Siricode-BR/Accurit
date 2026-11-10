//! Integração das rotas de leitura `GET /clientes/{id}/consultas` e
//! `GET /clientes/{id}/situacao-fiscal` (SPEC §9.3 — FASE 1).
//! Cobre: isolação por escritório (IDOR/404 sem vazar existência),
//! filtro de estado e paginação com envelope, audiência do token e o
//! `DISTINCT ON (servico)` dos cartões de situação.

mod common;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use axum::response::Response;
use diesel::sql_types::{Text, Uuid as SqlUuid};
use diesel::{QueryableByName, RunQueryDsl, sql_query};
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
const AVISO_CONSULTIVO: &str =
    "Consulta informativa. Não realiza negociação, parcelamento nem emissão de DARF.";

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

fn cnpj_de_slug(slug: &str) -> String {
    let valor = slug.bytes().fold(0u64, |acc, b| {
        acc.wrapping_mul(31).wrapping_add(u64::from(b))
    });
    format!("{:014}", valor % 100_000_000_000_000)
}

fn semear_escritorio(banco: &BancoTemporario, slug: &str) -> Uuid {
    let mut conexao = banco.conexao();
    let id = Uuid::now_v7();
    sql_query("INSERT INTO escritorios (id, cnpj, nome) VALUES ($1, $2, $3)")
        .bind::<SqlUuid, _>(id)
        .bind::<Text, _>(cnpj_de_slug(slug))
        .bind::<Text, _>(format!("Escritório {slug}"))
        .execute(&mut conexao)
        .expect("escritório deve ser inserido");
    id
}

fn id_do_escritorio(banco: &BancoTemporario, slug: &str) -> Option<Uuid> {
    sql_query("SELECT id FROM escritorios WHERE cnpj = $1")
        .bind::<Text, _>(cnpj_de_slug(slug))
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
        "INSERT INTO administradores (id, nome, email, cpf, senha_hash, ativo) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind::<SqlUuid, _>(Uuid::now_v7())
    .bind::<Text, _>("Gestor de teste")
    .bind::<Text, _>(EMAIL_GESTOR)
    .bind::<Text, _>("52998224725")
    .bind::<Text, _>(hash)
    .bind::<diesel::sql_types::Bool, _>(ativo)
    .execute(&mut conexao)
    .expect("administrador deve ser inserido");
}

fn semear_cliente(banco: &BancoTemporario, escritorio_id: Uuid, doc: &str, nome: &str) -> Uuid {
    let mut conexao = banco.conexao();
    let id = Uuid::now_v7();
    sql_query(
        "INSERT INTO clientes (id, escritorio_id, tipo, cpf_cnpj, nome_razao_social) \
         VALUES ($1, $2, 'PF', $3, $4)",
    )
    .bind::<SqlUuid, _>(id)
    .bind::<SqlUuid, _>(escritorio_id)
    .bind::<Text, _>(doc)
    .bind::<Text, _>(nome)
    .execute(&mut conexao)
    .expect("cliente deve ser inserido");
    id
}

/// `criado_em` explícito: a ordenação do histórico é determinística por
/// (criado_em DESC, id DESC) e o teste precisa controlar os empates.
#[allow(clippy::too_many_arguments)]
fn semear_consulta(
    banco: &BancoTemporario,
    escritorio_id: Uuid,
    cliente_id: Uuid,
    servico: &str,
    ambiente: &str,
    estado: &str,
    mascarada: Option<&str>,
    status_code: Option<i32>,
    fonte: Option<&str>,
    criado_em: chrono::DateTime<chrono::Utc>,
) {
    sql_query(
        "INSERT INTO consultas_fiscais \
         (id, escritorio_id, cliente_id, produto, servico, verbo, ambiente, \
          chave_consulta_hash, chave_consulta_mascarada, fonte_consulta, \
          status_code, estado, protocolo, request_redigido, response_redigido, criado_em) \
         VALUES ($1, $2, $3, 'DIVIDA_ATIVA', $4, 'GET', $5, NULL, $6, $7, \
                 $8, $9, NULL, $10::jsonb, $11::jsonb, $12)",
    )
    .bind::<SqlUuid, _>(Uuid::now_v7())
    .bind::<SqlUuid, _>(escritorio_id)
    .bind::<SqlUuid, _>(cliente_id)
    .bind::<Text, _>(servico)
    .bind::<Text, _>(ambiente)
    .bind::<diesel::sql_types::Nullable<Text>, _>(mascarada)
    .bind::<diesel::sql_types::Nullable<Text>, _>(fonte)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Int4>, _>(status_code)
    .bind::<Text, _>(estado)
    .bind::<diesel::sql_types::Nullable<Text>, _>(Some(r#"{"pedido":true}"#))
    .bind::<diesel::sql_types::Nullable<Text>, _>(Some(r#"{"resposta":true}"#))
    .bind::<diesel::sql_types::Timestamptz, _>(criado_em)
    .execute(&mut banco.conexao())
    .expect("consulta fiscal deve ser inserida");
}

fn instante(segundos: i64) -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::from_timestamp(segundos, 0).expect("timestamp válido")
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

#[tokio::test]
async fn historico_e_isolado_por_escritorio_e_404_nao_vaza_existencia() {
    let ambiente = ambiente(&[]);
    semear_usuario(&ambiente.banco, "alfa", "OPERADOR", true, &cpf_com_dv(1));
    semear_usuario(&ambiente.banco, "beta", "OPERADOR", true, &cpf_com_dv(2));

    let token_alfa = token_de(&entrar_portal(&ambiente.app, "alfa", "OPERADOR").await);

    let escritorio_alfa = id_do_escritorio(&ambiente.banco, "alfa").expect("escritório alfa");
    let escritorio_beta = id_do_escritorio(&ambiente.banco, "beta").expect("escritório beta");

    let cliente_alfa = semear_cliente(&ambiente.banco, escritorio_alfa, "52998224725", "Dona Alfa");
    let cliente_beta = semear_cliente(&ambiente.banco, escritorio_beta, "52998224725", "Dona Beta");

    semear_consulta(
        &ambiente.banco,
        escritorio_alfa,
        cliente_alfa,
        "DEVEDOR",
        "TRIAL",
        "SUCESSO",
        Some("***.***.247-**"),
        Some(200),
        Some("CPF_CNPJ"),
        instante(1_700_000_000),
    );
    semear_consulta(
        &ambiente.banco,
        escritorio_alfa,
        cliente_alfa,
        "DEVEDOR",
        "TRIAL",
        "SEM_PENDENCIAS",
        Some("***.***.247-**"),
        Some(200),
        Some("CPF_CNPJ"),
        instante(1_700_000_100),
    );
    semear_consulta(
        &ambiente.banco,
        escritorio_beta,
        cliente_beta,
        "DEVEDOR",
        "TRIAL",
        "SUCESSO",
        Some("***.***.247-**"),
        Some(200),
        Some("CPF_CNPJ"),
        instante(1_700_000_200),
    );

    let resposta = enviar(
        &ambiente.app,
        pedido(
            Method::GET,
            &format!("/api/v1/clientes/{cliente_alfa}/consultas"),
            None,
            Some(&token_alfa),
        ),
    )
    .await;
    assert_eq!(resposta.status(), StatusCode::OK);
    let corpo = corpo_json(resposta).await;
    assert_eq!(corpo["total"], 2);
    assert_eq!(corpo["pagina"], 1);
    assert_eq!(corpo["itens"].as_array().expect("itens").len(), 2);

    // Ordenação determinística: a tentativa mais recente vem primeiro —
    // e o campo redigido completo nunca é exposto.
    assert_eq!(corpo["itens"][0]["estado"], "SEM_PENDENCIAS");
    assert_eq!(corpo["itens"][1]["estado"], "SUCESSO");
    let item_texto = corpo.to_string();
    assert!(!item_texto.contains("requestRedigido"));
    assert!(!item_texto.contains("responseRedigido"));
    assert!(!item_texto.contains("chaveConsultaHash"));
    assert_eq!(corpo["itens"][0]["produto"], "DIVIDA_ATIVA");
    assert_eq!(corpo["itens"][0]["ambiente"], "TRIAL");
    assert_eq!(
        corpo["itens"][0]["chaveConsultaMascarada"],
        "***.***.247-**"
    );

    // Cliente do outro escritório: 404, sem vazar existência (IDOR).
    let resposta_alheia = enviar(
        &ambiente.app,
        pedido(
            Method::GET,
            &format!("/api/v1/clientes/{cliente_beta}/consultas"),
            None,
            Some(&token_alfa),
        ),
    )
    .await;
    assert_eq!(resposta_alheia.status(), StatusCode::NOT_FOUND);
    assert_eq!(
        codigo_de_erro(&corpo_json(resposta_alheia).await),
        "NAO_ENCONTRADO"
    );

    // Cliente inexistente: mesmo 404 (não distingue do alheio).
    let fantasma = Uuid::now_v7();
    let resposta_fantasma = enviar(
        &ambiente.app,
        pedido(
            Method::GET,
            &format!("/api/v1/clientes/{fantasma}/consultas"),
            None,
            Some(&token_alfa),
        ),
    )
    .await;
    assert_eq!(resposta_fantasma.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn filtro_de_estado_e_paginacao_invalidos_respondem_envelope() {
    let ambiente = ambiente(&[]);
    semear_usuario(&ambiente.banco, "alfa", "CLIENTE", true, &cpf_com_dv(3));

    let token = token_de(&entrar_portal(&ambiente.app, "alfa", "CLIENTE").await);
    let escritorio_alfa = id_do_escritorio(&ambiente.banco, "alfa").expect("escritório alfa");
    let cliente = semear_cliente(&ambiente.banco, escritorio_alfa, &cpf_com_dv(3), "Cliente");

    semear_consulta(
        &ambiente.banco,
        escritorio_alfa,
        cliente,
        "DEVEDOR",
        "TRIAL",
        "SUCESSO",
        None,
        Some(200),
        None,
        instante(1_700_000_000),
    );
    semear_consulta(
        &ambiente.banco,
        escritorio_alfa,
        cliente,
        "DEVEDOR",
        "TRIAL",
        "INDISPONIVEL",
        None,
        Some(503),
        None,
        instante(1_700_000_050),
    );

    let resposta = enviar(
        &ambiente.app,
        pedido(
            Method::GET,
            &format!("/api/v1/clientes/{cliente}/consultas"),
            None,
            Some(&token),
        ),
    )
    .await;
    assert_eq!(resposta.status(), StatusCode::OK);
    let corpo = corpo_json(resposta).await;
    assert_eq!(corpo["total"], 2);

    let resposta = enviar(
        &ambiente.app,
        pedido(
            Method::GET,
            &format!("/api/v1/clientes/{cliente}/consultas?estado=SUCESSO"),
            None,
            Some(&token),
        ),
    )
    .await;
    assert_eq!(resposta.status(), StatusCode::OK);
    let corpo = corpo_json(resposta).await;
    assert_eq!(corpo["total"], 1);
    assert_eq!(corpo["itens"][0]["estado"], "SUCESSO");
    assert_eq!(corpo["itens"][0]["statusCode"], 200);

    let invalido = enviar(
        &ambiente.app,
        pedido(
            Method::GET,
            &format!("/api/v1/clientes/{cliente}/consultas?estado=sucesso"),
            None,
            Some(&token),
        ),
    )
    .await;
    assert_eq!(invalido.status(), StatusCode::BAD_REQUEST);
    assert_eq!(codigo_de_erro(&corpo_json(invalido).await), "VALIDACAO");

    let paginacao_bad = enviar(
        &ambiente.app,
        pedido(
            Method::GET,
            &format!("/api/v1/clientes/{cliente}/consultas?pagina=0"),
            None,
            Some(&token),
        ),
    )
    .await;
    assert_eq!(paginacao_bad.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        codigo_de_erro(&corpo_json(paginacao_bad).await),
        "PAGINACAO_INVALIDA"
    );
}

#[tokio::test]
async fn token_do_gestor_admin_e_rejeitado_nas_rotas_do_portal() {
    let ambiente = ambiente(&[]);
    semear_gestor(&ambiente.banco, true);
    semear_usuario(&ambiente.banco, "alfa", "OPERADOR", true, &cpf_com_dv(4));

    let token_gestor = token_de(&entrar_gestor(&ambiente.app).await);
    let escritorio_alfa = id_do_escritorio(&ambiente.banco, "alfa").expect("escritório alfa");
    let cliente = semear_cliente(&ambiente.banco, escritorio_alfa, &cpf_com_dv(4), "Cliente");

    let resposta = enviar(
        &ambiente.app,
        pedido(
            Method::GET,
            &format!("/api/v1/clientes/{cliente}/consultas"),
            None,
            Some(&token_gestor),
        ),
    )
    .await;
    assert_eq!(resposta.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        codigo_de_erro(&corpo_json(resposta).await),
        "AUTH_AUDIENCE_MISMATCH"
    );

    let resposta = enviar(
        &ambiente.app,
        pedido(
            Method::GET,
            &format!("/api/v1/clientes/{cliente}/situacao-fiscal"),
            None,
            Some(&token_gestor),
        ),
    )
    .await;
    assert_eq!(resposta.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn situacao_fiscal_e_a_ultima_tentativa_por_servico() {
    let ambiente = ambiente(&[]);
    semear_usuario(&ambiente.banco, "alfa", "OPERADOR", true, &cpf_com_dv(5));

    let token = token_de(&entrar_portal(&ambiente.app, "alfa", "OPERADOR").await);
    let escritorio_alfa = id_do_escritorio(&ambiente.banco, "alfa").expect("escritório alfa");
    let cliente = semear_cliente(&ambiente.banco, escritorio_alfa, &cpf_com_dv(5), "Cliente");

    // DEVEDOR tem duas tentativas (SUCESSO antiga, INDETERMINADO nova);
    // INSCRICAO só uma. O resumo precisa trazer a última de cada serviço.
    semear_consulta(
        &ambiente.banco,
        escritorio_alfa,
        cliente,
        "DEVEDOR",
        "TRIAL",
        "SUCESSO",
        Some("***.***.247-**"),
        Some(200),
        Some("CPF_CNPJ"),
        instante(1_700_000_000),
    );
    semear_consulta(
        &ambiente.banco,
        escritorio_alfa,
        cliente,
        "DEVEDOR",
        "TRIAL",
        "INDETERMINADO",
        Some("***.***.247-**"),
        Some(504),
        Some("CPF_CNPJ"),
        instante(1_700_000_100),
    );
    semear_consulta(
        &ambiente.banco,
        escritorio_alfa,
        cliente,
        "INSCRICAO",
        "TRIAL",
        "SEM_PENDENCIAS",
        Some("123.456.78-90"),
        Some(200),
        Some("CPF_CNPJ"),
        instante(1_700_000_200),
    );

    let resposta = enviar(
        &ambiente.app,
        pedido(
            Method::GET,
            &format!("/api/v1/clientes/{cliente}/situacao-fiscal"),
            None,
            Some(&token),
        ),
    )
    .await;
    assert_eq!(resposta.status(), StatusCode::OK);
    let corpo = corpo_json(resposta).await;

    assert_eq!(corpo["aviso"], AVISO_CONSULTIVO);
    let servicos = corpo["servicos"].as_array().expect("servicos");
    assert_eq!(servicos.len(), 2);

    let devedor = servicos
        .iter()
        .find(|s| s["servico"] == "DEVEDOR")
        .expect("cartão DEVEDOR");
    assert_eq!(devedor["estado"], "INDETERMINADO");
    assert_eq!(devedor["statusCode"], 504);

    let inscricao = servicos
        .iter()
        .find(|s| s["servico"] == "INSCRICAO")
        .expect("cartão INSCRICAO");
    assert_eq!(inscricao["estado"], "SEM_PENDENCIAS");

    // Cliente sem nenhuma consulta: resumo vazio, não 404.
    let cliente_fresco = semear_cliente(&ambiente.banco, escritorio_alfa, &cpf_com_dv(6), "Novo");
    let resposta = enviar(
        &ambiente.app,
        pedido(
            Method::GET,
            &format!("/api/v1/clientes/{cliente_fresco}/situacao-fiscal"),
            None,
            Some(&token),
        ),
    )
    .await;
    assert_eq!(resposta.status(), StatusCode::OK);
    let corpo = corpo_json(resposta).await;
    assert_eq!(corpo["servicos"].as_array().expect("servicos").len(), 0);
    assert_eq!(corpo["aviso"], AVISO_CONSULTIVO);
}
