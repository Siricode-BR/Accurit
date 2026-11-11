//! Fase B.1 — testes de integração do job runner: claim concorrente,
//! retry com backoff, lease expirado, lock por escritório, limite do plano,
//! isolamento multi-tenant e contagem de chamadas.

mod common;

use diesel::QueryableByName;
use diesel::RunQueryDsl;
use diesel::sql_query;
use diesel::sql_types::{BigInt, Text, Uuid as SqlUuid};
use uuid::Uuid;

use accurit_backend::application::trabalhos::{ConfiguracaoRunner, JobRunner};
use accurit_backend::infrastructure::config::Config;
use accurit_backend::infrastructure::jobs::montar_dependencias;
use accurit_backend::infrastructure::persistence::migrations::{
    MIGRATIONS, aplicar_migrations_com_lock,
};
use accurit_backend::infrastructure::persistence::pool::{DbPool, criar_pool};

use common::{BancoTemporario, configuracao_com_extras};

const CNPJ_A: &str = "11222333000181";
const CNPJ_B: &str = "11444777000161";

/// CPFs válidos com o último dígito controlando o cenário do mock TRIAL.
/// 2 e 3 = consulta com sucesso; 4 = não encontrado; 8 = indeterminado.
const CPF_SUCESSO_A: &str = "12345678062";
const CPF_SUCESSO_B: &str = "12345678143";
const CPF_NAO_ENCONTRADO: &str = "12345678909";
const CPF_INDETERMINADO: &str = "95835384068";

struct Ambiente {
    banco: BancoTemporario,
    pool: DbPool,
    config: Config,
}

impl Ambiente {
    fn novo(extras: &[(&str, &str)]) -> Self {
        let banco = BancoTemporario::novo();
        aplicar_migrations_com_lock(banco.conexao(), MIGRATIONS).expect("migrations");
        let mut todos = vec![
            ("SERPRO_RATE_LIMIT_CONSULTAS_POR_SEGUNDO", "1000"),
            ("JOBS_MAX_CONCORRENCIA", "4"),
        ];
        todos.extend_from_slice(extras);
        let config = configuracao_com_extras("", &todos);
        let pool = criar_pool(&banco.database_url).expect("pool");
        Self {
            banco,
            pool,
            config,
        }
    }

    fn runner(&self) -> JobRunner {
        self.runner_com(|_| {})
    }

    fn runner_com(&self, ajuste: impl FnOnce(&mut ConfiguracaoRunner)) -> JobRunner {
        let (dependencias, mut configuracao) = montar_dependencias(&self.pool, &self.config);
        ajuste(&mut configuracao);
        let (_tx, rx) = tokio::sync::watch::channel(false);
        JobRunner::novo(dependencias, configuracao, rx)
    }

    fn conexao(&self) -> diesel::pg::PgConnection {
        self.banco.conexao()
    }
}

#[derive(QueryableByName)]
struct Contagem {
    #[diesel(sql_type = BigInt)]
    total: i64,
}

#[derive(QueryableByName)]
struct TextoRow {
    #[diesel(sql_type = Text)]
    valor: String,
}

#[derive(QueryableByName)]
struct OpcionalTextoRow {
    #[diesel(sql_type = diesel::sql_types::Nullable<Text>)]
    valor: Option<String>,
}

fn contar_simples(ambiente: &Ambiente, sql: &str) -> i64 {
    sql_query(sql)
        .get_result::<Contagem>(&mut ambiente.conexao())
        .expect("contagem")
        .total
}

fn contar_uuid(ambiente: &Ambiente, sql: &str, id: Uuid) -> i64 {
    sql_query(sql)
        .bind::<SqlUuid, _>(id)
        .get_result::<Contagem>(&mut ambiente.conexao())
        .expect("contagem")
        .total
}

