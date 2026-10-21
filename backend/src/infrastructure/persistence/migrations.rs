use std::time::Duration;

use diesel::migration::MigrationSource;
use diesel::pg::{Pg, PgConnection};
use diesel::sql_types::BigInt;
use diesel::{Connection, RunQueryDsl};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use thiserror::Error;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
pub const ADVISORY_LOCK_ID: i64 = 727_001;

const TENTATIVAS_CONEXAO: u32 = 15;
const ESPERA_INICIAL: Duration = Duration::from_secs(1);
const ESPERA_MAXIMA: Duration = Duration::from_secs(5);

#[derive(Debug, Error)]
pub enum MigrationError {
    #[error("não foi possível conectar ao PostgreSQL após {tentativas} tentativa(s)")]
    Conexao { tentativas: u32 },
    #[error("não foi possível adquirir o lock de migração")]
    Lock,
    #[error("falha ao aplicar migrações: {0}")]
    Aplicacao(String),
    #[error("falha ao reverter migrações: {0}")]
    Reversao(String),
}

pub fn conectar_com_retry(database_url: &str) -> Result<PgConnection, MigrationError> {
    conectar_com_retry_configurado(database_url, TENTATIVAS_CONEXAO, ESPERA_INICIAL)
}

pub fn conectar_com_retry_configurado(
    database_url: &str,
    tentativas: u32,
    espera_inicial: Duration,
) -> Result<PgConnection, MigrationError> {
    let tentativas = tentativas.max(1);
    let mut espera = espera_inicial.min(ESPERA_MAXIMA);

    for tentativa in 1..=tentativas {
        match PgConnection::establish(database_url) {
            Ok(conexao) => return Ok(conexao),
            Err(_) => {
                if tentativa == tentativas {
                    break;
                }
                tracing::warn!(
                    tentativa,
                    total = tentativas,
                    "falha ao conectar ao PostgreSQL; tentando novamente"
                );
                std::thread::sleep(espera);
                espera = (espera * 2).min(ESPERA_MAXIMA);
            }
        }
    }

    Err(MigrationError::Conexao { tentativas })
}

pub struct AdvisoryLockGuard {
    conexao: PgConnection,
}

impl AdvisoryLockGuard {
    pub fn adquirir(mut conexao: PgConnection) -> Result<Self, MigrationError> {
        diesel::sql_query("SELECT pg_advisory_lock($1)")
            .bind::<BigInt, _>(ADVISORY_LOCK_ID)
            .execute(&mut conexao)
            .map_err(|_| MigrationError::Lock)?;

        Ok(Self { conexao })
    }

    pub fn conexao(&mut self) -> &mut PgConnection {
        &mut self.conexao
    }
}

impl Drop for AdvisoryLockGuard {
    fn drop(&mut self) {
        let _ = diesel::sql_query("SELECT pg_advisory_unlock($1)")
            .bind::<BigInt, _>(ADVISORY_LOCK_ID)
            .execute(&mut self.conexao);
    }
}

pub fn aplicar_migrations_com_lock<M>(
    conexao: PgConnection,
    fonte: M,
) -> Result<usize, MigrationError>
where
    M: MigrationSource<Pg>,
{
    let mut guarda = AdvisoryLockGuard::adquirir(conexao)?;
    guarda
        .conexao()
        .run_pending_migrations(fonte)
        .map(|aplicadas| aplicadas.len())
        .map_err(|erro| MigrationError::Aplicacao(erro.to_string()))
}

pub fn aplicar_migrations(conexao: &mut PgConnection) -> Result<usize, MigrationError> {
    conexao
        .run_pending_migrations(MIGRATIONS)
        .map(|aplicadas| aplicadas.len())
        .map_err(|erro| MigrationError::Aplicacao(erro.to_string()))
}

pub fn reverter_todas(conexao: &mut PgConnection) -> Result<usize, MigrationError> {
    conexao
        .revert_all_migrations(MIGRATIONS)
        .map(|revertidas| revertidas.len())
        .map_err(|erro| MigrationError::Reversao(erro.to_string()))
}

pub fn migrations_pendentes(conexao: &mut PgConnection) -> Result<usize, MigrationError> {
    conexao
        .pending_migrations(MIGRATIONS)
        .map(|pendentes| pendentes.len())
        .map_err(|erro| MigrationError::Aplicacao(erro.to_string()))
}
