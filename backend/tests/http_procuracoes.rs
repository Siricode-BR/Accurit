//! Testes de integraÃ§Ã£o HTTP da FASE A (rotina fiscal): catÃ¡logo
//! configurÃ¡vel de serviÃ§os do Integra Contador (ADR-0013, backoffice),
//! validaÃ§Ã£o/revalidaÃ§Ã£o de procuraÃ§Ã£o pelo portal com mock SERPRO
//! in-process persistindo em `procuracoes` (AGGREGATE, P7), TTL de cache
//! (`SERPRO_PROCURACAO_CACHE_TTL_SECS`), mapeamento de resposta
//! (CONCLUIDOâ†’VALIDA, FALHA_AUTORIZACAOâ†’NAO_AUTORIZADA,
//! FALHA_TRANSPORTEâ†’INDETERMINADA), isolamento por escritÃ³rio e estado
//! por serviÃ§o (NAO_EXIGIDA/AUTORIZADA/CONSULTADA/PENDENTE).

mod common;

use axum::Router;
use axum::body::{Body, Bytes, to_bytes};
use axum::extract::State;
use axum::http::{HeaderMap, Method, Request, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use diesel::QueryableByName;
use diesel::RunQueryDsl;
use diesel::sql_query;
use diesel::sql_types::{BigInt, Nullable, Text, Uuid as SqlUuid};
use secrecy::SecretString;
use serde_json::{Value, json};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use tower::ServiceExt;
use url::Url;
use uuid::Uuid;

use accurit_backend::application::ports::{SaudeBanco, SaudeBancoError};
use accurit_backend::application::saude::ConsultarSaude;
use accurit_backend::infrastructure::auth::HashArgon2;
use accurit_backend::infrastructure::config::Config;
use accurit_backend::infrastructure::persistence::migrations::{
    MIGRATIONS, aplicar_migrations_com_lock,
};
use accurit_backend::infrastructure::persistence::pool::criar_pool;
use accurit_backend::presentation::auth::AuthState;
use accurit_backend::presentation::routes;
use accurit_backend::presentation::state::AppState;

use common::{BancoTemporario, configuracao_com_extras};

const SENHA_PORTAL: &str = "senha-portal-9";
const SENHA_GESTOR: &str = "senha-forte-123";
const EMAIL_GESTOR: &str = "gestor@accurit.test";
const CPF_OPERADOR: &str = "52998224725";
const CNPJ_ESCRITORIO: &str = "11222333000181";
const CNPJ_BETA: &str = "12345678000195";
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
    chamadas_consultar: Vec<Value>,
}

fn responder(estado: &Arc<Mutex<EstadoMock>>) -> Response {
    let (status, corpo) = estado
        .lock()
        .expect("estado mock")
        .respostas
        .pop_front()
        .unwrap_or((404, None));
    let mut resposta = StatusCode::from_u16(status)
        .expect("status vÃ¡lido")
        .into_response();
    if let Some(corpo) = corpo {
        *resposta.body_mut() = Body::from(corpo.to_string());
    }
    resposta
}

async fn consultar_mock(
    State(estado): State<Arc<Mutex<EstadoMock>>>,
    _header: HeaderMap,
    corpo: Bytes,
) -> Response {
    let corpo = serde_json::from_slice(&corpo).expect("envelope JSON do Integra Contador");
    estado
        .lock()
        .expect("estado mock")
        .chamadas_consultar
        .push(corpo);
    responder(&estado)
}

/// Resposta 200 completa do OBTERPROCURACAO41: dados tolerados (P7).
fn resposta_procuracoes(status: u16) -> Value {
    json!({
        "status": status,
        "mensagens": [{"codigo": "897", "tipo": null, "texto": "requisiÃ§Ã£o processada", "descricao": null}],
        "dados": format!(
            r#"{{"procuracoes":[], "outorgante":"{}", "outorgado":"{}"}}"#,
            "52998224725", CNPJ_ESCRITORIO
        )
    })
}

struct MockSerpro {
    url_base: Url,
    estado: Arc<Mutex<EstadoMock>>,
}