fn contar_uuid_texto(ambiente: &Ambiente, sql: &str, id: Uuid, texto: &str) -> i64 {
    sql_query(sql)
        .bind::<SqlUuid, _>(id)
        .bind::<Text, _>(texto)
        .get_result::<Contagem>(&mut ambiente.conexao())
        .expect("contagem")
        .total
}

fn texto_por_id(ambiente: &Ambiente, sql: &str, id: Uuid) -> String {
    sql_query(sql)
        .bind::<SqlUuid, _>(id)
        .get_result::<TextoRow>(&mut ambiente.conexao())
        .expect("texto")
        .valor
}

fn texto_opcional_por_id(ambiente: &Ambiente, sql: &str, id: Uuid) -> Option<String> {
    sql_query(sql)
        .bind::<SqlUuid, _>(id)
        .get_result::<OpcionalTextoRow>(&mut ambiente.conexao())
        .expect("texto opcional")
        .valor
}

// ===== seeds =====

fn semear_escritorio(ambiente: &Ambiente, cnpj: &str, nome: &str) -> Uuid {
    let id = Uuid::now_v7();
    sql_query("INSERT INTO escritorios (id, cnpj, nome) VALUES ($1, $2, $3)")
        .bind::<SqlUuid, _>(id)
        .bind::<Text, _>(cnpj)
        .bind::<Text, _>(nome)
        .execute(&mut ambiente.conexao())
        .expect("escritorio");
    id
}

fn semear_usuario(
    ambiente: &Ambiente,
    escritorio: Uuid,
    email: &str,
    cpf: &str,
    papel: &str,
) -> Uuid {
    let id = Uuid::now_v7();
    sql_query(
        "INSERT INTO usuarios (id, escritorio_id, email, cpf, senha_hash, papel, ativo) \
         VALUES ($1, $2, $3, $4, 'hash', $5, TRUE)",
    )
    .bind::<SqlUuid, _>(id)
    .bind::<SqlUuid, _>(escritorio)
    .bind::<Text, _>(email)
    .bind::<Text, _>(cpf)
    .bind::<Text, _>(papel)
    .execute(&mut ambiente.conexao())
    .expect("usuario");
    id
}

fn semear_cliente(ambiente: &Ambiente, escritorio: Uuid, documento: &str, nome: &str) -> Uuid {
    let id = Uuid::now_v7();
    sql_query(
        "INSERT INTO clientes (id, escritorio_id, tipo, cpf_cnpj, nome_razao_social) \
         VALUES ($1, $2, 'PF', $3, $4)",
    )
    .bind::<SqlUuid, _>(id)
    .bind::<SqlUuid, _>(escritorio)
    .bind::<Text, _>(documento)
    .bind::<Text, _>(nome)
    .execute(&mut ambiente.conexao())
    .expect("cliente");
    id
}

fn semear_plano_e_assinatura(ambiente: &Ambiente, escritorio: Uuid, limite: i64) {
    let plano = Uuid::now_v7();
    sql_query(
        "INSERT INTO planos_escritorio (id, nome, chamadas_mes, clientes_limite, usuarios_limite) \
         VALUES ($1, 'Plano de teste', $2, 1000, 100)",
    )
    .bind::<SqlUuid, _>(plano)
    .bind::<diesel::sql_types::Integer, _>(i32::try_from(limite).unwrap_or(i32::MAX))
    .execute(&mut ambiente.conexao())
    .expect("plano");

    sql_query(
        "INSERT INTO assinaturas_escritorio \
         (id, escritorio_id, plano_id, chamadas_mes_limite, clientes_limite, usuarios_limite) \
         VALUES ($1, $2, $3, $4, 1000, 100)",
    )
    .bind::<SqlUuid, _>(Uuid::now_v7())
    .bind::<SqlUuid, _>(escritorio)
    .bind::<SqlUuid, _>(plano)
    .bind::<diesel::sql_types::Integer, _>(i32::try_from(limite).unwrap_or(i32::MAX))
    .execute(&mut ambiente.conexao())
    .expect("assinatura");
}

