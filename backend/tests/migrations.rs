mod common;

use std::path::Path;
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;

use diesel::pg::PgConnection;
use diesel::sql_query;
use diesel::sql_types::{BigInt, Bool, Nullable, Text, Uuid as SqlUuid};
use diesel::{Connection, QueryableByName, RunQueryDsl};
use diesel_migrations::FileBasedMigrations;
use uuid::Uuid;

use accurit_backend::infrastructure::persistence::migrations::{
    ADVISORY_LOCK_ID, MIGRATIONS, MigrationError, aplicar_migrations_com_lock,
    conectar_com_retry_configurado, migrations_pendentes, reverter_todas,
};

use common::BancoTemporario;

#[derive(Debug, QueryableByName)]
struct Regclass {
    #[diesel(sql_type = Nullable<Text>)]
    reg: Option<String>,
}

#[derive(Debug, QueryableByName)]
struct Existe {
    #[diesel(sql_type = Bool)]
    existe: bool,
}

#[derive(Debug, QueryableByName)]
struct LockLivre {
    #[diesel(sql_type = Bool)]
    livre: bool,
}

fn tabela_existe(conexao: &mut PgConnection, tabela: &str) -> bool {
    let consulta = format!("SELECT to_regclass('public.{tabela}')::text AS reg");
    let resultado = sql_query(consulta)
        .get_result::<Regclass>(conexao)
        .expect("consulta de tabela deve funcionar");
    resultado.reg.is_some()
}

fn inserir_escritorio(
    conexao: &mut PgConnection,
    id: Uuid,
    slug: &str,
    cnpj: Option<&str>,
) -> diesel::QueryResult<usize> {
    sql_query("INSERT INTO escritorios (id, slug, nome, cnpj) VALUES ($1, $2, $3, $4)")
        .bind::<SqlUuid, _>(id)
        .bind::<Text, _>(slug)
        .bind::<Text, _>("Escritório de teste")
        .bind::<Nullable<Text>, _>(cnpj)
        .execute(conexao)
}

fn inserir_usuario(
    conexao: &mut PgConnection,
    id: Uuid,
    escritorio_id: Uuid,
    email: &str,
    cpf: Option<&str>,
    papel: &str,
) -> diesel::QueryResult<usize> {
    sql_query(
        "INSERT INTO usuarios (id, escritorio_id, email, cpf, senha_hash, papel) \
         VALUES ($1, $2, $3, $4, 'hash-de-teste', $5)",
    )
    .bind::<SqlUuid, _>(id)
    .bind::<SqlUuid, _>(escritorio_id)
    .bind::<Text, _>(email)
    .bind::<Nullable<Text>, _>(cpf)
    .bind::<Text, _>(papel)
    .execute(conexao)
}

#[test]
fn aplica_e_reverte_as_doze_migrations() {
    let banco = BancoTemporario::novo();
    let conexao = banco.conexao();

    let aplicadas =
        aplicar_migrations_com_lock(conexao, MIGRATIONS).expect("migrations devem aplicar");
    assert_eq!(aplicadas, 12);

    let mut conexao = banco.conexao();
    assert!(tabela_existe(&mut conexao, "escritorios"));
    assert!(tabela_existe(&mut conexao, "usuarios"));
    assert!(tabela_existe(&mut conexao, "auditoria_eventos"));
    assert!(tabela_existe(&mut conexao, "sessoes"));
    assert!(tabela_existe(&mut conexao, "administradores"));
    assert!(tabela_existe(&mut conexao, "sessoes_admin"));
    assert!(tabela_existe(&mut conexao, "clientes"));
    assert!(tabela_existe(&mut conexao, "credenciais_serpro"));
    assert!(tabela_existe(&mut conexao, "certificados_a1"));
    assert!(tabela_existe(&mut conexao, "tokens_serpro"));
    assert!(tabela_existe(&mut conexao, "procuracoes"));
    assert!(tabela_existe(&mut conexao, "consultas_fiscais"));
    assert_eq!(
        migrations_pendentes(&mut conexao).expect("consulta válida"),
        0
    );

    let revertidas = reverter_todas(&mut conexao).expect("migrations devem reverter");
    assert_eq!(revertidas, 12);
    assert!(!tabela_existe(&mut conexao, "escritorios"));
    assert!(!tabela_existe(&mut conexao, "usuarios"));
    assert!(!tabela_existe(&mut conexao, "auditoria_eventos"));
    assert!(!tabela_existe(&mut conexao, "sessoes"));
    assert!(!tabela_existe(&mut conexao, "administradores"));
    assert!(!tabela_existe(&mut conexao, "sessoes_admin"));
    assert!(!tabela_existe(&mut conexao, "clientes"));
    assert!(!tabela_existe(&mut conexao, "credenciais_serpro"));
    assert!(!tabela_existe(&mut conexao, "certificados_a1"));
    assert!(!tabela_existe(&mut conexao, "tokens_serpro"));
    assert!(!tabela_existe(&mut conexao, "procuracoes"));
    assert!(!tabela_existe(&mut conexao, "consultas_fiscais"));
    assert_eq!(
        migrations_pendentes(&mut conexao).expect("consulta válida"),
        12
    );
}

