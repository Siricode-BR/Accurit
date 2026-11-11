mod common;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use axum::response::Response;
use diesel::RunQueryDsl;
use diesel::sql_query;
use diesel::sql_types::{Text, Uuid as SqlUuid};
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

struct BancoFixo;

impl SaudeBanco for BancoFixo {
    async fn checar(&self) -> Result<(), SaudeBancoError> {
        Ok(())
    }
}

struct Ambiente {
    app: Router,
    /// Mantém o banco temporário vivo enquanto o roteador é exercitado.
    _banco: BancoTemporario,
    escritorio_a: Uuid,
    escritorio_b: Uuid,
    usuario_a: Uuid,
    cliente_a: Uuid,
    certificado_a: Uuid,
    trabalho_a: Uuid,
    guia_a: Uuid,
    relatorio_a: Uuid,
    notificacao_a: Uuid,
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
    let usuario_a = semear_usuario(&banco, escritorio_a, "a@idor.test", "52998224725");
    let usuario_b = semear_usuario(&banco, escritorio_b, "b@idor.test", "11144477735");
    let cliente_a = semear_cliente(&banco, escritorio_a, "Cliente Secreto A", "52998224725");
    let _cliente_b = semear_cliente(&banco, escritorio_b, "Cliente B", "12345678909");
    semear_socio_contato(&banco, escritorio_a, cliente_a);
    semear_filial(&banco, escritorio_a, cliente_a);
    semear_tag(&banco, escritorio_a, cliente_a);

    let certificado_a = semear_certificado(&banco, escritorio_a, CNPJ_A);
    let trabalho_a = semear_trabalho(&banco, escritorio_a);
    let guia_a = semear_guia(&banco, escritorio_a, cliente_a, usuario_a);
    let relatorio_a = semear_relatorio(&banco, escritorio_a, cliente_a, usuario_a);
    let notificacao_a = semear_notificacao(&banco, escritorio_a, usuario_a);
    let (plano_a, plano_b) = semear_planos(&banco);
    semear_assinatura(&banco, escritorio_a, plano_a, 999);
    semear_assinatura(&banco, escritorio_b, plano_b, 5);
    let _ = usuario_b;

