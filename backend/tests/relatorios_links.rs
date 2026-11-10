mod common;

use std::sync::Arc;

use chrono::{Duration, Utc};
use diesel::RunQueryDsl;
use uuid::Uuid;

use accurit_backend::application::relatorios::use_cases::{
    AcessarRelatorioPorLinkUseCase, CriarLinkCompartilhamentoUseCase, RevogarLinksRelatorioUseCase,
    ValidarLinkCompartilhamentoUseCase,
};
use accurit_backend::domain::relatorios::entidades::NovoRelatorio;
use accurit_backend::domain::relatorios::enums::{FormatoRelatorio, TipoRelatorio};
use accurit_backend::domain::relatorios::erros::ErroRelatorio;
use accurit_backend::domain::relatorios::ports::{
    RepositorioLinksCompartilhamento, RepositorioRelatorios,
};
use accurit_backend::infrastructure::persistence::migrations::{
    MIGRATIONS, aplicar_migrations_com_lock,
};
use accurit_backend::infrastructure::persistence::pool::criar_pool;
use accurit_backend::infrastructure::persistence::relatorios::RelatoriosDiesel;

use common::BancoTemporario;

struct Ambiente {
    banco: BancoTemporario,
    repo: Arc<RelatoriosDiesel>,
    escritorio_a: Uuid,
    usuario_a: Uuid,
    cliente_a: Uuid,
    escritorio_b: Uuid,
}

fn ambiente() -> Ambiente {
    let banco = BancoTemporario::novo();
    aplicar_migrations_com_lock(banco.conexao(), MIGRATIONS).expect("migrations");
    let pool = criar_pool(&banco.database_url).expect("pool");
    let repo = Arc::new(RelatoriosDiesel::novo(pool));

    let escritorio_a = Uuid::now_v7();
    let usuario_a = Uuid::now_v7();
    let cliente_a = Uuid::now_v7();
    let escritorio_b = Uuid::now_v7();
    let usuario_b = Uuid::now_v7();
    let mut conexao = banco.conexao();
    diesel::sql_query("INSERT INTO escritorios (id, nome, cnpj) VALUES ($1, $2, $3)")
        .bind::<diesel::sql_types::Uuid, _>(escritorio_a)
        .bind::<diesel::sql_types::Text, _>("Escritório A")
        .bind::<diesel::sql_types::Text, _>("11222333000181")
        .execute(&mut conexao)
        .expect("escritorio A");
    diesel::sql_query("INSERT INTO escritorios (id, nome, cnpj) VALUES ($1, $2, $3)")
        .bind::<diesel::sql_types::Uuid, _>(escritorio_b)
        .bind::<diesel::sql_types::Text, _>("Escritório B")
        .bind::<diesel::sql_types::Text, _>("11444777000161")
        .execute(&mut conexao)
        .expect("escritorio B");

    for (id, escritorio, email, cpf) in [
        (usuario_a, escritorio_a, "a@teste.test", "52998224725"),
        (usuario_b, escritorio_b, "b@teste.test", "11144477735"),
    ] {
        diesel::sql_query(
            "INSERT INTO usuarios (id, escritorio_id, email, cpf, senha_hash, papel) \
             VALUES ($1, $2, $3, $4, 'hash', 'OPERADOR')",
        )
        .bind::<diesel::sql_types::Uuid, _>(id)
        .bind::<diesel::sql_types::Uuid, _>(escritorio)
        .bind::<diesel::sql_types::Text, _>(email)
        .bind::<diesel::sql_types::Text, _>(cpf)
        .execute(&mut conexao)
        .expect("usuario");
    }

    diesel::sql_query(
        "INSERT INTO clientes (id, escritorio_id, tipo, cpf_cnpj, nome_razao_social) \
         VALUES ($1, $2, 'PF', '52998224725', 'Cliente A')",
    )
    .bind::<diesel::sql_types::Uuid, _>(cliente_a)
    .bind::<diesel::sql_types::Uuid, _>(escritorio_a)
    .execute(&mut conexao)
    .expect("cliente A");

    Ambiente {
        banco,
        repo,
        escritorio_a,
        usuario_a,
        cliente_a,
        escritorio_b,
    }
}

async fn relatorio_concluido(ambiente: &Ambiente, cliente_id: Option<Uuid>) -> Uuid {
    let relatorio = RepositorioRelatorios::criar(
        ambiente.repo.as_ref(),
        &NovoRelatorio {
            escritorio_id: ambiente.escritorio_a,
            usuario_id: ambiente.usuario_a,
            cliente_id,
            tipo_relatorio: TipoRelatorio::SituacaoFiscal,
            formato: FormatoRelatorio::Pdf,
            competencia: None,
        },
    )
    .await
    .expect("relatório criado");
    ambiente
        .repo
        .atualizar_status_e_conteudo(
            relatorio.id,
            accurit_backend::domain::relatorios::enums::StatusRelatorio::Concluido,
            Some(b"conteudo-do-relatorio".to_vec()),
            Some("relatorio.pdf".to_owned()),
        )
        .await
        .expect("relatório concluído");
    relatorio.id
}