#[test]
fn aplica_migrations_concorrentes_serializadas_pelo_lock() {
    let banco = BancoTemporario::novo();
    let url = banco.database_url.clone();
    let barreira = Arc::new(Barrier::new(2));

    let handles: Vec<_> = (0..2)
        .map(|_| {
            let url = url.clone();
            let barreira = Arc::clone(&barreira);
            thread::spawn(move || {
                let conexao = PgConnection::establish(&url).expect("conexão de teste");
                barreira.wait();
                aplicar_migrations_com_lock(conexao, MIGRATIONS)
            })
        })
        .collect();

    let total: usize = handles
        .into_iter()
        .map(|handle| {
            handle
                .join()
                .expect("thread não deve falhar")
                .expect("migration deve aplicar")
        })
        .sum();

    assert_eq!(total, 12);
}

#[test]
fn libera_lock_quando_uma_migration_falha() {
    let banco = BancoTemporario::novo();
    let conexao = banco.conexao();
    let caminho = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("migrations_quebradas");
    let fonte = FileBasedMigrations::from_path(caminho).expect("fonte de teste válida");

    let erro = aplicar_migrations_com_lock(conexao, fonte).expect_err("migration deve falhar");
    assert!(matches!(erro, MigrationError::Aplicacao(_)));

    let mut conexao = banco.conexao();
    assert!(
        !tabela_existe(&mut conexao, "tabela_quebrada"),
        "migration com falha deve ser revertida pela transação"
    );

    let livre = sql_query("SELECT pg_try_advisory_lock($1) AS livre")
        .bind::<BigInt, _>(ADVISORY_LOCK_ID)
        .get_result::<LockLivre>(&mut conexao)
        .expect("consulta de lock deve funcionar")
        .livre;

    assert!(livre, "o lock deve ter sido liberado após a falha");
    sql_query("SELECT pg_advisory_unlock($1)")
        .bind::<BigInt, _>(ADVISORY_LOCK_ID)
        .execute(&mut conexao)
        .expect("deve liberar o lock de teste");
}

