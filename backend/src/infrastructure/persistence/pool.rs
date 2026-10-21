use std::time::Duration;

use diesel::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use thiserror::Error;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;
pub type DbConnection = PooledConnection<ConnectionManager<PgConnection>>;

pub const TAMANHO_MAXIMO_POOL: u32 = 10;
pub const TIMEOUT_CONEXAO: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("não foi possível criar o pool de conexões")]
pub struct PoolIndisponivel;

pub fn criar_pool(database_url: &str) -> Result<DbPool, PoolIndisponivel> {
    let gerenciador = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder()
        .max_size(TAMANHO_MAXIMO_POOL)
        .connection_timeout(TIMEOUT_CONEXAO)
        .build(gerenciador)
        .map_err(|_| PoolIndisponivel)
}