fn caso_criar(ambiente: &Ambiente) -> CriarLinkCompartilhamentoUseCase {
    CriarLinkCompartilhamentoUseCase::novo(
        ambiente.repo.clone() as Arc<dyn RepositorioLinksCompartilhamento>,
        ambiente.repo.clone() as Arc<dyn RepositorioRelatorios>,
    )
}

#[tokio::test]
async fn link_expira_revoga_e_respeita_limite_de_acessos() {
    let ambiente = ambiente();
    let relatorio_id = relatorio_concluido(&ambiente, Some(ambiente.cliente_a)).await;

    let criar = caso_criar(&ambiente);
    let (link, token) = criar
        .executar(
            accurit_backend::application::relatorios::use_cases::ComandoCriarLink {
                relatorio_id,
                escritorio_id: ambiente.escritorio_a,
                expira_em: Utc::now() + Duration::hours(1),
                max_acessos: 1,
            },
        )
        .await
        .expect("link criado");

    assert!(
        !link.token_hash.is_empty(),
        "token só é persistido como hash"
    );
    assert_ne!(link.token_hash, token);

    let acessar = AcessarRelatorioPorLinkUseCase::novo(
        ambiente.repo.clone() as Arc<dyn RepositorioLinksCompartilhamento>,
        ambiente.repo.clone() as Arc<dyn RepositorioRelatorios>,
    );
    let primeira = acessar
        .executar(&token, Utc::now())
        .await
        .expect("primeiro acesso");
    assert_eq!(primeira.id, relatorio_id);
    assert_eq!(primeira.conteudo, b"conteudo-do-relatorio");

    let segunda = acessar.executar(&token, Utc::now()).await;
    assert_eq!(segunda.unwrap_err(), ErroRelatorio::LinkLimiteAtingido);

    // Revogação explícita bloqueia novos acessos mesmo dentro da validade.
    let (_, token_revogado) = criar
        .executar(
            accurit_backend::application::relatorios::use_cases::ComandoCriarLink {
                relatorio_id,
                escritorio_id: ambiente.escritorio_a,
                expira_em: Utc::now() + Duration::hours(1),
                max_acessos: 10,
            },
        )
        .await
        .expect("segundo link criado");

    let revogar = RevogarLinksRelatorioUseCase::novo(
        ambiente.repo.clone() as Arc<dyn RepositorioLinksCompartilhamento>,
        ambiente.repo.clone() as Arc<dyn RepositorioRelatorios>,
    );
    let revogados = revogar
        .executar(relatorio_id, ambiente.escritorio_a)
        .await
        .expect("revogação");
    assert!(revogados >= 1);

    let validar = ValidarLinkCompartilhamentoUseCase::novo(
        ambiente.repo.clone() as Arc<dyn RepositorioLinksCompartilhamento>
    );
    assert_eq!(
        validar
            .executar(&token_revogado, Utc::now())
            .await
            .unwrap_err(),
        ErroRelatorio::LinkRevogado
    );
}

#[tokio::test]
async fn link_expirado_e_idor_entre_escritorios() {
    let ambiente = ambiente();
    let relatorio_id = relatorio_concluido(&ambiente, None).await;
    let criar = caso_criar(&ambiente);

    let (_, token) = criar
        .executar(
            accurit_backend::application::relatorios::use_cases::ComandoCriarLink {
                relatorio_id,
                escritorio_id: ambiente.escritorio_a,
                expira_em: Utc::now() - Duration::minutes(1),
                max_acessos: 10,
            },
        )
        .await
        .expect("link expirado pode ser criado");

    let validar = ValidarLinkCompartilhamentoUseCase::novo(
        ambiente.repo.clone() as Arc<dyn RepositorioLinksCompartilhamento>
    );
    assert_eq!(
        validar.executar(&token, Utc::now()).await.unwrap_err(),
        ErroRelatorio::LinkExpirado
    );

    // Escritório B não pode criar link para relatório de A.
    let idor = criar
        .executar(
            accurit_backend::application::relatorios::use_cases::ComandoCriarLink {
                relatorio_id,
                escritorio_id: ambiente.escritorio_b,
                expira_em: Utc::now() + Duration::hours(1),
                max_acessos: 10,
            },
        )
        .await;
    assert_eq!(idor.unwrap_err(), ErroRelatorio::RelatorioNaoEncontrado);

    // Revogação de relatório de A por B também falha.
    let revogar = RevogarLinksRelatorioUseCase::novo(
        ambiente.repo.clone() as Arc<dyn RepositorioLinksCompartilhamento>,
        ambiente.repo.clone() as Arc<dyn RepositorioRelatorios>,
    );
    assert_eq!(
        revogar
            .executar(relatorio_id, ambiente.escritorio_b)
            .await
            .unwrap_err(),
        ErroRelatorio::RelatorioNaoEncontrado
    );
    let _ = &ambiente.banco;
}