#[test]
fn valida_constraints_do_schema() {
    let banco = BancoTemporario::novo();
    let conexao = banco.conexao();
    aplicar_migrations_com_lock(conexao, MIGRATIONS).expect("migrations devem aplicar");
    let mut conexao = banco.conexao();

    let escritorio_id = Uuid::now_v7();
    inserir_escritorio(&mut conexao, escritorio_id, "alfa", Some("12345678000199"))
        .expect("escritório válido deve ser inserido");

    assert!(
        inserir_escritorio(&mut conexao, Uuid::now_v7(), "alfa", None).is_err(),
        "slug duplicado deve falhar"
    );
    assert!(
        inserir_escritorio(&mut conexao, Uuid::now_v7(), "Maiúsculo", None).is_err(),
        "slug com caractere inválido deve falhar"
    );
    assert!(
        inserir_escritorio(&mut conexao, Uuid::now_v7(), "beta", Some("123")).is_err(),
        "CNPJ com tamanho inválido deve falhar"
    );

    let operador_id = Uuid::now_v7();
    inserir_usuario(
        &mut conexao,
        operador_id,
        escritorio_id,
        "operador@alfa.test",
        Some("12345678901"),
        "OPERADOR",
    )
    .expect("OPERADOR com CPF deve ser inserido");

    assert!(
        inserir_usuario(
            &mut conexao,
            Uuid::now_v7(),
            escritorio_id,
            "sem-cpf@alfa.test",
            None,
            "OPERADOR"
        )
        .is_err(),
        "usuário sem CPF deve falhar"
    );

    assert!(
        inserir_usuario(
            &mut conexao,
            Uuid::now_v7(),
            escritorio_id,
            "admin@alfa.test",
            Some("12345678901"),
            "ADMIN"
        )
        .is_err(),
        "papel ADMIN é exclusivo do backoffice (tabela administradores)"
    );

    assert!(
        inserir_usuario(
            &mut conexao,
            Uuid::now_v7(),
            escritorio_id,
            "operador@alfa.test",
            Some("98765432100"),
            "OPERADOR"
        )
        .is_err(),
        "e-mail duplicado no mesmo escritório deve falhar"
    );

    assert!(
        inserir_usuario(
            &mut conexao,
            Uuid::now_v7(),
            escritorio_id,
            "outro@alfa.test",
            Some("12345678901"),
            "OPERADOR"
        )
        .is_err(),
        "CPF duplicado no mesmo escritório deve falhar"
    );

    let cliente_id = Uuid::now_v7();
    inserir_usuario(
        &mut conexao,
        cliente_id,
        escritorio_id,
        "cliente@alfa.test",
        Some("99988877766"),
        "CLIENTE",
    )
    .expect("CLIENTE com CPF deve ser inserido");

    let outro_escritorio = Uuid::now_v7();
    inserir_escritorio(&mut conexao, outro_escritorio, "gama", None)
        .expect("segundo escritório deve ser inserido");
    inserir_usuario(
        &mut conexao,
        Uuid::now_v7(),
        outro_escritorio,
        "operador@alfa.test",
        None,
        "CLIENTE",
    )
    .expect_err("mesmo e-mail em outro escritório ainda exige CPF");

    inserir_usuario(
        &mut conexao,
        Uuid::now_v7(),
        outro_escritorio,
        "operador@alfa.test",
        Some("12345678901"),
        "CLIENTE",
    )
    .expect("mesmo e-mail em outro escritório deve ser permitido");

    assert!(
        inserir_usuario(
            &mut conexao,
            Uuid::now_v7(),
            escritorio_id,
            "invalido@alfa.test",
            Some("abc"),
            "OPERADOR"
        )
        .is_err(),
        "CPF não numérico deve falhar"
    );

    let auditoria_sql = "INSERT INTO auditoria_eventos \
        (id, escritorio_id, usuario_id, acao, entidade, entidade_id, valor_anterior, valor_novo, ip) \
        VALUES ($1, $2, $3, $4, $5, $6, $7::jsonb, $8::jsonb, $9::inet)";

    sql_query(auditoria_sql)
        .bind::<SqlUuid, _>(Uuid::now_v7())
        .bind::<SqlUuid, _>(escritorio_id)
        .bind::<Nullable<SqlUuid>, _>(Some(operador_id))
        .bind::<Text, _>("USUARIO_CRIADO")
        .bind::<Nullable<Text>, _>(Some("usuarios"))
        .bind::<Nullable<SqlUuid>, _>(Some(operador_id))
        .bind::<Nullable<Text>, _>(Some("{}"))
        .bind::<Nullable<Text>, _>(Some("{\"papel\":\"ADMIN\"}"))
        .bind::<Nullable<Text>, _>(Some("203.0.113.10"))
        .execute(&mut conexao)
        .expect("evento de auditoria válido deve ser inserido");

    sql_query(auditoria_sql)
        .bind::<SqlUuid, _>(Uuid::now_v7())
        .bind::<SqlUuid, _>(escritorio_id)
        .bind::<Nullable<SqlUuid>, _>(None::<Uuid>)
        .bind::<Text, _>("TESTE")
        .bind::<Nullable<Text>, _>(None::<&str>)
        .bind::<Nullable<SqlUuid>, _>(None::<Uuid>)
        .bind::<Nullable<Text>, _>(None::<&str>)
        .bind::<Nullable<Text>, _>(None::<&str>)
        .bind::<Nullable<Text>, _>(None::<&str>)
        .execute(&mut conexao)
        .expect("evento de auditoria com campos opcionais nulos deve ser inserido");

    sql_query(auditoria_sql)
        .bind::<SqlUuid, _>(Uuid::now_v7())
        .bind::<Nullable<SqlUuid>, _>(None::<Uuid>)
        .bind::<Nullable<SqlUuid>, _>(None::<Uuid>)
        .bind::<Text, _>("TESTE_SEM_TENANT")
        .bind::<Nullable<Text>, _>(None::<&str>)
        .bind::<Nullable<SqlUuid>, _>(None::<Uuid>)
        .bind::<Nullable<Text>, _>(None::<&str>)
        .bind::<Nullable<Text>, _>(None::<&str>)
        .bind::<Nullable<Text>, _>(None::<&str>)
        .execute(&mut conexao)
        .expect("evento de backoffice sem escritório deve ser inserido");

    sql_query(auditoria_sql)
        .bind::<SqlUuid, _>(Uuid::now_v7())
        .bind::<SqlUuid, _>(Uuid::now_v7())
        .bind::<Nullable<SqlUuid>, _>(None::<Uuid>)
        .bind::<Text, _>("TESTE")
        .bind::<Nullable<Text>, _>(None::<&str>)
        .bind::<Nullable<SqlUuid>, _>(None::<Uuid>)
        .bind::<Nullable<Text>, _>(None::<&str>)
        .bind::<Nullable<Text>, _>(None::<&str>)
        .bind::<Nullable<Text>, _>(None::<&str>)
        .execute(&mut conexao)
        .expect_err("escritório inexistente deve violar a FK");
}