fn semear_trabalho(
    ambiente: &Ambiente,
    escritorio: Uuid,
    tipo: &str,
    produto: &str,
    cron: &str,
) -> Uuid {
    let id = Uuid::now_v7();
    sql_query(
        "INSERT INTO trabalhos_agendados \
         (id, escritorio_id, nome, tipo, produto, ambiente, cron_expression, habilitado, proxima_execucao) \
         VALUES ($1, $2, 'Rotina de teste', $3, $4, 'TRIAL', $5, TRUE, NOW() - INTERVAL '1 minute')",
    )
    .bind::<SqlUuid, _>(id)
    .bind::<SqlUuid, _>(escritorio)
    .bind::<Text, _>(tipo)
    .bind::<Text, _>(produto)
    .bind::<Text, _>(cron)
    .execute(&mut ambiente.conexao())
    .expect("trabalho");
    id
}

fn semear_lote(ambiente: &Ambiente, escritorio: Uuid, servico: &str, documentos: &[&str]) -> Uuid {
    let id = Uuid::now_v7();
    sql_query(
        "INSERT INTO consultas_em_lote (id, escritorio_id, nome, produto, servico, verbo, status) \
         VALUES ($1, $2, 'Lote de teste', 'DIVIDA_ATIVA', $3, 'Consultar', 'pendente')",
    )
    .bind::<SqlUuid, _>(id)
    .bind::<SqlUuid, _>(escritorio)
    .bind::<Text, _>(servico)
    .execute(&mut ambiente.conexao())
    .expect("lote");

    for documento in documentos {
        sql_query(
            "INSERT INTO itens_consulta_lote (id, consulta_lote_id, documento, tipo, status) \
             VALUES ($1, $2, $3, 'PF', 'pendente')",
        )
        .bind::<SqlUuid, _>(Uuid::now_v7())
        .bind::<SqlUuid, _>(id)
        .bind::<Text, _>(*documento)
        .execute(&mut ambiente.conexao())
        .expect("item lote");
    }
    id
}

fn semear_execucao_presa(ambiente: &Ambiente, trabalho: Uuid, escritorio: Uuid) -> Uuid {
    let id = Uuid::now_v7();
    sql_query(
        "INSERT INTO execucoes_trabalho \
         (id, trabalho_agendado_id, escritorio_id, status, tentativas, lease_expira_em) \
         VALUES ($1, $2, $3, 'executando', 1, NOW() - INTERVAL '10 minutes')",
    )
    .bind::<SqlUuid, _>(id)
    .bind::<SqlUuid, _>(trabalho)
    .bind::<SqlUuid, _>(escritorio)
    .execute(&mut ambiente.conexao())
    .expect("execução presa");
    id
}

fn status_execucao(ambiente: &Ambiente, execucao: Uuid) -> String {
    texto_por_id(
        ambiente,
        "SELECT status AS valor FROM execucoes_trabalho WHERE id = $1",
        execucao,
    )
}

fn status_execucao_do_trabalho(ambiente: &Ambiente, trabalho: Uuid) -> String {
    texto_por_id(
        ambiente,
        "SELECT status AS valor FROM execucoes_trabalho WHERE trabalho_agendado_id = $1 ORDER BY criado_em DESC LIMIT 1",
        trabalho,
    )
}

fn execucoes_do_trabalho(ambiente: &Ambiente, trabalho: Uuid) -> i64 {
    contar_uuid(
        ambiente,
        "SELECT COUNT(*) AS total FROM execucoes_trabalho WHERE trabalho_agendado_id = $1",
        trabalho,
    )
}

fn status_lote(ambiente: &Ambiente, lote: Uuid) -> String {
    texto_por_id(
        ambiente,
        "SELECT status AS valor FROM consultas_em_lote WHERE id = $1",
        lote,
    )
}