impl MockSerpro {
    #[allow(non_snake_case)]
    fn novo(respostas: &[(u16, Option<Value>)]) -> Self {
        let estado = Arc::new(Mutex::new(EstadoMock {
            respostas: respostas.iter().cloned().collect(),
            chamadas_consultar: Vec::new(),
        }));

        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("mock deve ouvir");
        listener
            .set_nonblocking(true)
            .expect("socket do mock nÃ£o bloqueante");
        let porta = listener.local_addr().expect("porta local").port();

        let app = Router::new()
            .route("/api/v1/Consultar", post(consultar_mock))
            .with_state(estado.clone());

        tokio::spawn(async move {
            let listener = tokio::net::TcpListener::from_std(listener).expect("socket do mock");
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

    fn chamadas(&self) -> usize {
        self.estado
            .lock()
            .expect("estado mock")
            .chamadas_consultar
            .len()
    }
}

// ===== ambiente =====

struct Ambiente {
    app: Router,
    banco: BancoTemporario,
}

fn ambiente_com(ajustes: impl FnOnce(&mut Config)) -> Ambiente {
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

fn ambiente_mock(
    respostas: &[(u16, Option<Value>)],
    ajustes: impl FnOnce(&mut Config),
) -> (Ambiente, MockSerpro) {
    let mock = MockSerpro::novo(respostas);
    let url_base = mock.url_base.clone();
    let ambiente = ambiente_com(|config| {
        config.serpro.integra_trial_url = url_base;
        config.serpro.trial_bearer_token = Some(SecretString::from(TOKEN_TRIAL));
        config.serpro.trial_mock = false;
        ajustes(config);
    });
    (ambiente, mock)
}

fn ambiente_default(respostas: &[(u16, Option<Value>)]) -> (Ambiente, MockSerpro) {
    ambiente_mock(respostas, |_| {})
}

// ===== sementes =====

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
        .bind::<Text, _>(format!("EscritÃ³rio {slug}"))
        .execute(&mut conexao)
        .expect("escritÃ³rio deve ser inserido");
    id
}

fn semear_operador(banco: &BancoTemporario, escritorio: Uuid, email: &str) {
    let mut conexao = banco.conexao();
    let config = configuracao_com_extras("", &[]);
    let argon2 = HashArgon2::novo(&config.auth).expect("argon2 deve inicializar");
    let hash = argon2
        .gerar_hash(SENHA_PORTAL)
        .expect("hash deve ser gerado");

    sql_query(
        "INSERT INTO usuarios (id, escritorio_id, email, cpf, senha_hash, papel, ativo) \
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind::<SqlUuid, _>(Uuid::now_v7())
    .bind::<SqlUuid, _>(escritorio)
    .bind::<Text, _>(email)
    .bind::<Text, _>(CPF_OPERADOR)
    .bind::<Text, _>(hash)
    .bind::<Text, _>("OPERADOR")
    .bind::<diesel::sql_types::Bool, _>(true)
    .execute(&mut conexao)
    .expect("usuÃ¡rio deve ser inserido");
}

fn semear_gestor(banco: &BancoTemporario) {
    let mut conexao = banco.conexao();
    let config = configuracao_com_extras("", &[]);
    let argon2 = HashArgon2::novo(&config.auth).expect("argon2 deve inicializar");
    let hash = argon2
        .gerar_hash(SENHA_GESTOR)
        .expect("hash deve ser gerado");

    sql_query(
        "INSERT INTO administradores (id, nome, email, cpf, senha_hash, ativo) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind::<SqlUuid, _>(Uuid::now_v7())
    .bind::<Text, _>("Gestor de teste")
    .bind::<Text, _>(EMAIL_GESTOR)
    .bind::<Text, _>(CPF_OPERADOR)
    .bind::<Text, _>(hash)
    .bind::<diesel::sql_types::Bool, _>(true)
    .execute(&mut conexao)
    .expect("administrador deve ser inserido");
}

/// EscritÃ³rio com CNPJ (autor do pedido) e cliente PF (outorgante).
/// O CNPJ Ã© Ãºnico no banco: cada escritÃ³rio usa o seu.
fn semear_escritorio_com_cliente(banco: &BancoTemporario, slug: &str, cnpj: &str) -> (Uuid, Uuid) {
    let escritorio = semear_escritorio(banco, slug);
    semear_operador(banco, escritorio, &format!("operador@{slug}.test"));
    let mut conexao = banco.conexao();
    sql_query("UPDATE escritorios SET cnpj = $1 WHERE id = $2")
        .bind::<Text, _>(cnpj)
        .bind::<SqlUuid, _>(escritorio)
        .execute(&mut conexao)
        .expect("CNPJ do escritÃ³rio deve ser salvo");
    let cliente = Uuid::now_v7();
    sql_query(
        "INSERT INTO clientes (id, escritorio_id, tipo, cpf_cnpj, nome_razao_social) \
         VALUES ($1, $2, 'PF', $3, $4)",
    )
    .bind::<SqlUuid, _>(cliente)
    .bind::<SqlUuid, _>(escritorio)
    .bind::<Text, _>(CPF_OPERADOR)
    .bind::<Text, _>("Cliente de teste")
    .execute(&mut conexao)
    .expect("cliente deve ser inserido");
    (escritorio, cliente)
}

// ===== utilitÃ¡rios HTTP =====

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
        .expect("requisiÃ§Ã£o vÃ¡lida")
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

async fn entrar_portal(app: &Router, email: &str) -> Value {
    let resposta = enviar(
        app,
        pedido(
            Method::POST,
            "/api/v1/auth/client/login",
            Some(json!({
                "email": email,
                "senha": SENHA_PORTAL,
            })),
            None,
        ),
    )
    .await;
    assert_eq!(resposta.status(), StatusCode::OK, "login do portal");
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
                "senha": SENHA_GESTOR,
            })),
            None,
        ),
    )
    .await;
    assert_eq!(resposta.status(), StatusCode::OK, "login do gestor");
    corpo_json(resposta).await
}

