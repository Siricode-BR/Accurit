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
fn aplica_e_reverte_as_tres_migrations() {
    let banco = BancoTemporario::novo();
    let conexao = banco.conexao();

    let aplicadas =
        aplicar_migrations_com_lock(conexao, MIGRATIONS).expect("migrations devem aplicar");
    assert_eq!(aplicadas, 3);

    let mut conexao = banco.conexao();
    assert!(tabela_existe(&mut conexao, "escritorios"));
    assert!(tabela_existe(&mut conexao, "usuarios"));
    assert!(tabela_existe(&mut conexao, "auditoria_eventos"));
    assert_eq!(
        migrations_pendentes(&mut conexao).expect("consulta válida"),
        0
    );

    let revertidas = reverter_todas(&mut conexao).expect("migrations devem reverter");
    assert_eq!(revertidas, 3);
    assert!(!tabela_existe(&mut conexao, "escritorios"));
    assert!(!tabela_existe(&mut conexao, "usuarios"));
    assert!(!tabela_existe(&mut conexao, "auditoria_eventos"));
    assert_eq!(
        migrations_pendentes(&mut conexao).expect("consulta válida"),
        3
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

    assert_eq!(total, 3);
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

    let admin_id = Uuid::now_v7();
    assert!(
        inserir_usuario(
            &mut conexao,
            admin_id,
            escritorio_id,
            "admin@alfa.test",
            None,
            "ADMIN"
        )
        .is_err(),
        "ADMIN sem CPF deve falhar"
    );

    inserir_usuario(
        &mut conexao,
        admin_id,
        escritorio_id,
        "admin@alfa.test",
        Some("12345678901"),
        "ADMIN",
    )
    .expect("ADMIN com CPF deve ser inserido");

    assert!(
        inserir_usuario(
            &mut conexao,
            Uuid::now_v7(),
            escritorio_id,
            "admin@alfa.test",
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
        None,
        "CLIENTE",
    )
    .expect("CLIENTE sem CPF deve ser inserido");

    let outro_escritorio = Uuid::now_v7();
    inserir_escritorio(&mut conexao, outro_escritorio, "gama", None)
        .expect("segundo escritório deve ser inserido");
    inserir_usuario(
        &mut conexao,
        Uuid::now_v7(),
        outro_escritorio,
        "admin@alfa.test",
        None,
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
        .bind::<Nullable<SqlUuid>, _>(Some(admin_id))
        .bind::<Text, _>("USUARIO_CRIADO")
        .bind::<Nullable<Text>, _>(Some("usuarios"))
        .bind::<Nullable<SqlUuid>, _>(Some(admin_id))
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
