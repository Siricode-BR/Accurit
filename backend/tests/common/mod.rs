#![allow(dead_code)]

use std::collections::HashMap;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use diesel::connection::SimpleConnection;
use diesel::pg::PgConnection;
use diesel::sql_query;
use diesel::{Connection, RunQueryDsl};
use url::Url;
use uuid::Uuid;

use accurit_backend::infrastructure::config::Config;
use accurit_backend::infrastructure::persistence::pool::DbPool;

pub struct BancoTemporario {
    admin_url: String,
    pub database_url: String,
    nome: String,
}

impl BancoTemporario {
    pub fn novo() -> Self {
        let admin_url = std::env::var("TEST_DATABASE_ADMIN_URL")
            .expect("TEST_DATABASE_ADMIN_URL deve estar definido para os testes de integração");
        let nome = format!("accurit_test_{}", Uuid::now_v7().simple());
        let mut administrador = PgConnection::establish(&admin_url)
            .expect("deve conectar ao PostgreSQL administrativo");

        sql_query(format!("CREATE DATABASE \"{nome}\""))
            .execute(&mut administrador)
            .expect("deve criar o banco temporário");

        let mut url = Url::parse(&admin_url).expect("URL administrativa válida");
        url.set_path(&nome);
        let database_url = url.to_string();

        Self {
            admin_url,
            database_url,
            nome,
        }
    }

    pub fn conexao(&self) -> PgConnection {
        PgConnection::establish(&self.database_url).expect("deve conectar ao banco temporário")
    }
}

impl Drop for BancoTemporario {
    fn drop(&mut self) {
        if let Ok(mut administrador) = PgConnection::establish(&self.admin_url) {
            let _ = administrador.batch_execute(&format!(
                "DROP DATABASE IF EXISTS \"{}\" WITH (FORCE);",
                self.nome
            ));
        }
    }
}

pub fn configuracao(cors_allowed_origins: &str) -> Config {
    configuracao_com_extras(cors_allowed_origins, &[])
}

pub fn configuracao_com_extras(cors_allowed_origins: &str, extras: &[(&str, &str)]) -> Config {
    let mut pares: HashMap<String, String> = HashMap::from([
        ("APP_ENV".into(), "development".into()),
        (
            "DATABASE_URL".into(),
            "postgres://app:senha@localhost:5432/accurit".into(),
        ),
        ("JWT_SECRET".into(), "x".repeat(32)),
        ("CERTIFICATE_MASTER_KEY".into(), STANDARD.encode([7_u8; 32])),
        ("CORS_ALLOWED_ORIGINS".into(), cors_allowed_origins.into()),
        (
            "SERPRO_TOKEN_URL".into(),
            "https://exemplo.test/token".into(),
        ),
        (
            "SERPRO_SAPI_AUTH_URL".into(),
            "https://exemplo.test/auth".into(),
        ),
        (
            "SERPRO_INTEGRA_TRIAL_URL".into(),
            "https://exemplo.test/integra".into(),
        ),
        (
            "SERPRO_INTEGRA_PROD_URL".into(),
            "https://exemplo.test/integra-prod".into(),
        ),
        (
            "SERPRO_DIVIDA_ATIVA_TRIAL_URL".into(),
            "https://exemplo.test/divida".into(),
        ),
    ]);

    for (chave, valor) in extras {
        pares.insert((*chave).to_owned(), (*valor).to_owned());
    }

    Config::from_pairs(pares).expect("configuração de teste válida")
}

pub fn pool_do_banco(banco: &BancoTemporario) -> DbPool {
    use accurit_backend::infrastructure::persistence::pool::criar_pool;
    criar_pool(&banco.database_url).expect("pool deve ser criado")
}