fn status_item(ambiente: &Ambiente, lote: Uuid) -> String {
    texto_por_id(
        ambiente,
        "SELECT status AS valor FROM itens_consulta_lote WHERE consulta_lote_id = $1 ORDER BY criado_em LIMIT 1",
        lote,
    )
}

fn contagem_por_status_itens(ambiente: &Ambiente, lote: Uuid, status: &str) -> i64 {
    contar_uuid_texto(
        ambiente,
        "SELECT COUNT(*) AS total FROM itens_consulta_lote WHERE consulta_lote_id = $1 AND status = $2",
        lote,
        status,
    )
}

fn tentativas_do_item(ambiente: &Ambiente, lote: Uuid) -> i64 {
    contar_uuid(
        ambiente,
        "SELECT tentativas AS total FROM itens_consulta_lote WHERE consulta_lote_id = $1",
        lote,
    )
}

// ===== testes =====

#[tokio::test]
async fn executa_trabalho_devido_pela_rotina_e_mede_chamadas() {
    let ambiente = Ambiente::novo(&[]);
    let escritorio = semear_escritorio(&ambiente, CNPJ_A, "Escritório A");
    semear_usuario(
        &ambiente,
        escritorio,
        "a@jobs.test",
        CPF_SUCESSO_A,
        "CLIENTE",
    );
    semear_cliente(&ambiente, escritorio, CPF_SUCESSO_A, "Cliente Um");
    semear_cliente(&ambiente, escritorio, CPF_SUCESSO_B, "Cliente Dois");
    semear_plano_e_assinatura(&ambiente, escritorio, 1000);
    let trabalho = semear_trabalho(
        &ambiente,
        escritorio,
        "consulta_divida_ativa",
        "DIVIDA_ATIVA",
        "0 0 * * *",
    );

    ambiente.runner().rodada().await;

    assert_eq!(execucoes_do_trabalho(&ambiente, trabalho), 1);
    assert_eq!(
        status_execucao_do_trabalho(&ambiente, trabalho),
        "concluido"
    );

    let processados = contar_uuid(
        &ambiente,
        "SELECT registros_processados AS total FROM execucoes_trabalho WHERE trabalho_agendado_id = $1",
        trabalho,
    );
    assert_eq!(processados, 2, "as duas consultas devem ser processadas");

    assert_eq!(
        contar_simples(&ambiente, "SELECT COUNT(*) AS total FROM chamadas_serpro"),
        2
    );
    assert_eq!(
        contar_simples(
            &ambiente,
            "SELECT COUNT(*) AS total FROM chamadas_serpro WHERE origem = 'rotina' AND resultado = 'sucesso'"
        ),
        2
    );

    // A próxima execução foi agendada (a rotina não fica "vencida").
    let proxima = texto_opcional_por_id(
        &ambiente,
        "SELECT proxima_execucao::text AS valor FROM trabalhos_agendados WHERE id = $1",
        trabalho,
    );
    assert!(proxima.is_some(), "próxima execução deve ser reagendada");
}

#[tokio::test]
async fn limite_do_plano_bloqueia_trabalho_sem_chamar_serpro() {
    let ambiente = Ambiente::novo(&[]);
    let escritorio = semear_escritorio(&ambiente, CNPJ_A, "Escritório A");
    semear_usuario(
        &ambiente,
        escritorio,
        "a@jobs.test",
        CPF_SUCESSO_A,
        "CLIENTE",
    );
    semear_cliente(&ambiente, escritorio, CPF_SUCESSO_A, "Cliente Um");
    semear_plano_e_assinatura(&ambiente, escritorio, 0);
    let trabalho = semear_trabalho(
        &ambiente,
        escritorio,
        "consulta_divida_ativa",
        "DIVIDA_ATIVA",
        "0 0 * * *",
    );

    ambiente.runner().rodada().await;

    assert_eq!(execucoes_do_trabalho(&ambiente, trabalho), 1);
    assert_eq!(
        status_execucao_do_trabalho(&ambiente, trabalho),
        "bloqueado"
    );

    assert_eq!(
        contar_simples(&ambiente, "SELECT COUNT(*) AS total FROM chamadas_serpro"),
        0,
        "bloqueio local não chama a SERPRO"
    );
    assert_eq!(
        contar_uuid(
            &ambiente,
            "SELECT COUNT(*) AS total FROM bloqueios_serpro WHERE escritorio_id = $1 AND motivo = 'limite_plano'",
            escritorio,
        ),
        1
    );
}

