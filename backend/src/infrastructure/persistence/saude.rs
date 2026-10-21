use std::future::Future;
use std::time::Duration;

use diesel::RunQueryDsl;

use crate::application::ports::{SaudeBanco, SaudeBancoError};

use super::pool::DbPool;

const TIMEOUT_SAUDE: Duration = Duration::from_secs(2);

#[derive(Clone)]
pub struct PostgresSaudeBanco {
    pool: DbPool,
}

impl PostgresSaudeBanco {
    pub const fn novo(pool: DbPool) -> Self {
        Self { pool }
    }
}

impl SaudeBanco for PostgresSaudeBanco {
    fn checar(&self) -> impl Future<Output = Result<(), SaudeBancoError>> + Send {
        let pool = self.pool.clone();
        async move {
            let tarefa = tokio::task::spawn_blocking(move || {
                let mut conexao = pool.get().map_err(|_| SaudeBancoError)?;
                diesel::sql_query("SELECT 1")
                    .execute(&mut conexao)
                    .map_err(|_| SaudeBancoError)?;
                Ok::<(), SaudeBancoError>(())
            });

            match tokio::time::timeout(TIMEOUT_SAUDE, tarefa).await {
                Ok(Ok(Ok(()))) => Ok(()),
                _ => Err(SaudeBancoError),
            }
        }
    }
}