    Ambiente {
        app,
        _banco: banco,
        escritorio_a,
        escritorio_b,
        usuario_a,
        cliente_a,
        certificado_a,
        trabalho_a,
        guia_a,
        relatorio_a,
        notificacao_a,
    }
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

fn semear_usuario(banco: &BancoTemporario, escritorio: Uuid, email: &str, cpf: &str) -> Uuid {
    let config = configuracao_com_extras("", &[]);
    let hash = HashArgon2::novo(&config.auth)
        .expect("argon2")
        .gerar_hash(SENHA)
        .expect("hash");
    let id = Uuid::now_v7();
    sql_query(
        "INSERT INTO usuarios (id, escritorio_id, email, cpf, senha_hash, papel, ativo) \
         VALUES ($1, $2, $3, $4, $5, 'OPERADOR', TRUE)",
    )
    .bind::<SqlUuid, _>(id)
    .bind::<SqlUuid, _>(escritorio)
    .bind::<Text, _>(email)
    .bind::<Text, _>(cpf)
    .bind::<Text, _>(hash)
    .execute(&mut banco.conexao())
    .expect("usuario");
    id
}

fn semear_cliente(banco: &BancoTemporario, escritorio: Uuid, nome: &str, doc: &str) -> Uuid {
    let id = Uuid::now_v7();
    sql_query(
        "INSERT INTO clientes (id, escritorio_id, tipo, cpf_cnpj, nome_razao_social) \
         VALUES ($1, $2, 'PF', $3, $4)",
    )
    .bind::<SqlUuid, _>(id)
    .bind::<SqlUuid, _>(escritorio)
    .bind::<Text, _>(doc)
    .bind::<Text, _>(nome)
    .execute(&mut banco.conexao())
    .expect("cliente");
    id
}

fn semear_socio_contato(banco: &BancoTemporario, escritorio: Uuid, cliente: Uuid) {
    sql_query(
        "INSERT INTO socio_contatos (id, escritorio_id, cliente_id, papel, nome) \
         VALUES ($1, $2, $3, 'SOCIO', 'Sócio Secreto A')",
    )
    .bind::<SqlUuid, _>(Uuid::now_v7())
    .bind::<SqlUuid, _>(escritorio)
    .bind::<SqlUuid, _>(cliente)
    .execute(&mut banco.conexao())
    .expect("socio");
}

fn semear_filial(banco: &BancoTemporario, escritorio: Uuid, cliente: Uuid) {
    sql_query(
        "INSERT INTO cliente_filiais (id, escritorio_id, cliente_id, cnpj, nome_unidade) \
         VALUES ($1, $2, $3, '11444777000161', 'Filial Secreta A')",
    )
    .bind::<SqlUuid, _>(Uuid::now_v7())
    .bind::<SqlUuid, _>(escritorio)
    .bind::<SqlUuid, _>(cliente)
    .execute(&mut banco.conexao())
    .expect("filial");
}

fn semear_tag(banco: &BancoTemporario, escritorio: Uuid, cliente: Uuid) {
    let tag = Uuid::now_v7();
    let mut conexao = banco.conexao();
    sql_query(
        "INSERT INTO cliente_tags (id, escritorio_id, nome) VALUES ($1, $2, 'Tag Secreta A')",
    )
    .bind::<SqlUuid, _>(tag)
    .bind::<SqlUuid, _>(escritorio)
    .execute(&mut conexao)
    .expect("tag");
    sql_query(
        "INSERT INTO cliente_tags_vinculos (id, escritorio_id, tag_id, cliente_id) \
         VALUES ($1, $2, $3, $4)",
    )
    .bind::<SqlUuid, _>(Uuid::now_v7())
    .bind::<SqlUuid, _>(escritorio)
    .bind::<SqlUuid, _>(tag)
    .bind::<SqlUuid, _>(cliente)
    .execute(&mut conexao)
    .expect("vinculo tag");
}

fn semear_certificado(banco: &BancoTemporario, escritorio: Uuid, titular: &str) -> Uuid {
    let id = Uuid::now_v7();
    sql_query(
        "INSERT INTO certificados_a1 \
         (id, escritorio_id, tipo, titular_cpf_cnpj, conteudo_cifrado, fingerprint_sha256, valido_de, valido_ate) \
         VALUES ($1, $2, 'A1', $3, '\\x01'::bytea, $4, NOW() - INTERVAL '1 day', NOW() + INTERVAL '300 days')",
    )
    .bind::<SqlUuid, _>(id)
    .bind::<SqlUuid, _>(escritorio)
    .bind::<Text, _>(titular)
    .bind::<Text, _>(format!("{:0>64}", id.simple()))
    .execute(&mut banco.conexao())
    .expect("certificado");
    id
}

fn semear_trabalho(banco: &BancoTemporario, escritorio: Uuid) -> Uuid {
    let id = Uuid::now_v7();
    sql_query(
        "INSERT INTO trabalhos_agendados \
         (id, escritorio_id, nome, tipo, produto, ambiente, cron_expression) \
         VALUES ($1, $2, 'Rotina secreta', 'consulta_divida_ativa', 'DIVIDA_ATIVA', 'TRIAL', '0 0 * * *')",
    )
    .bind::<SqlUuid, _>(id)
    .bind::<SqlUuid, _>(escritorio)
    .execute(&mut banco.conexao())
    .expect("trabalho");
    id
}

fn semear_guia(banco: &BancoTemporario, escritorio: Uuid, cliente: Uuid, usuario: Uuid) -> Uuid {
    let id = Uuid::now_v7();
    sql_query(
        "INSERT INTO guias_emissao \
         (id, usuario_id, escritorio_id, cliente_id, tipo_guia, competencia, valor_total, status, idempotencia_chave) \
         VALUES ($1, $2, $3, $4, 'DAS', '2026-11', 100.00, 'solicitada', $5)",
    )
    .bind::<SqlUuid, _>(id)
    .bind::<SqlUuid, _>(usuario)
    .bind::<SqlUuid, _>(escritorio)
    .bind::<SqlUuid, _>(cliente)
    .bind::<Text, _>(format!("guia-{id}"))
    .execute(&mut banco.conexao())
    .expect("guia");
    id
}

fn semear_relatorio(
    banco: &BancoTemporario,
    escritorio: Uuid,
    cliente: Uuid,
    usuario: Uuid,
) -> Uuid {
    let id = Uuid::now_v7();
    sql_query(
        "INSERT INTO relatorios \
         (id, escritorio_id, usuario_id, cliente_id, tipo_relatorio, formato, status, conteudo, nome_arquivo, expira_em) \
         VALUES ($1, $2, $3, $4, 'situacao_fiscal', 'PDF', 'concluido', '\\x01'::bytea, 'relatorio.pdf', NOW() + INTERVAL '30 days')",
    )
    .bind::<SqlUuid, _>(id)
    .bind::<SqlUuid, _>(escritorio)
    .bind::<SqlUuid, _>(usuario)
    .bind::<SqlUuid, _>(cliente)
    .execute(&mut banco.conexao())
    .expect("relatorio");
    id
}

fn semear_notificacao(banco: &BancoTemporario, escritorio: Uuid, usuario: Uuid) -> Uuid {
    let id = Uuid::now_v7();
    sql_query(
        "INSERT INTO notificacoes \
         (id, usuario_id, escritorio_id, canal, tipo, titulo, corpo) \
         VALUES ($1, $2, $3, 'in_app', 'falha_rotina', 'Notificação secreta', 'conteúdo')",
    )
    .bind::<SqlUuid, _>(id)
    .bind::<SqlUuid, _>(usuario)
    .bind::<SqlUuid, _>(escritorio)
    .execute(&mut banco.conexao())
    .expect("notificacao");
    id
}

fn semear_planos(banco: &BancoTemporario) -> (Uuid, Uuid) {
    let plano_a = Uuid::now_v7();
    let plano_b = Uuid::now_v7();
    for (id, nome) in [(plano_a, "Premium A"), (plano_b, "Básico B")] {
        sql_query(
            "INSERT INTO planos_escritorio (id, nome, chamadas_mes, clientes_limite, usuarios_limite) \
             VALUES ($1, $2, 1000, 100, 10)",
        )
        .bind::<SqlUuid, _>(id)
        .bind::<Text, _>(nome)
        .execute(&mut banco.conexao())
        .expect("plano");
    }
    (plano_a, plano_b)
}

fn semear_assinatura(banco: &BancoTemporario, escritorio: Uuid, plano: Uuid, limite: i32) {
    sql_query(
        "INSERT INTO assinaturas_escritorio \
         (id, escritorio_id, plano_id, chamadas_mes_limite, clientes_limite, usuarios_limite) \
         VALUES ($1, $2, $3, $4, 100, 10)",
    )
    .bind::<SqlUuid, _>(Uuid::now_v7())
    .bind::<SqlUuid, _>(escritorio)
    .bind::<SqlUuid, _>(plano)
    .bind::<diesel::sql_types::Integer, _>(limite)
    .execute(&mut banco.conexao())
    .expect("assinatura");
}

fn pedido(metodo: Method, caminho: &str, token: Option<&str>) -> Request<Body> {
    let mut construtor = Request::builder().method(metodo).uri(caminho);
    if let Some(token) = token {
        construtor = construtor.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    construtor.body(Body::empty()).expect("request")
}

async fn enviar(app: &Router, request: Request<Body>) -> Response {
    app.clone().oneshot(request).await.expect("response")
}

async fn json(resposta: Response) -> Value {
    let bytes = to_bytes(resposta.into_body(), 1024 * 1024)
        .await
        .expect("body");
    serde_json::from_slice(&bytes).unwrap_or(Value::Null)
}

async fn login(app: &Router, email: &str) -> String {
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/auth/client/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "email": email, "senha": SENHA }).to_string(),
        ))
        .expect("request");
    let resposta = enviar(app, request).await;
    assert_eq!(resposta.status(), StatusCode::OK, "login {email}");
    json(resposta).await["accessToken"]
        .as_str()
        .expect("token")
        .to_owned()
}