#[allow(clippy::too_many_arguments)]
fn inserir_sessao(
    conexao: &mut PgConnection,
    id: Uuid,
    escritorio_id: Uuid,
    usuario_id: Uuid,
    audiencia: &str,
    refresh_hash: &str,
    expira_em: &str,
) -> diesel::QueryResult<usize> {
    sql_query(
        "INSERT INTO sessoes \
         (id, escritorio_id, usuario_id, audiencia, familia_id, refresh_hash, expira_em, ip, user_agent) \
         VALUES ($1, $2, $3, $4, $5, $6, NOW() + $7::interval, '203.0.113.10', 'teste')",
    )
    .bind::<SqlUuid, _>(id)
    .bind::<SqlUuid, _>(escritorio_id)
    .bind::<SqlUuid, _>(usuario_id)
    .bind::<Text, _>(audiencia)
    .bind::<SqlUuid, _>(Uuid::now_v7())
    .bind::<Text, _>(refresh_hash)
    .bind::<Text, _>(expira_em)
    .execute(conexao)
}

#[test]
fn valida_constraints_de_sessoes() {
    let banco = BancoTemporario::novo();
    let conexao = banco.conexao();
    aplicar_migrations_com_lock(conexao, MIGRATIONS).expect("migrations devem aplicar");
    let mut conexao = banco.conexao();

    let escritorio_id = Uuid::now_v7();
    inserir_escritorio(&mut conexao, escritorio_id, "sessoes", None).expect("escritório válido");
    let usuario_id = Uuid::now_v7();
    inserir_usuario(
        &mut conexao,
        usuario_id,
        escritorio_id,
        "operador@sessoes.test",
        Some("12345678901"),
        "OPERADOR",
    )
    .expect("usuário válido");

    inserir_sessao(
        &mut conexao,
        Uuid::now_v7(),
        escritorio_id,
        usuario_id,
        "CLIENT",
        &"a".repeat(64),
        "8 hours",
    )
    .expect("sessão válida deve ser inserida");

    assert!(
        inserir_sessao(
            &mut conexao,
            Uuid::now_v7(),
            escritorio_id,
            usuario_id,
            "ADMIN",
            &"b".repeat(64),
            "8 hours",
        )
        .is_err(),
        "sessão de portal com audiência ADMIN deve falhar"
    );

    assert!(
        inserir_sessao(
            &mut conexao,
            Uuid::now_v7(),
            escritorio_id,
            usuario_id,
            "SUPER",
            &"b".repeat(64),
            "8 hours",
        )
        .is_err(),
        "audiência fora de CLIENT deve falhar"
    );

    assert!(
        inserir_sessao(
            &mut conexao,
            Uuid::now_v7(),
            escritorio_id,
            usuario_id,
            "CLIENT",
            &"a".repeat(64),
            "8 hours",
        )
        .is_err(),
        "refresh_hash duplicado deve falhar"
    );

    assert!(
        inserir_sessao(
            &mut conexao,
            Uuid::now_v7(),
            escritorio_id,
            usuario_id,
            "CLIENT",
            &"c".repeat(64),
            "-1 hour",
        )
        .is_err(),
        "expira_em anterior a criado_em deve falhar"
    );

    assert!(
        inserir_sessao(
            &mut conexao,
            Uuid::now_v7(),
            Uuid::now_v7(),
            usuario_id,
            "CLIENT",
            &"d".repeat(64),
            "8 hours",
        )
        .is_err(),
        "escritório inexistente deve violar a FK"
    );

    assert!(
        inserir_sessao(
            &mut conexao,
            Uuid::now_v7(),
            escritorio_id,
            Uuid::now_v7(),
            "CLIENT",
            &"e".repeat(64),
            "8 hours",
        )
        .is_err(),
        "usuário inexistente deve violar a FK"
    );
}

