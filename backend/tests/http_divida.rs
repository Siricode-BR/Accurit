//! Testes de integração HTTP da Fase 4 (CONSULTIVO Dívida Ativa,
//! SPEC §4.3§§4.5): leitura autenticada via portal com mock serial
//! SERPRO in-process (127.0.0.1:0), registro em `consultas_fiscais`,
//! redação (sem valores no log), mapeamento estável de erros,
//! `Retry-After`, rate limit local por escritório, header de operador
//! (spoof ignorado), isolamento por escritório, produção bloqueada
//! (D5) e expurgo de retenção (SPEC §9.11).

mod common;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::extract::{Path, State};
use axum::http::{HeaderMap, Method, Request, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use diesel::QueryableByName;
use diesel::RunQueryDsl;
use diesel::sql_query;
use diesel::sql_types::{BigInt, Nullable, Text};
use secrecy::SecretString;
use serde_json::{Value, json};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use tower::ServiceExt;
use url::Url;
use uuid::Uuid;

use accurit_backend::application::saude::ConsultarSaude;
use accurit_backend::application::divida_ativa::ports::RepositorioConsultasFiscais;
use accurit_backend::application::ports::{SaudeBanco, SaudeBancoError};
use accurit_backend::infrastructure::auth::HashArgon2;
use accurit_backend::infrastructure::config::Config;
use accurit_backend::infrastructure::divida_ativa::repositorio::ConsultasFiscaisRepositorioDiesel;
use accurit_backend::infrastructure::persistence::migrations::{
    MIGRATIONS, aplicar_migrations_com_lock,
};
use accurit_backend::infrastructure::persistence::pool::criar_pool;
use accurit_backend::presentation::auth::AuthState;
use accurit_backend::presentation::routes;
use accurit_backend::presentation::state::AppState;

use common::{BancoTemporario, configuracao_com_extras};

const SENHA_PORTAL: &str = "senha-portal-9";
const CPF_OPERADOR: &str = "52998224725";
const TOKEN_TRIAL: &str = "token-demo-trial";

struct BancoFixo;

impl SaudeBanco for BancoFixo {
    async fn checar(&self) -> Result<(), SaudeBancoError> {
        Ok(())
    }
}

// ===== mock SERPRO in-process =====

#[derive(Default)]
struct EstadoMock {
    respostas: VecDeque<(u16, Option<Value>)>,
    capturas: Vec<Captura>,
}

#[derive(Clone, Debug)]
struct Captura {
    servico: &'static str,
    chave: String,
    autorizacao: String,
    request_id: String,
    operador: Option<String>,
}

fn capturar(estado: &Arc<Mutex<EstadoMock>>, servico: &'static str, chave: &str, header: &HeaderMap) {
    let autorizacao = header
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_owned();
    let request_id = header
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_owned();
    let operador = header
        .get("x-cpf-usuario")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_owned());

    estado.lock().expect("estado mock").capturas.push(Captura {
        servico,
        chave: chave.to_owned(),
        autorizacao,
        request_id,
        operador,
    });
}

fn responder(estado: &Arc<Mutex<EstadoMock>>) -> Response {
    let (status, corpo) = estado
        .lock()
        .expect("estado mock")
        .respostas
        .pop_front()
        .unwrap_or((404, None));
    let mut resposta = StatusCode::from_u16(status)
        .expect("status válido")
        .into_response();
    if status == 429 {
        resposta
            .headers_mut()
            .insert(header::RETRY_AFTER, "7".parse().expect("header"));
    }
    if let Some(corpo) = corpo {
        *resposta.body_mut() = Body::from(corpo.to_string());
    }
    resposta
}

async fn devedor_mock(
    State(estado): State<Arc<Mutex<EstadoMock>>>,
    header: HeaderMap,
    Path(chave): Path<String>,
) -> Response {
    capturar(&estado, "DEVEDOR", &chave, &header);
    responder(&estado)
}

async fn inscricao_mock(
    State(estado): State<Arc<Mutex<EstadoMock>>>,
    header: HeaderMap,
    Path(chave): Path<String>,
) -> Response {
    capturar(&estado, "INSCRICAO", &chave, &header);
    responder(&estado)
}

