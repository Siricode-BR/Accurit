//! Testes de integração das constraints do banco de dados da Fase 3
//! (SERPRO): defaults de `escritorios`, unicidade e CHECKs de
//! `tokens_serpro`/`credenciais_serpro` e comportamento de
//! `consultas_fiscais`/`auditoria_eventos` (SPEC §3.1; sem migrations
//! novas — as tabelas já existem desde as migrations 1 e 2B).

mod common;

use diesel::QueryableByName;
use diesel::RunQueryDsl;
use diesel::sql_query;
use diesel::sql_types::{BigInt, Bool, Jsonb, Text, Uuid as SqlUuid};
use serde_json::json;
use uuid::Uuid;

use accurit_backend::infrastructure::persistence::migrations::{
    MIGRATIONS, aplicar_migrations_com_lock,
};

use common::BancoTemporario;

struct Ambiente {
    banco: BancoTemporario,
}

fn ambiente() -> Ambiente {
    let banco = BancoTemporario::novo();
    aplicar_migrations_com_lock(banco.conexao(), MIGRATIONS).expect("migrations devem aplicar");
    Ambiente { banco }
}

#[derive(QueryableByName)]
struct ModoTrial {
    #[diesel(sql_type = Bool)]
    modo_trial_serpro: bool,
    #[diesel(sql_type = BigInt)]
    versao: i64,
}