#[tokio::test]
async fn matriz_idor_bloqueia_entidades_de_outro_escritorio() {
    let ambiente = ambiente();
    let token_b = login(&ambiente.app, "b@idor.test").await;

    let detalhes = [
        format!("/api/v1/clientes/{}", ambiente.cliente_a),
        format!("/api/v1/clientes/{}/situacao-fiscal", ambiente.cliente_a),
        format!("/api/v1/certificados/{}", ambiente.certificado_a),
        format!("/api/v1/trabalhos/{}", ambiente.trabalho_a),
        format!("/api/v1/guias/{}", ambiente.guia_a),
        format!("/api/v1/relatorios/{}", ambiente.relatorio_a),
        format!("/api/v1/relatorios/{}/baixar", ambiente.relatorio_a),
        format!("/api/v1/lgpd/clientes/{}/export", ambiente.cliente_a),
        format!("/api/v1/usuarios/{}", ambiente.usuario_a),
    ];

    for caminho in detalhes {
        let resposta = enviar(&ambiente.app, pedido(Method::GET, &caminho, Some(&token_b))).await;
        let status = resposta.status();
        assert!(
            status == StatusCode::NOT_FOUND
                || status == StatusCode::FORBIDDEN
                || status == StatusCode::UNAUTHORIZED,
            "{caminho} deveria negar acesso, veio {status}"
        );
        let corpo = json(resposta).await;
        let texto = corpo.to_string();
        assert!(
            !texto.contains("Cliente Secreto A"),
            "{caminho} vazou nome de cliente alheio"
        );
        assert!(
            !texto.contains(&ambiente.cliente_a.to_string()),
            "{caminho} vazou id de cliente alheio"
        );
    }

    // Subcoleções são escopadas pelo escritório: podem responder 200 vazio,
    // mas jamais podem devolver itens do escritório A.
    let colecoes = [
        format!("/api/v1/clientes/{}/consultas", ambiente.cliente_a),
        format!("/api/v1/clientes/{}/socios-contatos", ambiente.cliente_a),
        format!("/api/v1/clientes/{}/filiais", ambiente.cliente_a),
        format!("/api/v1/clientes/{}/tags", ambiente.cliente_a),
        format!(
            "/api/v1/lgpd/clientes/{}/consentimentos",
            ambiente.cliente_a
        ),
    ];

    for caminho in colecoes {
        let resposta = enviar(&ambiente.app, pedido(Method::GET, &caminho, Some(&token_b))).await;
        let status = resposta.status();
        if status != StatusCode::OK {
            assert!(
                status == StatusCode::NOT_FOUND
                    || status == StatusCode::FORBIDDEN
                    || status == StatusCode::UNAUTHORIZED,
                "{caminho} deveria negar ou esvaziar, veio {status}"
            );
            continue;
        }
        let corpo = json(resposta).await;
        let texto = corpo.to_string();
        assert!(
            !texto.contains("Secreta A") && !texto.contains("Secreto A"),
            "{caminho} vazou item do escritório A: {texto}"
        );
        assert!(
            !texto.contains(&ambiente.cliente_a.to_string()),
            "{caminho} vazou id de cliente alheio"
        );
        if let Some(itens) = corpo.get("itens").and_then(Value::as_array) {
            assert!(itens.is_empty(), "{caminho} devolveu itens alheios");
        }
    }

    // Listagens só trazem o próprio escritório.
    let clientes = enviar(
        &ambiente.app,
        pedido(Method::GET, "/api/v1/clientes", Some(&token_b)),
    )
    .await;
    let corpo = json(clientes).await;
    assert!(!corpo.to_string().contains("Cliente Secreto A"));

    let notificacoes = enviar(
        &ambiente.app,
        pedido(Method::GET, "/api/v1/notificacoes", Some(&token_b)),
    )
    .await;
    let corpo = json(notificacoes).await;
    assert!(!corpo.to_string().contains("Notificação secreta"));

    // Consumo usa o plano do escritório B, não o de A.
    let consumo = enviar(
        &ambiente.app,
        pedido(Method::GET, "/api/v1/consumo", Some(&token_b)),
    )
    .await;
    assert_eq!(consumo.status(), StatusCode::OK);
    let corpo = json(consumo).await;
    assert_eq!(corpo["escritorioId"], ambiente.escritorio_b.to_string());
    assert_eq!(corpo["chamadasMesLimite"], 5);

    // Backoffice exige audiência admin.
    let admin = enviar(
        &ambiente.app,
        pedido(Method::GET, "/api/v1/admin/consumo", Some(&token_b)),
    )
    .await;
    assert_eq!(admin.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(json(admin).await["error"]["code"], "AUTH_AUDIENCE_MISMATCH");

    // A entidade continua acessível ao próprio escritório.
    let token_a = login(&ambiente.app, "a@idor.test").await;
    let proprio = enviar(
        &ambiente.app,
        pedido(
            Method::GET,
            &format!("/api/v1/clientes/{}", ambiente.cliente_a),
            Some(&token_a),
        ),
    )
    .await;
    assert_eq!(proprio.status(), StatusCode::OK);
    let corpo = json(proprio).await;
    assert_eq!(corpo["nomeRazaoSocial"], "Cliente Secreto A");

    let consumo_a = enviar(
        &ambiente.app,
        pedido(Method::GET, "/api/v1/consumo", Some(&token_a)),
    )
    .await;
    assert_eq!(consumo_a.status(), StatusCode::OK);
    let corpo = json(consumo_a).await;
    assert_eq!(corpo["escritorioId"], ambiente.escritorio_a.to_string());
    assert_eq!(corpo["chamadasMesLimite"], 999);

    // Notificação de outro usuário não pode ser marcada como lida.
    let ler_alheia = Request::builder()
        .method(Method::PATCH)
        .uri(format!(
            "/api/v1/notificacoes/{}/ler",
            ambiente.notificacao_a
        ))
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {token_b}"))
        .body(Body::from("{}"))
        .expect("request");
    let resposta = enviar(&ambiente.app, ler_alheia).await;
    assert_eq!(resposta.status(), StatusCode::NOT_FOUND);

    let ler_propria = Request::builder()
        .method(Method::PATCH)
        .uri(format!(
            "/api/v1/notificacoes/{}/ler",
            ambiente.notificacao_a
        ))
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {token_a}"))
        .body(Body::from("{}"))
        .expect("request");
    let resposta = enviar(&ambiente.app, ler_propria).await;
    assert_eq!(resposta.status(), StatusCode::NO_CONTENT);
}