#[derive(QueryableByName)]
struct RegistroProcuracaoRow {
    #[diesel(sql_type = Nullable<Text>)]
    status: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    payload_redigido: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    cliente_id: Option<String>,
}

fn registro_agregado(banco: &BancoTemporario, escritorio: Uuid) -> RegistroProcuracaoRow {
    sql_query(
        "SELECT status, payload_redigido::text AS payload_redigido, \
                cliente_id::text AS cliente_id \
         FROM procuracoes \
         WHERE escritorio_id = $1 AND codigo_servico IS NULL",
    )
    .bind::<SqlUuid, _>(escritorio)
    .get_result::<RegistroProcuracaoRow>(&mut banco.conexao())
    .expect("registro agregado deve existir")
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

async fn validar(ambiente: &Ambiente, token: &str, cliente: Uuid, caminho: &str) -> Value {
    let resposta = enviar(
        &ambiente.app,
        pedido(
            Method::POST,
            &format!("/api/v1/serpro/integra-contador/procuracoes/{caminho}"),
            Some(json!({ "clienteId": cliente })),
            Some(token),
        ),
    )
    .await;
    assert_eq!(resposta.status(), StatusCode::OK, "validar deve responder");
    corpo_json(resposta).await
}

// ===== testes =====

#[tokio::test]
async fn validar_persiste_agregado_e_estado_por_servico() {
    let (ambiente, _mock) = ambiente_default(&[(200, Some(resposta_procuracoes(200)))]);
    let (escritorio, cliente) =
        semear_escritorio_com_cliente(&ambiente.banco, "alfa", CNPJ_ESCRITORIO);
    let token = token_de(&entrar_portal(&ambiente.app, "operador@alfa.test").await);

    let resultado = validar(&ambiente, &token, cliente, "validar").await;
    assert_eq!(resultado["fonte"], "SERPRO");
    assert_eq!(resultado["registro"]["status"], "VALIDA");
    assert_eq!(resultado["registro"]["codigoServico"], Value::Null);
    assert!(resultado["registro"]["payloadRedigido"].is_null());
    assert!(resultado["registro"]["verificadoEm"].is_string());

    let registro = registro_agregado(&ambiente.banco, escritorio);
    assert_eq!(
        registro.status.as_deref(),
        Some("VALIDA"),
        "CONCLUIDO â†’ VALIDA"
    );
    assert_eq!(
        registro.cliente_id.as_deref(),
        Some(cliente.to_string().as_str()),
        "registro vinculado ao cliente"
    );
    let payload = registro.payload_redigido.expect("payload tolerado");
    assert!(
        payload.contains("procuracoes"),
        "payload_redigido carrega o corpo cru do SERPRO: {payload}"
    );

    // CatÃ¡logo: estado por serviÃ§o, fail-closed (P7: sem registro
    // per-serviÃ§o, serviÃ§os exigentes ficam CONSULTADA; nÃ£o exigentes
    // ficam NAO_EXIGIDA).
    let resposta = enviar(
        &ambiente.app,
        pedido(
            Method::GET,
            &format!("/api/v1/serpro/integra-contador/procuracoes?clienteId={cliente}"),
            None,
            Some(&token),
        ),
    )
    .await;
    assert_eq!(resposta.status(), StatusCode::OK);
    let estado = corpo_json(resposta).await;
    assert_eq!(estado["contribuinteCpfCnpj"], CPF_OPERADOR);
    // O DTO normaliza VALIDA -> AUTORIZADA na resposta da API; o registro
    // persistido acima mantem VALIDA internamente.
    assert_eq!(estado["status"], "AUTORIZADA");
    let servicos = estado["servicos"]
        .as_array()
        .expect("serviÃ§os listados")
        .to_vec();
    assert!(!servicos.is_empty(), "catÃ¡logo configurÃ¡vel vem do banco");
    for servico in &servicos {
        let exigida = servico["habilitado"].as_bool().unwrap_or(false);
        let estado_str = servico["estado"].as_str().unwrap_or_default();
        if !exigida {
            continue; // estado varia conforme regra; sÃ³ garante valores vÃ¡lidos
        }
        assert!(
            matches!(
                estado_str,
                "NAO_EXIGIDA" | "AUTORIZADA" | "CONSULTADA" | "PENDENTE"
            ),
            "estado por serviÃ§o vÃ¡lido: {estado_str} em {servico:?}"
        );
    }
    let sitfis = servicos
        .iter()
        .find(|servico| servico["idServico"] == "RELATORIOSITFIS92")
        .expect("SITFIS listado");
    assert_eq!(
        sitfis["estado"], "CONSULTADA",
        "agregado vÃ¡lido recente â†’ CONSULTADA (fail-closed, sem per-serviÃ§o)"
    );
    let pgmei = servicos
        .iter()
        .find(|servico| servico["idServico"] == "GERARDASPDF21")
        .expect("PGMEI listado");
    assert_eq!(
        pgmei["estado"], "NAO_EXIGIDA",
        "PGMEI nÃ£o exige procuraÃ§Ã£o no catÃ¡logo"
    );
}

#[tokio::test]
async fn validar_reusa_cache_dentro_do_ttl_e_revalidar_forca_nova_chamada() {
    let (ambiente, mock) = ambiente_default(&[
        (200, Some(resposta_procuracoes(200))),
        (200, Some(resposta_procuracoes(200))),
    ]);
    let (_escritorio, cliente) =
        semear_escritorio_com_cliente(&ambiente.banco, "alfa", CNPJ_ESCRITORIO);
    let token = token_de(&entrar_portal(&ambiente.app, "operador@alfa.test").await);

    let primeiro = validar(&ambiente, &token, cliente, "validar").await;
    assert_eq!(primeiro["fonte"], "SERPRO");
    assert_eq!(mock.chamadas(), 1, "primeira validaÃ§Ã£o consulta o SERPRO");

    let segundo = validar(&ambiente, &token, cliente, "validar").await;
    assert_eq!(segundo["fonte"], "CACHE", "TTL responde sem nova chamada");
    assert_eq!(mock.chamadas(), 1, "nenhuma chamada extra dentro do TTL");

    let terceiro = validar(&ambiente, &token, cliente, "revalidar").await;
    assert_eq!(terceiro["fonte"], "SERPRO", "revalidar ignora o TTL");
    assert_eq!(mock.chamadas(), 2, "revalidar forÃ§a nova chamada");
}

#[tokio::test]
async fn falha_autorizacao_persiste_nao_autorizada_e_servaicos_ficam_pendentes() {
    let (ambiente, _mock) = ambiente_default(&[(
        403,
        Some(json!({
            "status": 403,
            "mensagens": [{"codigo": "841", "tipo": "ERRO", "texto": "acesso negado", "descricao": null}],
            "dados": null
        })),
    )]);
    let (_escritorio, cliente) =
        semear_escritorio_com_cliente(&ambiente.banco, "alfa", CNPJ_ESCRITORIO);
    let token = token_de(&entrar_portal(&ambiente.app, "operador@alfa.test").await);

    let resultado = validar(&ambiente, &token, cliente, "validar").await;
    assert_eq!(
        resultado["registro"]["status"], "NAO_AUTORIZADA",
        "FALHA_AUTORIZACAO â†’ NAO_AUTORIZADA"
    );

    let resposta = enviar(
        &ambiente.app,
        pedido(
            Method::GET,
            &format!("/api/v1/serpro/integra-contador/procuracoes?clienteId={cliente}"),
            None,
            Some(&token),
        ),
    )
    .await;
    let estado = corpo_json(resposta).await;
    assert_eq!(estado["status"], "NAO_AUTORIZADA");
    let sitfis = estado["servicos"]
        .as_array()
        .expect("serviÃ§os")
        .iter()
        .find(|servico| servico["idServico"] == "RELATORIOSITFIS92")
        .expect("SITFIS listado");
    assert_eq!(sitfis["estado"], "PENDENTE");
}

#[tokio::test]
async fn falha_de_transporte_persiste_indeterminada() {
    // 503 com corpo nÃ£o-parseÃ¡vel â†’ estado FALHA_TRANSPORTE com resposta
    // opaca; registro persiste INDETERMINADA e o corpo nÃ£o Ã© exposto.
    let (ambiente, _mock) = ambiente_default(&[(503, Some(json!("erro opaco")))]);
    let (_escritorio, cliente) =
        semear_escritorio_com_cliente(&ambiente.banco, "alfa", CNPJ_ESCRITORIO);
    let token = token_de(&entrar_portal(&ambiente.app, "operador@alfa.test").await);

    let resultado = validar(&ambiente, &token, cliente, "validar").await;
    assert_eq!(
        resultado["registro"]["status"], "INDETERMINADA",
        "FALHA_TRANSPORTE â†’ INDETERMINADA"
    );
}

#[tokio::test]
async fn validar_rejeita_cliente_de_outro_escritorio_ou_inexistente() {
    let (ambiente, _mock) = ambiente_default(&[]);
    let (escritorio_alfa, _cliente_alfa) =
        semear_escritorio_com_cliente(&ambiente.banco, "alfa", CNPJ_ESCRITORIO);
    let (_, cliente_beta) = semear_escritorio_com_cliente(&ambiente.banco, "beta", CNPJ_BETA);
    let token = token_de(&entrar_portal(&ambiente.app, "operador@alfa.test").await);
    let _ = escritorio_alfa;

    // IDOR: cliente de outro escritÃ³rio responde 404, nunca dados.
    let resposta = enviar(
        &ambiente.app,
        pedido(
            Method::POST,
            "/api/v1/serpro/integra-contador/procuracoes/validar",
            Some(json!({ "clienteId": cliente_beta })),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(resposta.status(), StatusCode::NOT_FOUND, "IDOR: 404");
    let corpo = corpo_json(resposta).await;
    assert_eq!(codigo_de_erro(&corpo), "NAO_ENCONTRADO");

    // Cliente inexistente tambÃ©m Ã© 404.
    let resposta = enviar(
        &ambiente.app,
        pedido(
            Method::POST,
            "/api/v1/serpro/integra-contador/procuracoes/validar",
            Some(json!({ "clienteId": Uuid::now_v7() })),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(resposta.status(), StatusCode::NOT_FOUND);
    assert_eq!(
        codigo_de_erro(&corpo_json(resposta).await),
        "NAO_ENCONTRADO"
    );
    assert_eq!(
        contar(&ambiente.banco, "SELECT COUNT(*) AS total FROM procuracoes",),
        0,
        "nenhuma persistÃªncia em tentativa com cliente estranho"
    );
}

#[tokio::test]
async fn admin_gerencia_catalogo_configuravel() {
    let (ambiente, _mock) = ambiente_default(&[]);
    semear_escritorio_com_cliente(&ambiente.banco, "alfa", CNPJ_ESCRITORIO);
    semear_gestor(&ambiente.banco);
    let gestor = token_de(&entrar_gestor(&ambiente.app).await);

    // Lista completa do catÃ¡logo configurÃ¡vel (semeado nas migrations).
    let resposta = enviar(
        &ambiente.app,
        pedido(
            Method::GET,
            "/api/v1/admin/integra-contador/catalogo",
            None,
            Some(&gestor),
        ),
    )
    .await;
    assert_eq!(resposta.status(), StatusCode::OK);
    let catalogo = corpo_json(resposta).await;
    let linhas = catalogo.as_array().expect("linhas").to_vec();
    assert_eq!(
        contar(
            &ambiente.banco,
            "SELECT COUNT(*) AS total FROM catalogo_servicos_integra",
        ),
        linhas.len() as i64
    );
    let obter = linhas
        .iter()
        .find(|linha| linha["idServico"] == "OBTERPROCURACAO41")
        .expect("OBTERPROCURACAO41 listado");
    assert_eq!(obter["habilitado"], true);
    assert_eq!(obter["versaoConfirmada"], true);
    let sitfis = linhas
        .iter()
        .find(|linha| linha["idServico"] == "RELATORIOSITFIS92")
        .expect("SITFIS listado");
    assert_eq!(
        sitfis["habilitado"], false,
        "versÃ£o pendente â†’ desligado"
    );

    // Desliga e religa o Ãºnico serviÃ§o habilitado.
    let desligar = enviar(
        &ambiente.app,
        pedido(
            Method::PATCH,
            "/api/v1/admin/integra-contador/catalogo/PROCURACOES/OBTERPROCURACAO41",
            Some(json!({ "habilitado": false })),
            Some(&gestor),
        ),
    )
    .await;
    assert_eq!(desligar.status(), StatusCode::OK);
    let linha = corpo_json(desligar).await;
    assert_eq!(linha["habilitado"], false);
    assert_eq!(
        contar(
            &ambiente.banco,
            "SELECT COUNT(*) AS total FROM catalogo_servicos_integra \
             WHERE id_servico = 'OBTERPROCURACAO41' AND habilitado = TRUE",
        ),
        0
    );

    // ServiÃ§o sem versÃ£o confirmada nÃ£o habilita (fail-closed).
    let ligar_sitfis = enviar(
        &ambiente.app,
        pedido(
            Method::PATCH,
            "/api/v1/admin/integra-contador/catalogo/SITFIS/RELATORIOSITFIS92",
            Some(json!({ "habilitado": true })),
            Some(&gestor),
        ),
    )
    .await;
    assert_eq!(
        ligar_sitfis.status(),
        StatusCode::FORBIDDEN,
        "sem versÃ£o confirmada o PATCH nÃ£o habilita"
    );

    // Contrato que nÃ£o existe no catÃ¡logo de referÃªncia Ã© 404.
    let desconhecido = enviar(
        &ambiente.app,
        pedido(
            Method::PATCH,
            "/api/v1/admin/integra-contador/catalogo/SISTEMA/DESCONHECIDO99",
            Some(json!({ "habilitado": true })),
            Some(&gestor),
        ),
    )
    .await;
    assert_eq!(desconhecido.status(), StatusCode::NOT_FOUND);
    assert_eq!(
        codigo_de_erro(&corpo_json(desconhecido).await),
        "NAO_ENCONTRADO"
    );

    // AudiÃªncia: portal nÃ£o acessa o backoffice.
    let token_portal = token_de(&entrar_portal(&ambiente.app, "operador@alfa.test").await);
    let resposta = enviar(
        &ambiente.app,
        pedido(
            Method::GET,
            "/api/v1/admin/integra-contador/catalogo",
            None,
            Some(&token_portal),
        ),
    )
    .await;
    assert_eq!(resposta.status(), StatusCode::UNAUTHORIZED);
    let corpo = corpo_json(resposta).await;
    assert_eq!(codigo_de_erro(&corpo), "AUTH_AUDIENCE_MISMATCH");

    let resposta = enviar(
        &ambiente.app,
        pedido(
            Method::GET,
            "/api/v1/admin/integra-contador/catalogo",
            None,
            None,
        ),
    )
    .await;
    assert_eq!(resposta.status(), StatusCode::UNAUTHORIZED);
}