fn inserir_administrador(
    conexao: &mut PgConnection,
    id: Uuid,
    email: &str,
    cpf: &str,
) -> diesel::QueryResult<usize> {
    sql_query(
        "INSERT INTO administradores (id, email, cpf, senha_hash) \
         VALUES ($1, $2, $3, 'hash-de-teste')",
    )
    .bind::<SqlUuid, _>(id)
    .bind::<Text, _>(email)
    .bind::<Text, _>(cpf)
    .execute(conexao)
}

fn inserir_sessao_admin(
    conexao: &mut PgConnection,
    id: Uuid,
    administrador_id: Uuid,
    refresh_hash: &str,
    expira_em: &str,
) -> diesel::QueryResult<usize> {
    sql_query(
        "INSERT INTO sessoes_admin \
         (id, administrador_id, familia_id, refresh_hash, expira_em, ip, user_agent) \
         VALUES ($1, $2, $3, $4, NOW() + $5::interval, '203.0.113.10', 'teste')",
    )
    .bind::<SqlUuid, _>(id)
    .bind::<SqlUuid, _>(administrador_id)
    .bind::<SqlUuid, _>(Uuid::now_v7())
    .bind::<Text, _>(refresh_hash)
    .bind::<Text, _>(expira_em)
    .execute(conexao)
}

#[test]
fn valida_constraints_de_administradores_e_sessoes_admin() {
    let banco = BancoTemporario::novo();
    let conexao = banco.conexao();
    aplicar_migrations_com_lock(conexao, MIGRATIONS).expect("migrations devem aplicar");
    let mut conexao = banco.conexao();

    let admin_id = Uuid::now_v7();
    inserir_administrador(&mut conexao, admin_id, "gestor@accurit.test", "52998224725")
        .expect("administrador válido deve ser inserido");

    assert!(
        inserir_administrador(
            &mut conexao,
            Uuid::now_v7(),
            "gestor@accurit.test",
            "99999999999"
        )
        .is_err(),
        "e-mail duplicado de administrador deve falhar"
    );

    assert!(
        inserir_administrador(
            &mut conexao,
            Uuid::now_v7(),
            "outro@accurit.test",
            "52998224725"
        )
        .is_err(),
        "CPF duplicado de administrador deve falhar"
    );

    assert!(
        inserir_administrador(&mut conexao, Uuid::now_v7(), "sem-cpf@accurit.test", "").is_err(),
        "CPF com formato inválido deve falhar"
    );

    inserir_sessao_admin(
        &mut conexao,
        Uuid::now_v7(),
        admin_id,
        &"f".repeat(64),
        "8 hours",
    )
    .expect("sessão do backoffice deve ser inserida");

    assert!(
        inserir_sessao_admin(
            &mut conexao,
            Uuid::now_v7(),
            admin_id,
            &"f".repeat(64),
            "8 hours",
        )
        .is_err(),
        "refresh_hash duplicado deve falhar"
    );

    assert!(
        inserir_sessao_admin(
            &mut conexao,
            Uuid::now_v7(),
            Uuid::now_v7(),
            &"g".repeat(64),
            "8 hours",
        )
        .is_err(),
        "administrador inexistente deve violar a FK"
    );

    assert!(
        inserir_sessao_admin(
            &mut conexao,
            Uuid::now_v7(),
            admin_id,
            &"h".repeat(64),
            "-1 hour",
        )
        .is_err(),
        "expira_em anterior a criado_em deve falhar"
    );
}