#[tokio::test]
async fn lote_com_falha_transitoria_usa_backoff_e_reexecuta() {
    let ambiente = Ambiente::novo(&[
        ("JOBS_MAX_TENTATIVAS", "3"),
        ("JOBS_BACKOFF_BASE_SECS", "30"),
        ("JOBS_BACKOFF_TETO_SECS", "120"),
    ]);
    let escritorio = semear_escritorio(&ambiente, CNPJ_A, "Escritório A");
    semear_usuario(
        &ambiente,
        escritorio,
        "a@jobs.test",
        CPF_SUCESSO_A,
        "CLIENTE",
    );
    semear_plano_e_assinatura(&ambiente, escritorio, 1000);
    let lote = semear_lote(&ambiente, escritorio, "DEVEDOR", &[CPF_INDETERMINADO]);

    let runner = ambiente.runner();
    runner.rodada().await;

    assert_eq!(status_item(&ambiente, lote), "pendente");
    assert_eq!(
        status_lote(&ambiente, lote),
        "pendente",
        "lote volta à fila"
    );
    let tentativas = tentativas_do_item(&ambiente, lote);
    assert_eq!(tentativas, 1);

    // Backoff ainda não venceu: nova rodada não reivindica o item.
    runner.rodada().await;
    let tentativas = tentativas_do_item(&ambiente, lote);
    assert_eq!(tentativas, 1, "item em backoff não pode ser reivindicado");

    // Vence o backoff: reivindica de novo (tentativas = 2).
    sql_query(
        "UPDATE itens_consulta_lote SET proxima_tentativa_em = NOW() - INTERVAL '1 second' \
         WHERE consulta_lote_id = $1",
    )
    .bind::<SqlUuid, _>(lote)
    .execute(&mut ambiente.conexao())
    .expect("vence backoff");
    runner.rodada().await;

    let tentativas = tentativas_do_item(&ambiente, lote);
    assert_eq!(tentativas, 2);

    // Esgota as tentativas até `max_tentativas`: item e lote falham.
    for _ in 0..2 {
        sql_query(
            "UPDATE itens_consulta_lote SET proxima_tentativa_em = NOW() - INTERVAL '1 second' \
             WHERE consulta_lote_id = $1 AND status = 'pendente'",
        )
        .bind::<SqlUuid, _>(lote)
        .execute(&mut ambiente.conexao())
        .expect("vence backoff");
        runner.rodada().await;
    }
    assert_eq!(status_item(&ambiente, lote), "falhou");
    assert_eq!(status_lote(&ambiente, lote), "falhou");
}