#[derive(QueryableByName)]
struct Total {
    #[diesel(sql_type = Bool)]
    total: bool,
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

fn existe(banco: &BancoTemporario, consulta: &str) -> bool {
    sql_query(consulta)
        .get_result::<Total>(&mut banco.conexao())
        .expect("consulta deve funcionar")
        .total
}

#[tokio::test]
async fn escritorio_nasce_com_modo_trial_e_versao_default() {
    let ambiente = ambiente();
    let id = semear_escritorio(&ambiente.banco, "alfa");

    let estado = sql_query("SELECT modo_trial_serpro, versao FROM escritorios WHERE id = $1")
        .bind::<SqlUuid, _>(id)
        .get_result::<ModoTrial>(&mut ambiente.banco.conexao())
        .expect("escritório deve existir");

    assert!(estado.modo_trial_serpro, "default do modo trial é TRUE");
    assert_eq!(estado.versao, 1, "versão otimista começa em 1");
}

#[tokio::test]
async fn credenciais_serpro_impedem_duplicidade_e_valores_ilegais() {
    let ambiente = ambiente();
    let escritorio = semear_escritorio(&ambiente.banco, "alfa");

    let mut conexao = ambiente.banco.conexao();
    let nova = sql_query(
        "INSERT INTO credenciais_serpro \
         (id, escritorio_id, produto, ambiente, consumer_key_cifrado, consumer_secret_cifrado) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind::<SqlUuid, _>(Uuid::now_v7())
    .bind::<SqlUuid, _>(escritorio)
    .bind::<Text, _>("INTEGRA_CONTADOR")
    .bind::<Text, _>("TRIAL")
    .bind::<diesel::sql_types::Binary, _>(vec![1_u8, 2, 3, 4])
    .bind::<diesel::sql_types::Binary, _>(vec![5_u8, 6, 7, 8])
    .execute(&mut conexao)
    .expect("credencial válida deve ser inserida")
        > 0;
    assert!(nova);

    let duplicada = sql_query(
        "INSERT INTO credenciais_serpro \
         (id, escritorio_id, produto, ambiente, consumer_key_cifrado, consumer_secret_cifrado) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind::<SqlUuid, _>(Uuid::now_v7())
    .bind::<SqlUuid, _>(escritorio)
    .bind::<Text, _>("INTEGRA_CONTADOR")
    .bind::<Text, _>("TRIAL")
    .bind::<diesel::sql_types::Binary, _>(vec![9_u8, 10])
    .bind::<diesel::sql_types::Binary, _>(vec![11_u8, 12])
    .execute(&mut conexao);

    assert!(
        duplicada.is_err(),
        "UNIQUE (escritorio_id, produto, ambiente) deve impedir duplicidade"
    );

    for produto in ["SAPI", "NADA"] {
        let rejeitada = sql_query(
            "INSERT INTO credenciais_serpro \
             (id, escritorio_id, produto, ambiente, consumer_key_cifrado, consumer_secret_cifrado) \
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind::<SqlUuid, _>(Uuid::now_v7())
        .bind::<SqlUuid, _>(escritorio)
        .bind::<Text, _>(produto)
        .bind::<Text, _>("TRIAL")
        .bind::<diesel::sql_types::Binary, _>(vec![1_u8])
        .bind::<diesel::sql_types::Binary, _>(vec![1_u8])
        .execute(&mut conexao);
        assert!(
            rejeitada.is_err(),
            "CHECK de produto deve rejeitar '{produto}'"
        );
    }

    let ambiente_ilegal = sql_query(
        "INSERT INTO credenciais_serpro \
         (id, escritorio_id, produto, ambiente, consumer_key_cifrado, consumer_secret_cifrado) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind::<SqlUuid, _>(Uuid::now_v7())
    .bind::<SqlUuid, _>(escritorio)
    .bind::<Text, _>("INTEGRA_CONTADOR")
    .bind::<Text, _>("PROD")
    .bind::<diesel::sql_types::Binary, _>(vec![1_u8])
    .bind::<diesel::sql_types::Binary, _>(vec![1_u8])
    .execute(&mut conexao);
    assert!(
        ambiente_ilegal.is_err(),
        "CHECK de ambiente deve rejeitar 'PROD'"
    );

    // mesma credencial para outro produto deve ser aceita
    let outro_produto = sql_query(
        "INSERT INTO credenciais_serpro \
         (id, escritorio_id, produto, ambiente, consumer_key_cifrado, consumer_secret_cifrado) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind::<SqlUuid, _>(Uuid::now_v7())
    .bind::<SqlUuid, _>(escritorio)
    .bind::<Text, _>("DIVIDA_ATIVA")
    .bind::<Text, _>("TRIAL")
    .bind::<diesel::sql_types::Binary, _>(vec![1_u8])
    .bind::<diesel::sql_types::Binary, _>(vec![1_u8])
    .execute(&mut conexao)
    .expect("credencial de outro produto é permitida");
    assert!(outro_produto > 0);
}

#[tokio::test]
async fn tokens_serpro_impedem_duplicidade_e_valores_ilegais() {
    let ambiente = ambiente();
    let escritorio = semear_escritorio(&ambiente.banco, "alfa");

    let mut conexao = ambiente.banco.conexao();
    let inserir = |conexao: &mut diesel::PgConnection, id: Uuid, produto: &str, tipo: &str| {
        sql_query(
            "INSERT INTO tokens_serpro \
             (id, escritorio_id, produto, ambiente, tipo_token, token_cifrado, expira_em) \
             VALUES ($1, $2, $3, $4, $5, $6, NOW() + INTERVAL '60 minutes')",
        )
        .bind::<SqlUuid, _>(id)
        .bind::<SqlUuid, _>(escritorio)
        .bind::<Text, _>(produto)
        .bind::<Text, _>("TRIAL")
        .bind::<Text, _>(tipo)
        .bind::<diesel::sql_types::Binary, _>(vec![1_u8, 2, 3])
        .execute(conexao)
    };

    inserir(&mut conexao, Uuid::now_v7(), "INTEGRA_CONTADOR", "ACCESS")
        .expect("token válido deve ser inserido");

    let duplicado = inserir(&mut conexao, Uuid::now_v7(), "INTEGRA_CONTADOR", "ACCESS");
    assert!(
        duplicado.is_err(),
        "UNIQUE NULLS NOT DISTINCT (escritorio_id, cliente_id, produto, ambiente, tipo_token, \
         autor_pedido_dados) deve impedir duplicidade"
    );

    let outro_tipo = inserir(&mut conexao, Uuid::now_v7(), "INTEGRA_CONTADOR", "JWT");
    assert!(outro_tipo.is_ok(), "trocar tipo_token é permitido");

    let tipo_ilegal = inserir(&mut conexao, Uuid::now_v7(), "INTEGRA_CONTADOR", "REFRESH");
    assert!(
        tipo_ilegal.is_err(),
        "CHECK de tipo_token deve rejeitar 'REFRESH'"
    );

    let produto_ilegal = inserir(&mut conexao, Uuid::now_v7(), "SAPI", "ACCESS");
    assert!(
        produto_ilegal.is_err(),
        "CHECK de produto deve rejeitar 'SAPI'"
    );
}

#[tokio::test]
async fn consultas_fiscais_impedem_ambiente_ilegal() {
    let ambiente = ambiente();
    let escritorio = semear_escritorio(&ambiente.banco, "alfa");

    let mut conexao = ambiente.banco.conexao();
    sql_query(
        "INSERT INTO consultas_fiscais \
         (id, escritorio_id, produto, servico, ambiente, estado) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind::<SqlUuid, _>(Uuid::now_v7())
    .bind::<SqlUuid, _>(escritorio)
    .bind::<Text, _>("INTEGRA_CONTADOR")
    .bind::<Text, _>("CONSULTA_SITUACAO_FISCAL")
    .bind::<Text, _>("TRIAL")
    .bind::<Text, _>("SUCESSO")
    .execute(&mut conexao)
    .expect("consulta TRIAL válida");

    let producao = sql_query(
        "INSERT INTO consultas_fiscais \
         (id, escritorio_id, produto, servico, ambiente, estado) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind::<SqlUuid, _>(Uuid::now_v7())
    .bind::<SqlUuid, _>(escritorio)
    .bind::<Text, _>("INTEGRA_CONTADOR")
    .bind::<Text, _>("CONSULTA_SITUACAO_FISCAL")
    .bind::<Text, _>("PRODUCAO")
    .bind::<Text, _>("SUCESSO")
    .execute(&mut conexao)
    .expect("consulta PRODUCAO válida — ambiente permitido pelo toggle");
    assert!(producao > 0);

    let prod_ilegal = sql_query(
        "INSERT INTO consultas_fiscais \
         (id, escritorio_id, produto, servico, ambiente, estado) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind::<SqlUuid, _>(Uuid::now_v7())
    .bind::<SqlUuid, _>(escritorio)
    .bind::<Text, _>("INTEGRA_CONTADOR")
    .bind::<Text, _>("CONSULTA_SITUACAO_FISCAL")
    .bind::<Text, _>("HOMOLOG")
    .bind::<Text, _>("SUCESSO")
    .execute(&mut conexao);
    assert!(
        prod_ilegal.is_err(),
        "CHECK de ambiente deve rejeitar 'HOMOLOG'"
    );
}

#[tokio::test]
async fn auditoria_aceita_evento_de_administrador_com_valores() {
    let ambiente = ambiente();
    let escritorio = semear_escritorio(&ambiente.banco, "alfa");

    let mut conexao = ambiente.banco.conexao();
    sql_query(
        "INSERT INTO auditoria_eventos \
         (id, escritorio_id, administrador_id, acao, entidade, entidade_id, \
          valor_anterior, valor_novo, ip) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9::inet)",
    )
    .bind::<SqlUuid, _>(Uuid::now_v7())
    .bind::<SqlUuid, _>(escritorio)
    .bind::<SqlUuid, _>(Uuid::now_v7())
    .bind::<Text, _>("SERPRO_MODO_TRIAL_ALTERADO")
    .bind::<Text, _>("escritorios")
    .bind::<SqlUuid, _>(escritorio)
    .bind::<Jsonb, _>(json!({"modoTrialSerpro": true, "versao": 1}))
    .bind::<Jsonb, _>(json!({"modoTrialSerpro": false, "versao": 2}))
    .bind::<Text, _>("2001:db8::1")
    .execute(&mut conexao)
    .expect("evento de auditoria deve ser aceito");

    assert!(
        existe(
            &ambiente.banco,
            "SELECT (COUNT(*) > 0) AS total FROM auditoria_eventos \
             WHERE acao = 'SERPRO_MODO_TRIAL_ALTERADO' \
             AND administrador_id IS NOT NULL AND usuario_id IS NULL \
             AND valor_anterior IS NOT NULL AND valor_novo IS NOT NULL"
        ),
        "evento deve registrar ator de backoffice e valores antes/depois"
    );
    let evento_com_ip = existe(
        &ambiente.banco,
        "SELECT (COUNT(*) > 0) AS total FROM auditoria_eventos \
         WHERE acao = 'SERPRO_MODO_TRIAL_ALTERADO' AND ip = '2001:db8::1'",
    );
    assert!(evento_com_ip, "IP do administrador deve ser registrado");
    assert!(!existe(
        &ambiente.banco,
        "SELECT (COUNT(*) > 0) AS total FROM auditoria_eventos \
         WHERE acao = 'SERPRO_MODO_TRIAL_ALTERADO' AND ip = '203.1.1.1'"
    ));
}

#[tokio::test]
async fn procuracoes_e_certificados_aceitam_filhos_do_escritorio() {
    let ambiente = ambiente();
    let escritorio = semear_escritorio(&ambiente.banco, "alfa");
    let agora = chrono::Utc::now();

    let mut conexao = ambiente.banco.conexao();

    sql_query(
        "INSERT INTO procuracoes \
         (id, escritorio_id, ambiente, contribuinte_cpf_cnpj, procurador_cpf_cnpj, \
          codigo_servico, status, verificado_em) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind::<SqlUuid, _>(Uuid::now_v7())
    .bind::<SqlUuid, _>(escritorio)
    .bind::<Text, _>("TRIAL")
    .bind::<Text, _>("11111111111111")
    .bind::<Text, _>("22222222222222")
    .bind::<Text, _>("00093")
    .bind::<Text, _>("ATIVO")
    .bind::<diesel::sql_types::Timestamptz, _>(agora)
    .execute(&mut conexao)
    .expect("procuração deve ser inserida");

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
    .bind::<diesel::sql_types::Binary, _>(vec![1_u8, 2, 3])
    .bind::<Text, _>("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
    .bind::<diesel::sql_types::Timestamptz, _>(agora - chrono::Duration::days(1))
    .bind::<diesel::sql_types::Timestamptz, _>(agora + chrono::Duration::days(30))
    .execute(&mut conexao)
    .expect("certificado deve ser inserido");

    assert!(existe(
        &ambiente.banco,
        "SELECT (COUNT(*) > 0) AS total FROM procuracoes \
         WHERE escritorio_id <> '00000000-0000-0000-0000-000000000000' \
         AND ambiente = 'TRIAL'"
    ));
    assert!(existe(
        &ambiente.banco,
        "SELECT (COUNT(*) > 0) AS total FROM certificados_a1 WHERE ativo IS TRUE"
    ));
}

fn semear_cliente(banco: &BancoTemporario, escritorio: Uuid, documento: &str) -> Uuid {
    let mut conexao = banco.conexao();
    let id = Uuid::now_v7();
    sql_query(
        "INSERT INTO clientes (id, escritorio_id, cpf_cnpj, nome_razao_social, tipo, criado_em, atualizado_em) \
         VALUES ($1, $2, $3, $4, 'PJ', NOW(), NOW())",
    )
    .bind::<SqlUuid, _>(id)
    .bind::<SqlUuid, _>(escritorio)
    .bind::<Text, _>(documento)
    .bind::<Text, _>(format!("Cliente {documento}"))
    .execute(&mut conexao)
    .expect("cliente deve ser inserido");
    id
}

#[tokio::test]
async fn tokens_serpro_exigem_cliente_para_autenticar_procurador() {
    let ambiente = ambiente();
    let escritorio = semear_escritorio(&ambiente.banco, "alfa");
    let cliente = semear_cliente(&ambiente.banco, escritorio, "11222333000181");

    let mut conexao = ambiente.banco.conexao();

    // AUTENTICAR_PROCURADOR SEM cliente_id deve ser rejeitado pelo CHECK.
    let sem_cliente = sql_query(
        "INSERT INTO tokens_serpro \
         (id, escritorio_id, produto, ambiente, tipo_token, autor_pedido_dados, token_cifrado, expira_em) \
         VALUES ($1, $2, 'INTEGRA_CONTADOR', 'TRIAL', 'AUTENTICAR_PROCURADOR', '', $3, NOW() + INTERVAL '60 minutes')",
    )
    .bind::<SqlUuid, _>(Uuid::now_v7())
    .bind::<SqlUuid, _>(escritorio)
    .bind::<diesel::sql_types::Binary, _>(vec![1_u8])
    .execute(&mut conexao);
    assert!(
        sem_cliente.is_err(),
        "CHECK exige_cliente deve rejeitar AUTENTICAR_PROCURADOR sem cliente_id"
    );

    // AUTENTICAR_PROCURADOR COM cliente_id deve ser aceito.
    let com_cliente = sql_query(
        "INSERT INTO tokens_serpro \
         (id, escritorio_id, cliente_id, produto, ambiente, tipo_token, autor_pedido_dados, token_cifrado, expira_em) \
         VALUES ($1, $2, $3, 'INTEGRA_CONTADOR', 'TRIAL', 'AUTENTICAR_PROCURADOR', '', $4, NOW() + INTERVAL '60 minutes')",
    )
    .bind::<SqlUuid, _>(Uuid::now_v7())
    .bind::<SqlUuid, _>(escritorio)
    .bind::<SqlUuid, _>(cliente)
    .bind::<diesel::sql_types::Binary, _>(vec![2_u8])
    .execute(&mut conexao);
    assert!(
        com_cliente.is_ok(),
        "AUTENTICAR_PROCURADOR com cliente_id deve ser aceito"
    );

    // Outros tipos de token (ACCESS/JWT) continuam aceitando cliente_id NULL.
    let sem_cliente_outro_tipo = sql_query(
        "INSERT INTO tokens_serpro \
         (id, escritorio_id, produto, ambiente, tipo_token, autor_pedido_dados, token_cifrado, expira_em) \
         VALUES ($1, $2, 'INTEGRA_CONTADOR', 'TRIAL', 'ACCESS', '', $3, NOW() + INTERVAL '60 minutes')",
    )
    .bind::<SqlUuid, _>(Uuid::now_v7())
    .bind::<SqlUuid, _>(escritorio)
    .bind::<diesel::sql_types::Binary, _>(vec![3_u8])
    .execute(&mut conexao);
    assert!(
        sem_cliente_outro_tipo.is_ok(),
        "ACCESS com cliente_id NULL deve ser aceito"
    );
}

#[tokio::test]
async fn tokens_serpro_upsert_por_constraint_uq_tokens_serpro() {
    let ambiente = ambiente();
    let escritorio = semear_escritorio(&ambiente.banco, "alfa");
    let cliente = semear_cliente(&ambiente.banco, escritorio, "11222333000181");

    let mut conexao = ambiente.banco.conexao();

    // Insere o token inicial.
    let primeiro = sql_query(
        "INSERT INTO tokens_serpro \
         (id, escritorio_id, cliente_id, produto, ambiente, tipo_token, autor_pedido_dados, token_cifrado, expira_em) \
         VALUES ($1, $2, $3, 'INTEGRA_CONTADOR', 'TRIAL', 'AUTENTICAR_PROCURADOR', '', $4, NOW() + INTERVAL '60 minutes')",
    )
    .bind::<SqlUuid, _>(Uuid::now_v7())
    .bind::<SqlUuid, _>(escritorio)
    .bind::<SqlUuid, _>(cliente)
    .bind::<diesel::sql_types::Binary, _>(vec![1_u8])
    .execute(&mut conexao)
    .expect("token inicial deve ser inserido");
    assert_eq!(primeiro, 1);

    // Reinserir a mesma chave (escritorio, cliente, produto, ambiente,
    // tipo_token, autor_pedido_dados) deve falhar pela constraint única
    // uq_tokens_serpro (UNIQUE NULLS NOT DISTINCT). O upsert por essa
    // constraint é executado pelo código Rust (salvar_jwt_procurador), não
    // por SQL cru aqui.
    let duplicado = sql_query(
        "INSERT INTO tokens_serpro \
         (id, escritorio_id, cliente_id, produto, ambiente, tipo_token, autor_pedido_dados, token_cifrado, expira_em) \
         VALUES ($1, $2, $3, 'INTEGRA_CONTADOR', 'TRIAL', 'AUTENTICAR_PROCURADOR', '', $4, NOW() + INTERVAL '60 minutes')",
    )
    .bind::<SqlUuid, _>(Uuid::now_v7())
    .bind::<SqlUuid, _>(escritorio)
    .bind::<SqlUuid, _>(cliente)
    .bind::<diesel::sql_types::Binary, _>(vec![9_u8])
    .execute(&mut conexao);
    assert!(
        duplicado.is_err(),
        "UNIQUE uq_tokens_serpro deve impedir duplicidade de (escritorio, cliente, produto, ambiente, tipo_token, autor)"
    );

    // Confirma que ainda existe exatamente uma linha para a chave.
    let unica = sql_query(
        "SELECT (COUNT(*) = 1) AS total FROM tokens_serpro \
         WHERE escritorio_id = $1 \
           AND cliente_id = $2 \
           AND produto = 'INTEGRA_CONTADOR' \
           AND ambiente = 'TRIAL' \
           AND tipo_token = 'AUTENTICAR_PROCURADOR'",
    )
    .bind::<SqlUuid, _>(escritorio)
    .bind::<SqlUuid, _>(cliente)
    .get_result::<Total>(&mut conexao)
    .expect("consulta deve funcionar");
    assert!(unica.total, "deve haver exatamente um token por chave");
}