/// Sobe o mock do gateway SERPRO em `127.0.0.1:0` com as rotas de
/// consulta (`/api/v1/devedor/{chave}`, `/api/v1/inscricao/{chave}`).
struct MockSerpro {
    url_base: Url,
    estado: Arc<Mutex<EstadoMock>>,
}

impl MockSerpro {
    fn novo(respostas: &[(u16, Option<Value>)]) -> Self {
        let estado = Arc::new(Mutex::new(EstadoMock {
            respostas: respostas.iter().cloned().collect(),
            capturas: Vec::new(),
        }));

        // Bind síncrono com socket padrão (válido dentro do runtime
        // do teste); o servidor `axum` roda em task separada.
        let listener = std::net::TcpListener::bind("127.0.0.1:0")
            .expect("mock deve ouvir");
        let porta = listener.local_addr().expect("porta local").port();

        let estado_router = estado.clone();
        let app = Router::new()
            .route("/api/v1/devedor/{chave}", get(devedor_mock))
            .route("/api/v1/inscricao/{chave}", get(inscricao_mock))
            .with_state(estado_router);

        tokio::spawn(async move {
            let listener =
                tokio::net::TcpListener::from_std(listener).expect("socket do mock");
            axum::serve(listener, app)
                .await
                .expect("mock SERPRO deve servir");
        });

        Self {
            url_base: Url::parse(&format!("http://127.0.0.1:{porta}/api/v1"))
                .expect("URL base do mock"),
            estado,
        }
    }

    fn capturas(&self) -> Vec<Captura> {
        self.estado.lock().expect("estado mock").capturas.clone()
    }
}

// ===== ambiente =====

struct Ambiente {
    app: Router,
    banco: BancoTemporario,
}

fn ambiente_com(_respostas: &[(u16, Option<Value>)], ajustes: impl FnOnce(&mut Config)) -> Ambiente {
    let banco = BancoTemporario::novo();
    aplicar_migrations_com_lock(banco.conexao(), MIGRATIONS).expect("migrations devem aplicar");

    let mut config = configuracao_com_extras("", &[]);
    ajustes(&mut config);

    let pool = criar_pool(&banco.database_url).expect("pool deve ser criado");
    let auth = AuthState::novo(pool.clone(), &config).expect("auth deve inicializar");
    let estado = AppState::novo(ConsultarSaude::novo(BancoFixo), pool);
    let app = routes::construir(estado, Some(auth), &config);

    Ambiente { app, banco }
}

fn ambiente_mock(respostas: &[(u16, Option<Value>)]) -> (Ambiente, MockSerpro) {
    let mock = MockSerpro::novo(respostas);
    let url_base = mock.url_base.clone();
    let ambiente = ambiente_com(respostas, |config| {
        config.serpro.divida_ativa_trial_url = url_base;
        config.serpro.trial_bearer_token = Some(SecretString::from(TOKEN_TRIAL));
    });
    (ambiente, mock)
}

fn semear_escritorio(banco: &BancoTemporario, slug: &str) -> Uuid {
    let mut conexao = banco.conexao();
    let id = Uuid::now_v7();
    sql_query("INSERT INTO escritorios (id, slug, nome) VALUES ($1, $2, $3)")
        .bind::<diesel::sql_types::Uuid, _>(id)
        .bind::<Text, _>(slug)
        .bind::<Text, _>(format!("Escritório {slug}"))
        .execute(&mut conexao)
        .expect("escritório deve ser inserido");
    id
}