#[tokio::test]
async fn recupera_execucao_presa_por_lease_expirado() {
    let ambiente = Ambiente::novo(&[]);
    let escritorio = semear_escritorio(&ambiente, CNPJ_A, "Escritório A");
    semear_usuario(
        &ambiente,
        escritorio,
        "a@jobs.test",
        CPF_SUCESSO_A,
        "CLIENTE",
    );
    semear_cliente(&ambiente, escritorio, CPF_SUCESSO_A, "Cliente Um");
    semear_plano_e_assinatura(&ambiente, escritorio, 1000);
    let trabalho = semear_trabalho(
        &ambiente,
        escritorio,
        "consulta_divida_ativa",
        "DIVIDA_ATIVA",
        "0 0 * * *",
    );
    // O trabalho não deve ser reivindicado como "devido" — a execução presa
    // é a que será recuperada e retomada.
    sql_query(
        "UPDATE trabalhos_agendados SET proxima_execucao = NOW() + INTERVAL '1 day' WHERE id = $1",
    )
    .bind::<SqlUuid, _>(trabalho)
    .execute(&mut ambiente.conexao())
    .expect("adia");
    let execucao = semear_execucao_presa(&ambiente, trabalho, escritorio);

    ambiente.runner().rodada().await;

    assert_eq!(status_execucao(&ambiente, execucao), "concluido");
    let tentativas = contar_uuid(
        &ambiente,
        "SELECT tentativas AS total FROM execucoes_trabalho WHERE id = $1",
        execucao,
    );
    assert_eq!(
        tentativas, 2,
        "reivindicação de retry incrementa tentativas"
    );
    assert_eq!(
        contar_simples(&ambiente, "SELECT COUNT(*) AS total FROM chamadas_serpro"),
        1
    );
}

#[tokio::test]
async fn lock_por_escritorio_e_isolamento_multi_tenant() {
    let ambiente = Ambiente::novo(&[]);
    let escritorio_a = semear_escritorio(&ambiente, CNPJ_A, "Escritório A");
    let escritorio_b = semear_escritorio(&ambiente, CNPJ_B, "Escritório B");
    semear_usuario(
        &ambiente,
        escritorio_a,
        "a@jobs.test",
        CPF_SUCESSO_A,
        "CLIENTE",
    );
    semear_usuario(
        &ambiente,
        escritorio_b,
        "b@jobs.test",
        CPF_SUCESSO_B,
        "CLIENTE",
    );
    semear_cliente(&ambiente, escritorio_a, CPF_SUCESSO_A, "Cliente A");
    semear_cliente(&ambiente, escritorio_b, CPF_SUCESSO_B, "Cliente B");
    semear_plano_e_assinatura(&ambiente, escritorio_a, 1000);
    semear_plano_e_assinatura(&ambiente, escritorio_b, 1000);
    let _trabalho_a1 = semear_trabalho(
        &ambiente,
        escritorio_a,
        "consulta_divida_ativa",
        "DIVIDA_ATIVA",
        "0 0 * * *",
    );
    let _trabalho_a2 = semear_trabalho(
        &ambiente,
        escritorio_a,
        "consulta_divida_ativa",
        "DIVIDA_ATIVA",
        "0 12 * * *",
    );
    let _trabalho_b = semear_trabalho(
        &ambiente,
        escritorio_b,
        "consulta_divida_ativa",
        "DIVIDA_ATIVA",
        "0 0 * * *",
    );

    ambiente.runner().rodada().await;

    // Um trabalho por escritório por rodada (lock por escritório), mesmo com
    // dois trabalhos vencidos no escritório A.
    assert_eq!(
        contar_uuid(
            &ambiente,
            "SELECT COUNT(*) AS total FROM execucoes_trabalho WHERE escritorio_id = $1",
            escritorio_a,
        ),
        1
    );
    assert_eq!(
        contar_uuid(
            &ambiente,
            "SELECT COUNT(*) AS total FROM execucoes_trabalho WHERE escritorio_id = $1",
            escritorio_b,
        ),
        1
    );
    // Cada chamada fica no escritório dono da rotina.
    assert_eq!(
        contar_uuid(
            &ambiente,
            "SELECT COUNT(*) AS total FROM chamadas_serpro WHERE escritorio_id = $1",
            escritorio_a,
        ),
        1
    );
    assert_eq!(
        contar_uuid(
            &ambiente,
            "SELECT COUNT(*) AS total FROM chamadas_serpro WHERE escritorio_id = $1",
            escritorio_b,
        ),
        1
    );
}