fn inserir_cliente(
    conexao: &mut PgConnection,
    id: Uuid,
    escritorio_id: Uuid,
    tipo: Option<&str>,
    porte_regime: Option<&str>,
    cpf_cnpj: &str,
    nome: &str,
) -> diesel::QueryResult<usize> {
    sql_query(
        "INSERT INTO clientes \
         (id, escritorio_id, tipo, porte_regime, cpf_cnpj, nome_razao_social) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind::<SqlUuid, _>(id)
    .bind::<SqlUuid, _>(escritorio_id)
    .bind::<Nullable<Text>, _>(tipo)
    .bind::<Nullable<Text>, _>(porte_regime)
    .bind::<Text, _>(cpf_cnpj)
    .bind::<Text, _>(nome)
    .execute(conexao)
}

#[test]
fn valida_constraints_de_clientes() {
    let banco = BancoTemporario::novo();
    let conexao = banco.conexao();
    aplicar_migrations_com_lock(conexao, MIGRATIONS).expect("migrations devem aplicar");
    let mut conexao = banco.conexao();

    let escritorio_id = Uuid::now_v7();
    inserir_escritorio(&mut conexao, escritorio_id, "clientes", None).expect("escritório válido");
    let outro_escritorio = Uuid::now_v7();
    inserir_escritorio(&mut conexao, outro_escritorio, "clientes-b", None)
        .expect("segundo escritório válido");

    let cpf = "11144477735";
    let cnpj = "11444777000161";

    inserir_cliente(
        &mut conexao,
        Uuid::now_v7(),
        escritorio_id,
        Some("PF"),
        None,
        cpf,
        "Pessoa Física",
    )
    .expect("cliente PF sem porte deve ser inserido");

    assert!(
        inserir_cliente(
            &mut conexao,
            Uuid::now_v7(),
            escritorio_id,
            Some("PF"),
            Some("MEI"),
            "12345678909",
            "PF com porte deve falhar"
        )
        .is_err(),
        "cliente PF não pode ter porte_regime"
    );

    inserir_cliente(
        &mut conexao,
        Uuid::now_v7(),
        escritorio_id,
        Some("PJ"),
        Some("MEI"),
        cnpj,
        "PJ MEI",
    )
    .expect("cliente PJ com regime deve ser inserido");

    inserir_cliente(
        &mut conexao,
        Uuid::now_v7(),
        escritorio_id,
        Some("PJ"),
        None,
        "45723174000103",
        "PJ sem regime informado",
    )
    .expect("cliente PJ sem regime deve ser inserido");

    assert!(
        inserir_cliente(
            &mut conexao,
            Uuid::now_v7(),
            escritorio_id,
            Some("PJ"),
            Some("MEI"),
            cnpj,
            "duplicado no mesmo escritório"
        )
        .is_err(),
        "cpf_cnpj duplicado no mesmo escritório deve falhar"
    );

    inserir_cliente(
        &mut conexao,
        Uuid::now_v7(),
        outro_escritorio,
        Some("PJ"),
        Some("MEI"),
        cnpj,
        "mesmo CNPJ em outro escritório",
    )
    .expect("mesmo cpf_cnpj em outro escritório deve ser permitido");

    assert!(
        inserir_cliente(
            &mut conexao,
            Uuid::now_v7(),
            escritorio_id,
            Some("XX"),
            None,
            cpf,
            "tipo inválido"
        )
        .is_err(),
        "tipo fora de PF/PJ deve falhar"
    );

    assert!(
        inserir_cliente(
            &mut conexao,
            Uuid::now_v7(),
            escritorio_id,
            Some("PJ"),
            Some("SUPERSIMPLES"),
            cnpj,
            "regime inválido"
        )
        .is_err(),
        "porte_regime fora do domínio deve falhar"
    );

    assert!(
        inserir_cliente(
            &mut conexao,
            Uuid::now_v7(),
            escritorio_id,
            Some("PF"),
            None,
            "123456",
            "documento curto"
        )
        .is_err(),
        "cpf_cnpj com tamanho inválido deve falhar"
    );

    assert!(
        inserir_cliente(
            &mut conexao,
            Uuid::now_v7(),
            Uuid::now_v7(),
            Some("PF"),
            None,
            cpf,
            "escritório inexistente"
        )
        .is_err(),
        "escritório inexistente deve violar a FK"
    );
}

fn inserir_certificado(
    conexao: &mut PgConnection,
    escritorio_id: Uuid,
    titular: &str,
    fingerprint: &str,
    valido_ate: &str,
) -> diesel::QueryResult<usize> {
    sql_query(
        "INSERT INTO certificados_a1 \
         (id, escritorio_id, tipo, titular_cpf_cnpj, fingerprint_sha256, conteudo_cifrado, valido_de, valido_ate) \
         VALUES ($1, $2, 'A1', $3, $4, $5, NOW() - interval '30 days', NOW() + $6::interval)",
    )
    .bind::<SqlUuid, _>(Uuid::now_v7())
    .bind::<SqlUuid, _>(escritorio_id)
    .bind::<Text, _>(titular)
    .bind::<Text, _>(fingerprint)
    .bind::<diesel::sql_types::Bytea, _>(vec![1_u8, 2, 3])
    .bind::<Text, _>(valido_ate)
    .execute(conexao)
}

#[test]
fn valida_constraints_de_certificados_e_dados_fiscais() {
    let banco = BancoTemporario::novo();
    let conexao = banco.conexao();
    aplicar_migrations_com_lock(conexao, MIGRATIONS).expect("migrations devem aplicar");
    let mut conexao = banco.conexao();

    let escritorio_id = Uuid::now_v7();
    inserir_escritorio(&mut conexao, escritorio_id, "certificados", None).expect("escritório");
    let cliente_id = Uuid::now_v7();
    let cnpj = "11444777000161";
    inserir_cliente(
        &mut conexao,
        cliente_id,
        escritorio_id,
        Some("PJ"),
        Some("MEI"),
        cnpj,
        "Cliente de consulta",
    )
    .expect("cliente válido");

    inserir_certificado(
        &mut conexao,
        escritorio_id,
        cnpj,
        &"a".repeat(64),
        "365 days",
    )
    .expect("certificado válido deve ser inserido");

    assert!(
        inserir_certificado(
            &mut conexao,
            escritorio_id,
            cnpj,
            &"a".repeat(64),
            "365 days"
        )
        .is_err(),
        "fingerprint duplicado no mesmo escritório deve falhar"
    );

    assert!(
        inserir_certificado(
            &mut conexao,
            escritorio_id,
            cnpj,
            &"a".repeat(64),
            "-30 days"
        )
        .is_err(),
        "valido_ate anterior a valido_de deve falhar"
    );

    let credenciais_sql = "INSERT INTO credenciais_serpro \
         (id, escritorio_id, produto, ambiente, consumer_key_cifrado, consumer_secret_cifrado) \
         VALUES ($1, $2, $3, $4, $5, $6)";
    let bytes = vec![9_u8; 16];

    sql_query(credenciais_sql)
        .bind::<SqlUuid, _>(Uuid::now_v7())
        .bind::<SqlUuid, _>(escritorio_id)
        .bind::<Text, _>("DIVIDA_ATIVA")
        .bind::<Text, _>("TRIAL")
        .bind::<diesel::sql_types::Bytea, _>(&bytes)
        .bind::<diesel::sql_types::Bytea, _>(&bytes)
        .execute(&mut conexao)
        .expect("credencial válida deve ser inserida");

    assert!(
        sql_query(credenciais_sql)
            .bind::<SqlUuid, _>(Uuid::now_v7())
            .bind::<SqlUuid, _>(escritorio_id)
            .bind::<Text, _>("DIVIDA_ATIVA")
            .bind::<Text, _>("TRIAL")
            .bind::<diesel::sql_types::Bytea, _>(&bytes)
            .bind::<diesel::sql_types::Bytea, _>(&bytes)
            .execute(&mut conexao)
            .is_err(),
        "credencial duplicada deve falhar"
    );

    assert!(
        sql_query(credenciais_sql)
            .bind::<SqlUuid, _>(Uuid::now_v7())
            .bind::<SqlUuid, _>(escritorio_id)
            .bind::<Text, _>("OUTRO_PRODUTO")
            .bind::<Text, _>("TRIAL")
            .bind::<diesel::sql_types::Bytea, _>(&bytes)
            .bind::<diesel::sql_types::Bytea, _>(&bytes)
            .execute(&mut conexao)
            .is_err(),
        "produto fora do domínio deve falhar"
    );

    let consultas_sql = "INSERT INTO consultas_fiscais \
         (id, escritorio_id, cliente_id, produto, servico, ambiente, estado) \
         VALUES ($1, $2, $3, $4, $5, $6, $7)";

    sql_query(consultas_sql)
        .bind::<SqlUuid, _>(Uuid::now_v7())
        .bind::<SqlUuid, _>(escritorio_id)
        .bind::<Nullable<SqlUuid>, _>(Some(cliente_id))
        .bind::<Text, _>("DIVIDA_ATIVA")
        .bind::<Text, _>("DEVEDOR")
        .bind::<Text, _>("TRIAL")
        .bind::<Text, _>("SUCESSO")
        .execute(&mut conexao)
        .expect("consulta fiscal válida deve ser inserida");

    assert!(
        sql_query(consultas_sql)
            .bind::<SqlUuid, _>(Uuid::now_v7())
            .bind::<SqlUuid, _>(escritorio_id)
            .bind::<Nullable<SqlUuid>, _>(Some(cliente_id))
            .bind::<Text, _>("DIVIDA_ATIVA")
            .bind::<Text, _>("DEVEDOR")
            .bind::<Text, _>("HOMOLOGACAO")
            .bind::<Text, _>("SUCESSO")
            .execute(&mut conexao)
            .is_err(),
        "ambiente fora de TRIAL/PRODUCAO deve falhar"
    );

    assert!(
        sql_query(consultas_sql)
            .bind::<SqlUuid, _>(Uuid::now_v7())
            .bind::<SqlUuid, _>(Uuid::now_v7())
            .bind::<Nullable<SqlUuid>, _>(None::<Uuid>)
            .bind::<Text, _>("DIVIDA_ATIVA")
            .bind::<Text, _>("DEVEDOR")
            .bind::<Text, _>("TRIAL")
            .bind::<Text, _>("SUCESSO")
            .execute(&mut conexao)
            .is_err(),
        "escritório inexistente deve violar a FK"
    );
}

#[test]
fn confirma_que_os_indices_obrigatorios_existem() {
    let banco = BancoTemporario::novo();
    let conexao = banco.conexao();
    aplicar_migrations_com_lock(conexao, MIGRATIONS).expect("migrations devem aplicar");
    let mut conexao = banco.conexao();

    let existe = sql_query(
        "SELECT EXISTS (SELECT 1 FROM pg_indexes WHERE indexname = 'idx_usuarios_escritorio') AS existe",
    )
    .get_result::<Existe>(&mut conexao)
    .expect("consulta de índice deve funcionar")
    .existe;
    assert!(existe);

    let existe = sql_query(
        "SELECT EXISTS (SELECT 1 FROM pg_indexes WHERE indexname = 'idx_auditoria_escritorio_criado') AS existe",
    )
    .get_result::<Existe>(&mut conexao)
    .expect("consulta de índice deve funcionar")
    .existe;
    assert!(existe);

    for indice in [
        "idx_sessoes_escritorio_usuario",
        "idx_sessoes_familia",
        "idx_sessoes_admin_familia",
        "idx_clientes_escritorio_tipo",
        "idx_clientes_escritorio_regime",
        "idx_credenciais_escritorio_produto",
        "idx_certificados_escritorio_validade",
        "idx_procuracoes_escritorio_servico",
        "idx_consultas_escritorio_criado",
        "idx_consultas_cliente",
    ] {
        let existe = sql_query(format!(
            "SELECT EXISTS (SELECT 1 FROM pg_indexes WHERE indexname = '{indice}') AS existe"
        ))
        .get_result::<Existe>(&mut conexao)
        .expect("consulta de índice deve funcionar")
        .existe;
        assert!(existe, "índice {indice} deve existir");
    }
}

#[test]
fn retry_de_conexao_termina_sem_panico() {
    let erro = conectar_com_retry_configurado(
        "postgres://postgres:postgres@127.0.0.1:1/accurit",
        2,
        Duration::from_millis(5),
    )
    .err()
    .expect("porta fechada deve falhar");

    assert!(matches!(erro, MigrationError::Conexao { tentativas: 2 }));
}