fn semear_operador(banco: &BancoTemporario, escritorio: Uuid, email: &str) {
    let mut conexao = banco.conexao();
    let config = configuracao_com_extras("", &[]);
    let argon2 = HashArgon2::novo(&config.auth).expect("argon2 deve inicializar");
    let hash = argon2.gerar_hash(SENHA_PORTAL).expect("hash deve ser gerado");

    sql_query(
        "INSERT INTO usuarios (id, escritorio_id, email, cpf, senha_hash, papel, ativo) \
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind::<diesel::sql_types::Uuid, _>(Uuid::now_v7())
    .bind::<diesel::sql_types::Uuid, _>(escritorio)
    .bind::<Text, _>(email)
    .bind::<Text, _>(CPF_OPERADOR)
    .bind::<Text, _>(hash)
    .bind::<Text, _>("OPERADOR")
    .bind::<diesel::sql_types::Bool, _>(true)
    .execute(&mut conexao)
    .expect("usuário deve ser inserido");
}

fn apontar_producao(banco: &BancoTemporario, escritorio: Uuid) {
    let mut conexao = banco.conexao();
    sql_query("UPDATE escritorios SET modo_trial_serpro = false WHERE id = $1")
        .bind::<diesel::sql_types::Uuid, _>(escritorio)
        .execute(&mut conexao)
        .expect("modo trial deve ser alterado");
}

// ===== utilitários HTTP =====

fn pedido(
    metodo: Method,
    caminho: &str,
    token: &str,
    extras: &[(&str, &str)],
) -> Request<Body> {
    let mut construtor = Request::builder()
        .method(metodo)
        .uri(caminho)
        .header(header::AUTHORIZATION, format!("Bearer {token}"));
    for (nome, valor) in extras {
        construtor = construtor.header(*nome, *valor);
    }
    construtor
        .header("x-real-ip", "10.9.0.1")
        .body(Body::empty())
        .expect("requisição válida")
}

async fn enviar(app: &Router, requisicao: Request<Body>) -> Response {
    app.clone().oneshot(requisicao).await.expect("resposta deve existir")
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

fn codigo_de_erro(corpo: &Value) -> String {
    corpo["error"]["code"]
        .as_str()
        .expect("envelope de erro")
        .to_owned()
}

async fn entrar_portal(app: &Router, escritorio: &str, email: &str) -> String {
    let resposta = enviar(
        app,
        Request::builder()
            .method(Method::POST)
            .uri("/api/v1/auth/client/login")
            .header(header::CONTENT_TYPE, "application/json")
            .header("x-real-ip", "10.9.0.1")
            .body(Body::from(
                json!({ "escritorio": escritorio, "email": email, "senha": SENHA_PORTAL })
                    .to_string(),
            ))
            .expect("login válido"),
    )
    .await;
    assert_eq!(resposta.status(), StatusCode::OK, "login do portal");
    token_de(&corpo_json(resposta).await)
}

async fn consultar_devedor(app: &Router, token: &str, chave: &str, extras: &[(&str, &str)]) -> Response {
    enviar(
        app,
        pedido(
            Method::GET,
            &format!("/api/v1/divida-ativa/devedor/{chave}"),
            token,
            extras,
        ),
    )
    .await
}

async fn consultar_inscricao(app: &Router, token: &str, numero: &str) -> Response {
    enviar(
        app,
        pedido(
            Method::GET,
            &format!("/api/v1/divida-ativa/inscricao/{numero}"),
            token,
            &[],
        ),
    )
    .await
}

// ===== utilitários de banco =====

#[derive(QueryableByName, Debug)]
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

#[derive(QueryableByName, Debug)]
struct LinhaConsultas {
    #[diesel(sql_type = Text)]
    estado: String,
    #[diesel(sql_type = Nullable<Text>)]
    request_redigido_texto: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    response_redigido_texto: Option<String>,
    #[diesel(sql_type = Text)]
    ambiente: String,
    #[diesel(sql_type = Nullable<Text>)]
    fonte_consulta: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    protocolo: Option<String>,
    #[diesel(sql_type = Nullable<BigInt>)]
    status_code: Option<i64>,
}

fn consultas(banco: &BancoTemporario) -> Vec<LinhaConsultas> {
    sql_query(
        "SELECT estado, request_redigido::text AS request_redigido_texto, \
         response_redigido::text AS response_redigido_texto, ambiente, \
         fonte_consulta, protocolo, status_code \
         FROM consultas_fiscais ORDER BY criado_em ASC",
    )
    .load::<LinhaConsultas>(&mut banco.conexao())
    .expect("linhas de consultas")
}

fn registrar_consulta_antiga(banco: &BancoTemporario, escritorio: Uuid) {
    let mut conexao = banco.conexao();
    sql_query(
        "INSERT INTO consultas_fiscais \
         (id, escritorio_id, produto, servico, verbo, ambiente, estado, criado_em) \
         VALUES ($1, $2, 'DIVIDA_ATIVA', 'DEVEDOR', 'GET', 'TRIAL', 'SUCESSO', \
         NOW() - INTERVAL '120 days')",
    )
    .bind::<diesel::sql_types::Uuid, _>(Uuid::now_v7())
    .bind::<diesel::sql_types::Uuid, _>(escritorio)
    .execute(&mut conexao)
    .expect("consulta antiga deve ser inserida");
}

// ===== payloads =====

fn corpo_sucesso_devedor() -> Value {
    json!([{
        "numero_inscricao": "90 6 17 000964-19",
        "valor_total_consolidado_moeda": "4.457,56",
        "situacao_descricao": "DEVEDOR CONCILIADO EM DIVIDA ATIVA",
        "nome_devedor": "EMPRESA TESTE LTDA",
        "tipo_devedor": "CNPJ",
        "cpf_cnpj": "52998224725",
        "codigo_sida": "1000"
    }])
}

fn corpo_sucesso_inscricao() -> Value {
    json!({
        "inscricao": "90 6 17 000964-19",
        "processoAdministrativo": "46206 099222/99-01",
        "situacao": "INSCRITO",
        "numeroPFNResponsavel": "1000",
        "nomePFNResponsavel": "PRIMEIRA REGIAO",
        "indicadorPrescricaoSV8": "NAO",
        "valorTotalInscritoMoeda": "805,06",
        "valorTotalConsolidadoMoeda": "4.457,56",
        "valorRemanescenteMoeda": "4.457,56",
        "nomeDevedor": "EMPRESA TESTE LTDA",
        "tipoDevedor": "CNPJ",
        "nomeUnidade": "UNIDADE 1000",
        "codigoSistemaOrigem": "M",
        "descricaoSistemaOrigem": "Manual"
    })
}

// ===== testes =====

#[tokio::test]
async fn devedor_sucesso_com_historico_e_persistencia() {
    let (ambiente, _mock) = ambiente_mock(&[
        (200, Some(corpo_sucesso_devedor())),
    ]);
    let escritorio = semear_escritorio(&ambiente.banco, "alfa");
    semear_operador(&ambiente.banco, escritorio, "operador@alfa.test");
    let token = entrar_portal(&ambiente.app, "alfa", "operador@alfa.test").await;

    let resposta =
        consultar_devedor(&ambiente.app, &token, "52998224725", &[]).await;
    assert_eq!(resposta.status(), StatusCode::OK);

    let corpo = corpo_json(resposta).await;
    assert_eq!(corpo["ambiente"], "TRIAL");
    assert_eq!(corpo["estado"], "SUCESSO");
    assert_eq!(corpo["fonteConsulta"], "CHAVE_CONTRIBUINTE");
    assert_eq!(corpo["chaveMascarada"], "***.***.247-25");
    assert_eq!(corpo["tentativas"], 1);
    assert_eq!(corpo["inscricoes"][0]["numeroInscricao"], "90 6 17 000964-19");
    assert_eq!(corpo["inscricoes"][0]["valorTotalConsolidado"], "4457.56");
    assert_eq!(corpo["inscricoes"][0]["cpfCnpj"], "52998224725");

    let linhas = consultas(&ambiente.banco);
    assert_eq!(linhas.len(), 1, "uma consulta em consultas_fiscais");
    assert_eq!(linhas[0].estado, "SUCESSO");
    assert_eq!(linhas[0].ambiente, "TRIAL");
    assert_eq!(linhas[0].fonte_consulta.as_deref(), Some("CHAVE_CONTRIBUINTE"));
    assert!(linhas[0].protocolo.is_none(), "protocolo NULL");
    let request = linhas[0].request_redigido_texto.as_deref().unwrap_or_default();
    let response = linhas[0].response_redigido_texto.as_deref().unwrap_or_default();
    assert!(
        request.contains("247-25"),
        "request redigido só tem chave mascarada"
    );
    assert!(
        !request.contains("52998224725"),
        "não pode ter documento completo"
    );
    assert!(
        response.contains(r#""quantidade":1"#),
        "response redigido é só a contagem"
    );
    assert!(
        !response.contains("4457"),
        "response redigido não tem valor"
    );
}

#[tokio::test]
async fn devedor_200_nulo_e_sem_pendencias() {
    let (ambiente, _mock) = ambiente_mock(&[(200, Some(Value::Null))]);
    let escritorio = semear_escritorio(&ambiente.banco, "alfa");
    semear_operador(&ambiente.banco, escritorio, "operador@alfa.test");
    let token = entrar_portal(&ambiente.app, "alfa", "operador@alfa.test").await;

    let resposta = consultar_devedor(&ambiente.app, &token, "52998224725", &[]).await;
    assert_eq!(resposta.status(), StatusCode::OK);
    let corpo = corpo_json(resposta).await;
    assert_eq!(corpo["estado"], "SEM_PENDENCIAS");
    assert!(corpo["inscricoes"].as_array().expect("lista").is_empty());
}

#[tokio::test]
async fn devedor_404_e_estado_historico() {
    let (ambiente, _mock) = ambiente_mock(&[(404, None)]);
    let escritorio = semear_escritorio(&ambiente.banco, "alfa");
    semear_operador(&ambiente.banco, escritorio, "operador@alfa.test");
    let token = entrar_portal(&ambiente.app, "alfa", "operador@alfa.test").await;

    let resposta = consultar_devedor(&ambiente.app, &token, "11111111111", &[]).await;
    assert_eq!(resposta.status(), StatusCode::NOT_FOUND);
    let corpo = corpo_json(resposta).await;
    assert_eq!(codigo_de_erro(&corpo), "SERPRO_NAO_ENCONTRADO");

    let linhas = consultas(&ambiente.banco);
    assert_eq!(linhas.len(), 1, "consultas com 404 também são registradas");
    assert_eq!(linhas[0].estado, "NAO_ENCONTRADO");
}

#[tokio::test]
async fn devedor_429_ecoa_retry_after() {
    let (ambiente, _mock) = ambiente_mock(&[(429, None)]);
    let escritorio = semear_escritorio(&ambiente.banco, "alfa");
    semear_operador(&ambiente.banco, escritorio, "operador@alfa.test");
    let token = entrar_portal(&ambiente.app, "alfa", "operador@alfa.test").await;

    let resposta = consultar_devedor(&ambiente.app, &token, "52998224725", &[]).await;
    assert_eq!(resposta.status(), StatusCode::TOO_MANY_REQUESTS);
    let retry_after = resposta
        .headers()
        .get(header::RETRY_AFTER)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_owned())
        .expect("Retry-After presente");
    assert_eq!(retry_after, "7");

    let corpo = corpo_json(resposta).await;
    assert_eq!(codigo_de_erro(&corpo), "SERPRO_LIMITE_EXCEDIDO");
    assert!(corpo["error"]["message"].as_str().unwrap().len() > 5);

    let linhas = consultas(&ambiente.banco);
    assert_eq!(linhas.len(), 1);
    assert_eq!(linhas[0].estado, "LIMITE_EXCEDIDO");
    assert_eq!(linhas[0].status_code, Some(429));
}

#[tokio::test]
async fn devedor_401_sem_renovacao_em_trial_ignora_upstream() {
    let (ambiente, _mock) = ambiente_mock(&[(401, None)]);
    let escritorio = semear_escritorio(&ambiente.banco, "alfa");
    semear_operador(&ambiente.banco, escritorio, "operador@alfa.test");
    let token = entrar_portal(&ambiente.app, "alfa", "operador@alfa.test").await;

    let resposta = consultar_devedor(&ambiente.app, &token, "52998224725", &[]).await;
    assert_eq!(resposta.status(), StatusCode::SERVICE_UNAVAILABLE);
    let corpo = corpo_json(resposta).await;
    assert_eq!(codigo_de_erro(&corpo), "SERPRO_NAO_AUTORIZADO");
}

#[tokio::test]
async fn devedor_5xx_e_indisponivel() {
    let (ambiente, _mock) = ambiente_mock(&[(500, None)]);
    let escritorio = semear_escritorio(&ambiente.banco, "alfa");
    semear_operador(&ambiente.banco, escritorio, "operador@alfa.test");
    let token = entrar_portal(&ambiente.app, "alfa", "operador@alfa.test").await;

    let resposta = consultar_devedor(&ambiente.app, &token, "52998224725", &[]).await;
    assert_eq!(resposta.status(), StatusCode::SERVICE_UNAVAILABLE);
    let corpo = corpo_json(resposta).await;
    assert_eq!(codigo_de_erro(&corpo), "SERPRO_INDISPONIVEL");
}

#[tokio::test]
async fn devedor_504_e_indeterminado() {
    let (ambiente, _mock) = ambiente_mock(&[(504, None)]);
    let escritorio = semear_escritorio(&ambiente.banco, "alfa");
    semear_operador(&ambiente.banco, escritorio, "operador@alfa.test");
    let token = entrar_portal(&ambiente.app, "alfa", "operador@alfa.test").await;

    let resposta = consultar_devedor(&ambiente.app, &token, "52998224725", &[]).await;
    assert_eq!(resposta.status(), StatusCode::GATEWAY_TIMEOUT);
    let corpo = corpo_json(resposta).await;
    assert_eq!(codigo_de_erro(&corpo), "SERPRO_INDETERMINADO");
}

#[tokio::test]
async fn devedor_200_fora_de_contrato_e_502() {
    let (ambiente, _mock) = ambiente_mock(&[(200, Some(json!([1])))]);
    let escritorio = semear_escritorio(&ambiente.banco, "alfa");
    semear_operador(&ambiente.banco, escritorio, "operador@alfa.test");
    let token = entrar_portal(&ambiente.app, "alfa", "operador@alfa.test").await;

    let resposta = consultar_devedor(&ambiente.app, &token, "52998224725", &[]).await;
    assert_eq!(resposta.status(), StatusCode::BAD_GATEWAY);
    let corpo = corpo_json(resposta).await;
    assert_eq!(codigo_de_erro(&corpo), "SERPRO_RESPOSTA_INVALIDA");
}

#[tokio::test]
async fn parametro_invalido_nao_bate_no_serpro() {
    let (ambiente, _mock) = ambiente_mock(&[]);
    let escritorio = semear_escritorio(&ambiente.banco, "alfa");
    semear_operador(&ambiente.banco, escritorio, "operador@alfa.test");
    let token = entrar_portal(&ambiente.app, "alfa", "operador@alfa.test").await;

    let resposta = consultar_devedor(&ambiente.app, &token, "abc", &[]).await;
    assert_eq!(resposta.status(), StatusCode::BAD_REQUEST);
    let corpo = corpo_json(resposta).await;
    assert_eq!(codigo_de_erro(&corpo), "PARAMETRO_INVALIDO");

    assert_eq!(
        contar(&ambiente.banco, "SELECT COUNT(*) AS total FROM consultas_fiscais"),
        0,
        "chave inválida não chega ao SERPRO nem gera histórico"
    );
}

#[tokio::test]
async fn inscricao_sucesso_com_detalhe_completo() {
    let (ambiente, _mock) = ambiente_mock(&[
        (200, Some(corpo_sucesso_inscricao())),
    ]);
    let escritorio = semear_escritorio(&ambiente.banco, "alfa");
    semear_operador(&ambiente.banco, escritorio, "operador@alfa.test");
    let token = entrar_portal(&ambiente.app, "alfa", "operador@alfa.test").await;

    let resposta =
        consultar_inscricao(&ambiente.app, &token, "9061700096419").await;
    assert_eq!(resposta.status(), StatusCode::OK);

    let corpo = corpo_json(resposta).await;
    assert_eq!(corpo["ambiente"], "TRIAL");
    assert_eq!(corpo["estado"], "SUCESSO");
    assert_eq!(corpo["chaveMascarada"], "906*******419");
    assert_eq!(corpo["detalhe"]["inscricao"], "90 6 17 000964-19");
    assert_eq!(corpo["detalhe"]["inscricaoDigitos"], "9061700096419");
    assert_eq!(corpo["detalhe"]["prescrito"], false);
    assert_eq!(corpo["detalhe"]["valorInscrito"], "805.06");
    assert_eq!(corpo["detalhe"]["sistemaOrigem"]["codigo"], "M");

    let capturas = _mock.capturas();
    assert_eq!(capturas.len(), 1);
    assert_eq!(capturas[0].servico, "INSCRICAO");
    assert_eq!(capturas[0].chave, "9061700096419");
    assert_eq!(capturas[0].autorizacao, format!("Bearer {TOKEN_TRIAL}"));
    assert!(!capturas[0].request_id.is_empty(), "x-request-id enviado");
}

#[tokio::test]
async fn inscricao_200_array_vazio_e_resposta_invalida() {
    let (ambiente, _mock) = ambiente_mock(&[(200, Some(json!([])))]);
    let escritorio = semear_escritorio(&ambiente.banco, "alfa");
    semear_operador(&ambiente.banco, escritorio, "operador@alfa.test");
    let token = entrar_portal(&ambiente.app, "alfa", "operador@alfa.test").await;

    let resposta = consultar_inscricao(&ambiente.app, &token, "9061700096419").await;
    assert_eq!(resposta.status(), StatusCode::BAD_GATEWAY);
    let corpo = corpo_json(resposta).await;
    assert_eq!(codigo_de_erro(&corpo), "SERPRO_RESPOSTA_INVALIDA");
}

#[tokio::test]
async fn header_operador_usa_cpf_do_operador_e_ignora_spoof() {
    let (ambiente, _mock) = ambiente_mock(&[
        (200, Some(corpo_sucesso_devedor())),
    ]);
    let escritorio = semear_escritorio(&ambiente.banco, "alfa");
    semear_operador(&ambiente.banco, escritorio, "operador@alfa.test");
    let token = entrar_portal(&ambiente.app, "alfa", "operador@alfa.test").await;

    let resposta = consultar_devedor(
        &ambiente.app,
        &token,
        "52998224725",
        &[("x-cpf-usuario", "11111111111")],
    )
    .await;
    assert_eq!(resposta.status(), StatusCode::OK);

    let capturas = _mock.capturas();
    assert_eq!(capturas.len(), 1);
    assert_eq!(
        capturas[0].operador.as_deref(),
        Some(CPF_OPERADOR),
        "header x-cpf-usuario vem do catálogo, não do cliente"
    );
}

#[tokio::test]
async fn historico_isola_por_escritorio() {
    let (ambiente, _mock) = ambiente_mock(&[
        (200, Some(corpo_sucesso_devedor())),
    ]);
    let _escritorio_a = semear_escritorio(&ambiente.banco, "alfa");
    let escritorio_b = semear_escritorio(&ambiente.banco, "bravo");
    semear_operador(&ambiente.banco, escritorio_b, "operador@bravo.test");
    let token_b = entrar_portal(&ambiente.app, "bravo", "operador@bravo.test").await;

    let resposta = consultar_devedor(&ambiente.app, &token_b, "52998224725", &[]).await;
    assert_eq!(resposta.status(), StatusCode::OK);

    let alfa = contar(
        &ambiente.banco,
        "SELECT COUNT(*) AS total FROM consultas_fiscais \
         WHERE escritorio_id = (SELECT id FROM escritorios WHERE slug = 'alfa')",
    );
    let bravo = contar(
        &ambiente.banco,
        "SELECT COUNT(*) AS total FROM consultas_fiscais \
         WHERE escritorio_id = (SELECT id FROM escritorios WHERE slug = 'bravo')",
    );
    assert_eq!(bravo, 1, "consulta de B fica em B");
    assert_eq!(alfa, 0, "A não vê consulta de B");
}

#[tokio::test]
async fn producao_divida_ativa_e_bloqueada() {
    let mock = MockSerpro::novo(&[]);
    let url_base = mock.url_base.clone();
    let ambiente = ambiente_com(&[], |config| {
        config.serpro.divida_ativa_trial_url = url_base;
        config.serpro.trial_bearer_token = Some(SecretString::from(TOKEN_TRIAL));
    });
    let escritorio = semear_escritorio(&ambiente.banco, "alfa");
    semear_operador(&ambiente.banco, escritorio, "operador@alfa.test");
    apontar_producao(&ambiente.banco, escritorio);
    let token = entrar_portal(&ambiente.app, "alfa", "operador@alfa.test").await;

    let resposta = consultar_devedor(&ambiente.app, &token, "52998224725", &[]).await;
    assert_eq!(resposta.status(), StatusCode::SERVICE_UNAVAILABLE);
    let corpo = corpo_json(resposta).await;
    assert_eq!(codigo_de_erro(&corpo), "SERPRO_PROD_NOT_CONFIGURED");

    assert_eq!(
        contar(&ambiente.banco, "SELECT COUNT(*) AS total FROM consultas_fiscais"),
        1,
        "bloqueio também é registrado"
    );
}

#[tokio::test]
async fn limite_local_por_escritorio_e_429() {
    let mock = MockSerpro::novo(&[(200, Some(corpo_sucesso_devedor()))]);
    let url_base = mock.url_base.clone();
    let ambiente = ambiente_com(&[], |config| {
        config.serpro.divida_ativa_trial_url = url_base;
        config.serpro.trial_bearer_token = Some(SecretString::from(TOKEN_TRIAL));
        config.serpro.rate_limit_consultas_por_segundo = 1;
    });
    let escritorio = semear_escritorio(&ambiente.banco, "alfa");
    semear_operador(&ambiente.banco, escritorio, "operador@alfa.test");
    let token = entrar_portal(&ambiente.app, "alfa", "operador@alfa.test").await;

    let primeira = consultar_devedor(&ambiente.app, &token, "52998224725", &[]).await;
    assert_eq!(primeira.status(), StatusCode::OK);

    let segunda = consultar_devedor(&ambiente.app, &token, "52998224725", &[]).await;
    assert_eq!(segunda.status(), StatusCode::TOO_MANY_REQUESTS);
    let retry_after = segunda
        .headers()
        .get(header::RETRY_AFTER)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_owned())
        .expect("Retry-After do limite local");
    assert_eq!(retry_after, "1");
    let corpo = corpo_json(segunda).await;
    assert_eq!(codigo_de_erro(&corpo), "LIMITE_REQUISICOES");
}

#[tokio::test]
async fn flag_fallback_on_nao_cria_segunda_tentativa_sem_codigo() {
    let mock = MockSerpro::novo(&[(404, None)]);
    let url_base = mock.url_base.clone();
    let ambiente = ambiente_com(&[], |config| {
        config.serpro.divida_ativa_trial_url = url_base;
        config.serpro.trial_bearer_token = Some(SecretString::from(TOKEN_TRIAL));
        config.serpro.divida_ativa_fallback_codigo_contribuinte = true;
    });
    // sem semente de catálogo: nenhum código de contribuinte disponível
    let escritorio = semear_escritorio(&ambiente.banco, "alfa");
    semear_operador(&ambiente.banco, escritorio, "operador@alfa.test");
    let token = entrar_portal(&ambiente.app, "alfa", "operador@alfa.test").await;

    let resposta = consultar_devedor(&ambiente.app, &token, "11111111111", &[]).await;
    assert_eq!(resposta.status(), StatusCode::NOT_FOUND);

    assert_eq!(
        mock.capturas().len(),
        1,
        "sem código alternativo não há segunda tentativa (D2)"
    );
}

#[tokio::test]
async fn expurgo_apaga_somente_acima_da_retencao() {
    let (ambiente, _mock) = ambiente_mock(&[]);
    let escritorio = semear_escritorio(&ambiente.banco, "alfa");
    registrar_consulta_antiga(&ambiente.banco, escritorio);
    registrar_consulta_antiga(&ambiente.banco, escritorio);

    // consulta recente
    let mut conexao = ambiente.banco.conexao();
    sql_query(
        "INSERT INTO consultas_fiscais \
         (id, escritorio_id, produto, servico, verbo, ambiente, estado, criado_em) \
         VALUES ($1, $2, 'DIVIDA_ATIVA', 'DEVEDOR', 'GET', 'TRIAL', 'SUCESSO', NOW())",
    )
    .bind::<diesel::sql_types::Uuid, _>(Uuid::now_v7())
    .bind::<diesel::sql_types::Uuid, _>(escritorio)
    .execute(&mut conexao)
    .expect("consulta recente deve ser inserida");
    drop(conexao);

    let repositorio =
        ConsultasFiscaisRepositorioDiesel::novo(criar_pool(&ambiente.banco.database_url)
            .expect("pool do expurgo"));
    let apagados = repositorio
        .expurgar_antes_de(chrono::Utc::now() - chrono::Duration::days(90))
        .expect("expurgo deve funcionar");
    assert_eq!(apagados, 2, "apaga as duas antigas (120 dias)");

    assert_eq!(
        contar(&ambiente.banco, "SELECT COUNT(*) AS total FROM consultas_fiscais"),
        1,
        "a recente (NOW) sobrevive"
    );
}
