//! Testes de integração HTTP da Fase 3 (SERPRO): toggle de modo trial
//! por escritório no backoffice (GET/PATCH admin), leitura de ambiente
//! no portal (GET client), auditoria, invalidação de caches e isolamento
//! de audiências (SPEC §10.1/§10.2; ADR-0005).

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
use accurit_backend::application::serpro::CriptografiaSerpro;
use accurit_backend::infrastructure::auth::HashArgon2;
use accurit_backend::infrastructure::persistence::migrations::{
    MIGRATIONS, aplicar_migrations_com_lock,
};
use accurit_backend::infrastructure::persistence::pool::criar_pool;
use accurit_backend::infrastructure::serpro::SerproCriptografia;
use accurit_backend::presentation::auth::AuthState;
use accurit_backend::presentation::routes;
use accurit_backend::presentation::state::AppState;

use common::{BancoTemporario, configuracao_com_extras};

const SENHA_GESTOR: &str = "senha-forte-123";
const SENHA_PORTAL: &str = "senha-portal-9";
const EMAIL_GESTOR: &str = "gestor@accurit.test";

/// mesma chave de CERTIFICATE_MASTER_KEY do `common::configuracao_com_extras`
const CHAVE_MESTRA: &[u8] = &[7_u8; 32];

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