#[tokio::test]
async fn dois_workers_nao_processam_o_mesmo_lote_duas_vezes() {
    let ambiente = Ambiente::novo(&[]);
    let escritorio = semear_escritorio(&ambiente, CNPJ_A, "Escritório A");
    semear_usuario(
        &ambiente,
        escritorio,
        "a@jobs.test",
        CPF_SUCESSO_A,
        "CLIENTE",
    );
    semear_plano_e_assinatura(&ambiente, escritorio, 1000);
    let lote = semear_lote(
        &ambiente,
        escritorio,
        "DEVEDOR",
        &[CPF_SUCESSO_A, CPF_SUCESSO_B],
    );

    let worker_a = ambiente.runner();
    let worker_b = ambiente.runner();
    tokio::join!(worker_a.rodada(), worker_b.rodada());

    assert_eq!(status_lote(&ambiente, lote), "concluido");
    assert_eq!(contagem_por_status_itens(&ambiente, lote, "concluido"), 2);
    assert_eq!(
        contar_simples(&ambiente, "SELECT COUNT(*) AS total FROM chamadas_serpro"),
        2,
        "cada item deve ser consultado exatamente uma vez"
    );
}

#[tokio::test]
async fn lote_bloqueado_por_limite_marca_itens_como_bloqueado() {
    let ambiente = Ambiente::novo(&[]);
    let escritorio = semear_escritorio(&ambiente, CNPJ_A, "Escritório A");
    semear_usuario(
        &ambiente,
        escritorio,
        "a@jobs.test",
        CPF_SUCESSO_A,
        "CLIENTE",
    );
    semear_plano_e_assinatura(&ambiente, escritorio, 0);
    let lote = semear_lote(
        &ambiente,
        escritorio,
        "DEVEDOR",
        &[CPF_SUCESSO_A, CPF_SUCESSO_B],
    );

    ambiente.runner().rodada().await;

    assert_eq!(contagem_por_status_itens(&ambiente, lote, "bloqueado"), 2);
    assert_eq!(status_lote(&ambiente, lote), "bloqueado");
    assert_eq!(
        contar_simples(&ambiente, "SELECT COUNT(*) AS total FROM chamadas_serpro"),
        0
    );
    assert_eq!(
        contar_uuid(
            &ambiente,
            "SELECT COUNT(*) AS total FROM bloqueios_serpro WHERE escritorio_id = $1 AND motivo = 'limite_plano'",
            escritorio,
        ),
        2
    );

    // Notificação de conclusão (mesmo bloqueado) para o usuário do escritório.
    assert_eq!(
        contar_simples(
            &ambiente,
            "SELECT COUNT(*) AS total FROM notificacoes WHERE tipo = 'lote_concluido'"
        ),
        1
    );
}

#[tokio::test]
async fn trabalho_sem_sucesso_notifica_falha_da_rotina() {
    let ambiente = Ambiente::novo(&[]);
    let escritorio = semear_escritorio(&ambiente, CNPJ_A, "Escritório A");
    semear_usuario(
        &ambiente,
        escritorio,
        "a@jobs.test",
        CPF_SUCESSO_A,
        "CLIENTE",
    );
    semear_cliente(
        &ambiente,
        escritorio,
        CPF_NAO_ENCONTRADO,
        "Cliente Fantasma",
    );
    semear_plano_e_assinatura(&ambiente, escritorio, 1000);
    let trabalho = semear_trabalho(
        &ambiente,
        escritorio,
        "consulta_divida_ativa",
        "DIVIDA_ATIVA",
        "0 0 * * *",
    );

    ambiente.runner().rodada().await;

    let status = status_execucao_do_trabalho(&ambiente, trabalho);
    assert_eq!(status, "falhou");
    assert_eq!(
        contar_simples(
            &ambiente,
            "SELECT COUNT(*) AS total FROM notificacoes WHERE tipo = 'falha_rotina'"
        ),
        1
    );
}