fn ambiente() -> Ambiente {
    let banco = BancoTemporario::novo();
    aplicar_migrations_com_lock(banco.conexao(), MIGRATIONS).expect("migrations devem aplicar");

    let config = configuracao_com_extras("", &[]);
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
struct EstadoEscritorio {
    #[diesel(sql_type = diesel::sql_types::Bool)]
    modo_trial_serpro: bool,
    #[diesel(sql_type = BigInt)]
    versao: i64,
}

#[derive(QueryableByName)]
struct Contagem {
    #[diesel(sql_type = BigInt)]
    total: i64,
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

fn id_do_escritorio(banco: &BancoTemporario, slug: &str) -> Uuid {
    sql_query("SELECT id FROM escritorios WHERE slug = $1")
        .bind::<Text, _>(slug)
        .get_result::<IdRow>(&mut banco.conexao())
        .expect("escritório deve existir")
        .id
}

fn semear_gestor(banco: &BancoTemporario) {
    let mut conexao = banco.conexao();
    let config = configuracao_com_extras("", &[]);
    let argon2 = HashArgon2::novo(&config.auth).expect("argon2 deve inicializar");
    let hash = argon2
        .gerar_hash(SENHA_GESTOR)
        .expect("hash deve ser gerado");

    sql_query(
        "INSERT INTO administradores (id, email, cpf, senha_hash, ativo) \
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind::<SqlUuid, _>(Uuid::now_v7())
    .bind::<Text, _>(EMAIL_GESTOR)
    .bind::<Text, _>("52998224725")
    .bind::<Text, _>(hash)
    .bind::<diesel::sql_types::Bool, _>(true)
    .execute(&mut conexao)
    .expect("administrador deve ser inserido");
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
    .bind::<Text, _>("52998224725")
    .bind::<Text, _>(hash)
    .bind::<Text, _>("OPERADOR")
    .bind::<diesel::sql_types::Bool, _>(true)
    .execute(&mut conexao)
    .expect("usuário deve ser inserido");
}

/// semeia credencial de produção do Integra Contador cifrada com a mesma
/// chave mestra da configuração do app (SPEC §10.1: troca exige esta
/// credencial + certificado A1 em validade)
fn semear_credencial_producao(
    banco: &BancoTemporario,
    escritorio: Uuid,
    produto: &str,
    ambiente: &str,
) {
    let cripto = SerproCriptografia::novo(CHAVE_MESTRA).expect("criptografia deve inicializar");
    let chave_cifrada = cripto
        .cifrar(b"consumer-key-prod")
        .expect("chave deve ser cifrada");
    let segredo_cifrado = cripto
        .cifrar(b"consumer-secret-prod")
        .expect("segredo deve ser cifrado");

    let mut conexao = banco.conexao();
    sql_query(
        "INSERT INTO credenciais_serpro \
         (id, escritorio_id, produto, ambiente, consumer_key_cifrado, consumer_secret_cifrado) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind::<SqlUuid, _>(Uuid::now_v7())
    .bind::<SqlUuid, _>(escritorio)
    .bind::<Text, _>(produto)
    .bind::<Text, _>(ambiente)
    .bind::<diesel::sql_types::Binary, _>(chave_cifrada)
    .bind::<diesel::sql_types::Binary, _>(segredo_cifrado)
    .execute(&mut conexao)
    .expect("credencial deve ser inserida");
}

fn semear_certificado_valido(banco: &BancoTemporario, escritorio: Uuid) {
    let agora = chrono::Utc::now();
    let mut conexao = banco.conexao();
    sql_query(
        "INSERT INTO certificados_a1 \
         (id, escritorio_id, tipo, titular_cpf_cnpj, conteudo_cifrado, \
          fingerprint_sha256, valido_de, valido_ate, ativo) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, TRUE)",
    )
    .bind::<SqlUuid, _>(Uuid::now_v7())
    .bind::<SqlUuid, _>(escritorio)
    .bind::<Text, _>("A1")
    .bind::<Text, _>("52998224725")
    .bind::<diesel::sql_types::Binary, _>(vec![1_u8, 2, 3, 4])
    .bind::<Text, _>("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
    .bind::<diesel::sql_types::Timestamptz, _>(agora - chrono::Duration::days(1))
    .bind::<diesel::sql_types::Timestamptz, _>(agora + chrono::Duration::days(30))
    .execute(&mut conexao)
    .expect("certificado deve ser inserido");
}

/// semeia cache de token do produto/ambiente indicado (o invalidador da
/// troca de modo deve apagá-los)
fn semear_token(banco: &BancoTemporario, escritorio: Uuid, tipo_token: &str) {
    let mut conexao = banco.conexao();
    sql_query(
        "INSERT INTO tokens_serpro \
         (id, escritorio_id, produto, ambiente, tipo_token, token_cifrado, expira_em) \
         VALUES ($1, $2, $3, $4, $5, $6, NOW() + INTERVAL '60 minutes')",
    )
    .bind::<SqlUuid, _>(Uuid::now_v7())
    .bind::<SqlUuid, _>(escritorio)
    .bind::<Text, _>("INTEGRA_CONTADOR")
    .bind::<Text, _>("TRIAL")
    .bind::<Text, _>(tipo_token)
    .bind::<diesel::sql_types::Binary, _>(vec![9_u8; 16])
    .execute(&mut conexao)
    .expect("token deve ser inserido");
}

fn semear_procuracao(banco: &BancoTemporario, escritorio: Uuid) {
    let mut conexao = banco.conexao();
    sql_query(
        "INSERT INTO procuracoes \
         (id, escritorio_id, ambiente, contribuinte_cpf_cnpj, procurador_cpf_cnpj, \
          codigo_servico, status, verificado_em) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())",
    )
    .bind::<SqlUuid, _>(Uuid::now_v7())
    .bind::<SqlUuid, _>(escritorio)
    .bind::<Text, _>("TRIAL")
    .bind::<Text, _>("11111111111111")
    .bind::<Text, _>("22222222222222")
    .bind::<Text, _>("00093")
    .bind::<Text, _>("ATIVO")
    .execute(&mut conexao)
    .expect("procuração deve ser inserida");
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

async fn entrar_portal(app: &Router, email: &str) -> Value {
    let resposta = enviar(
        app,
        pedido(
            Method::POST,
            "/api/v1/auth/client/login",
            Some(json!({
                "escritorio": "alfa",
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

async fn configuracao_admin(app: &Router, token: &str, escritorio: Uuid) -> Response {
    enviar(
        app,
        pedido(
            Method::GET,
            &format!("/api/v1/admin/serpro/configuracao?escritorioId={escritorio}"),
            None,
            Some(token),
        ),
    )
    .await
}

async fn alterar_modo_trial(
    app: &Router,
    token: &str,
    escritorio: Uuid,
    modo_trial: bool,
    versao: i64,
) -> Response {
    enviar(
        app,
        pedido(
            Method::PATCH,
            "/api/v1/admin/serpro/trial",
            Some(json!({
                "escritorioId": escritorio,
                "modoTrialSerpro": modo_trial,
                "versao": versao,
            })),
            Some(token),
        ),
    )
    .await
}

#[tokio::test]
async fn gestor_consulta_e_altera_modo_com_auditoria_e_cache_invalidado() {
    let ambiente = ambiente();
    let escritorio = semear_escritorio(&ambiente.banco, "alfa");
    semear_token(&ambiente.banco, escritorio, "ACCESS");
    semear_token(&ambiente.banco, escritorio, "JWT");
    semear_procuracao(&ambiente.banco, escritorio);
    semear_credencial_producao(&ambiente.banco, escritorio, "INTEGRA_CONTADOR", "PRODUCAO");
    semear_certificado_valido(&ambiente.banco, escritorio);
    semear_gestor(&ambiente.banco);

    let token = token_de(&entrar_gestor(&ambiente.app).await);

    let leitura = configuracao_admin(&ambiente.app, &token, escritorio).await;
    assert_eq!(leitura.status(), StatusCode::OK);
    let corpo = corpo_json(leitura).await;
    assert_eq!(corpo["escritorioId"], escritorio.to_string());
    assert_eq!(corpo["modoTrialSerpro"], true);
    assert_eq!(corpo["ambienteEfetivo"], "TRIAL");
    assert_eq!(corpo["versao"], 1);

    let troca = alterar_modo_trial(&ambiente.app, &token, escritorio, false, 1).await;
    assert_eq!(troca.status(), StatusCode::OK);
    let corpo = corpo_json(troca).await;
    assert_eq!(corpo["modoTrialSerpro"], false);
    assert_eq!(corpo["ambienteEfetivo"], "PRODUCAO");
    assert_eq!(corpo["versaoAtual"], 2);

    let estado = sql_query("SELECT modo_trial_serpro, versao FROM escritorios WHERE id = $1")
        .bind::<SqlUuid, _>(escritorio)
        .get_result::<EstadoEscritorio>(&mut ambiente.banco.conexao())
        .expect("escritório deve existir");
    assert!(!estado.modo_trial_serpro);
    assert_eq!(estado.versao, 2);

    assert_eq!(
        contar(
            &ambiente.banco,
            &format!(
                "SELECT COUNT(*) AS total FROM tokens_serpro \
                 WHERE escritorio_id = '{escritorio}'"
            ),
        ),
        0,
        "tokens TRIAL devem ser apagados na troca"
    );
    assert_eq!(
        contar(
            &ambiente.banco,
            &format!(
                "SELECT COUNT(*) AS total FROM procuracoes \
                 WHERE escritorio_id = '{escritorio}'"
            ),
        ),
        0,
        "procurações participam da invalidação"
    );
    assert_eq!(
        contar(
            &ambiente.banco,
            "SELECT COUNT(*) AS total FROM auditoria_eventos \
             WHERE acao = 'SERPRO_MODO_TRIAL_ALTERADO' \
             AND administrador_id IS NOT NULL AND usuario_id IS NULL \
             AND valor_anterior IS NOT NULL AND valor_novo IS NOT NULL",
        ),
        1
    );

    let leitura_gravada = configuracao_admin(&ambiente.app, &token, escritorio).await;
    assert_eq!(leitura_gravada.status(), StatusCode::OK);
    let corpo = corpo_json(leitura_gravada).await;
    assert_eq!(corpo["ambienteEfetivo"], "PRODUCAO");

    let desatualizado = alterar_modo_trial(&ambiente.app, &token, escritorio, true, 1).await;
    assert_eq!(desatualizado.status(), StatusCode::CONFLICT);
    let corpo = corpo_json(desatualizado).await;
    assert_eq!(codigo_de_erro(&corpo), "CONFLITO");
}

#[tokio::test]
async fn troca_para_producao_bloqueada_sem_credencial() {
    let ambiente = ambiente();
    let escritorio = semear_escritorio(&ambiente.banco, "alfa");
    semear_certificado_valido(&ambiente.banco, escritorio);
    semear_gestor(&ambiente.banco);

    let token = token_de(&entrar_gestor(&ambiente.app).await);

    let troca = alterar_modo_trial(&ambiente.app, &token, escritorio, false, 1).await;
    assert_eq!(
        troca.status(),
        StatusCode::BAD_REQUEST,
        "sem credencial de produção a troca é bloqueada"
    );
    let corpo = corpo_json(troca).await;
    assert_eq!(codigo_de_erro(&corpo), "VALIDACAO");
    assert!(
        corpo["error"]["message"]
            .as_str()
            .unwrap_or_default()
            .contains("credencial"),
        "mensagem deve explicar o requisito faltante: {}",
        corpo["error"]["message"]
    );

    let estado = sql_query("SELECT modo_trial_serpro, versao FROM escritorios WHERE id = $1")
        .bind::<SqlUuid, _>(escritorio)
        .get_result::<EstadoEscritorio>(&mut ambiente.banco.conexao())
        .expect("escritório deve existir");
    assert!(estado.modo_trial_serpro, "modo trial não deve mudar");
    assert_eq!(estado.versao, 1, "versão não deve mudar");
}

#[tokio::test]
async fn troca_para_producao_bloqueada_sem_certificado_valido() {
    let ambiente = ambiente();
    let escritorio = semear_escritorio(&ambiente.banco, "alfa");
    semear_credencial_producao(&ambiente.banco, escritorio, "INTEGRA_CONTADOR", "PRODUCAO");
    semear_gestor(&ambiente.banco);

    let token = token_de(&entrar_gestor(&ambiente.app).await);

    let troca = alterar_modo_trial(&ambiente.app, &token, escritorio, false, 1).await;
    assert_eq!(
        troca.status(),
        StatusCode::BAD_REQUEST,
        "sem certificado A1 em validade a troca é bloqueada"
    );
    let corpo = corpo_json(troca).await;
    assert_eq!(codigo_de_erro(&corpo), "VALIDACAO");

    let estado = sql_query("SELECT modo_trial_serpro, versao FROM escritorios WHERE id = $1")
        .bind::<SqlUuid, _>(escritorio)
        .get_result::<EstadoEscritorio>(&mut ambiente.banco.conexao())
        .expect("escritório deve existir");
    assert!(estado.modo_trial_serpro);
    assert_eq!(estado.versao, 1);
}

#[tokio::test]
async fn portal_le_ambiente_do_proprio_escritorio_sem_poder_trocar() {
    let ambiente = ambiente();
    let escritorio = semear_escritorio(&ambiente.banco, "alfa");
    let outro = semear_escritorio(&ambiente.banco, "beta");
    semear_operador(&ambiente.banco, escritorio, "operador@alfa.test");
    semear_gestor(&ambiente.banco);

    let token = token_de(&entrar_portal(&ambiente.app, "operador@alfa.test").await);

    let leitura = enviar(
        &ambiente.app,
        pedido(
            Method::GET,
            "/api/v1/serpro/configuracao",
            None,
            Some(&token),
        ),
    )
    .await;
    assert_eq!(leitura.status(), StatusCode::OK);
    let corpo = corpo_json(leitura).await;
    assert_eq!(corpo["escritorioId"], escritorio.to_string());
    assert_eq!(corpo["ambienteEfetivo"], "TRIAL");
    assert_ne!(corpo["escritorioId"], outro.to_string());

    let trocas = [
        pedido(
            Method::PATCH,
            "/api/v1/serpro/configuracao",
            Some(json!({ "modoTrialSerpro": false })),
            Some(&token),
        ),
        pedido(
            Method::DELETE,
            "/api/v1/serpro/configuracao",
            None,
            Some(&token),
        ),
    ];
    for requisicao in trocas {
        let resposta = enviar(&ambiente.app, requisicao).await;
        assert_eq!(
            resposta.status(),
            StatusCode::METHOD_NOT_ALLOWED,
            "rota de leitura não aceita troca de modo"
        );
    }

    let gestor = token_de(&entrar_gestor(&ambiente.app).await);
    let gestor_no_portal = enviar(
        &ambiente.app,
        pedido(
            Method::GET,
            "/api/v1/serpro/configuracao",
            None,
            Some(&gestor),
        ),
    )
    .await;
    assert_eq!(gestor_no_portal.status(), StatusCode::UNAUTHORIZED);
    let corpo = corpo_json(gestor_no_portal).await;
    assert_eq!(codigo_de_erro(&corpo), "AUTH_AUDIENCE_MISMATCH");
}

#[tokio::test]
async fn rotas_admin_serpro_exigem_audiencia_admin() {
    let ambiente = ambiente();
    let escritorio = semear_escritorio(&ambiente.banco, "alfa");
    semear_operador(&ambiente.banco, escritorio, "operador@alfa.test");
    semear_gestor(&ambiente.banco);

    let sem_token = configuracao_admin(&ambiente.app, "", escritorio).await;
    assert_eq!(sem_token.status(), StatusCode::UNAUTHORIZED);
    let corpo = corpo_json(sem_token).await;
    assert_eq!(codigo_de_erro(&corpo), "NAO_AUTENTICADO");

    let token = token_de(&entrar_portal(&ambiente.app, "operador@alfa.test").await);

    let leitura = configuracao_admin(&ambiente.app, &token, escritorio).await;
    assert_eq!(leitura.status(), StatusCode::UNAUTHORIZED);
    let corpo = corpo_json(leitura).await;
    assert_eq!(codigo_de_erro(&corpo), "AUTH_AUDIENCE_MISMATCH");

    let troca = alterar_modo_trial(&ambiente.app, &token, escritorio, false, 1).await;
    assert_eq!(troca.status(), StatusCode::UNAUTHORIZED);

    let lista = enviar(
        &ambiente.app,
        pedido(Method::GET, "/api/v1/admin/escritorios", None, Some(&token)),
    )
    .await;
    assert_eq!(lista.status(), StatusCode::UNAUTHORIZED);
    let corpo = corpo_json(lista).await;
    assert_eq!(codigo_de_erro(&corpo), "AUTH_AUDIENCE_MISMATCH");

    let gestor = token_de(&entrar_gestor(&ambiente.app).await);
    let desconhecido = configuracao_admin(&ambiente.app, &gestor, Uuid::now_v7()).await;
    assert_eq!(desconhecido.status(), StatusCode::NOT_FOUND);
    let corpo = corpo_json(desconhecido).await;
    assert_eq!(codigo_de_erro(&corpo), "NAO_ENCONTRADO");

    let troca_desconhecida =
        alterar_modo_trial(&ambiente.app, &gestor, Uuid::now_v7(), false, 1).await;
    assert_eq!(troca_desconhecida.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn gestor_lista_escritorios_com_modo_trial() {
    let ambiente = ambiente();
    let _afa = semear_escritorio(&ambiente.banco, "alfa");
    let _beta = semear_escritorio(&ambiente.banco, "beta");
    semear_gestor(&ambiente.banco);
    semear_operador(
        &ambiente.banco,
        id_do_escritorio(&ambiente.banco, "alfa"),
        "operador@alfa.test",
    );

    let token = token_de(&entrar_gestor(&ambiente.app).await);

    let lista = enviar(
        &ambiente.app,
        pedido(Method::GET, "/api/v1/admin/escritorios", None, Some(&token)),
    )
    .await;
    assert_eq!(lista.status(), StatusCode::OK);
    let corpo = corpo_json(lista).await;
    let itens = corpo.as_array().expect("lista de escritórios");
    assert_eq!(itens.len(), 2);
    for item in itens {
        assert!(item["id"].as_str().is_some());
        assert!(item["modoTrialSerpro"].is_boolean());
        assert!(item["slug"].as_str().is_some());
    }
}
