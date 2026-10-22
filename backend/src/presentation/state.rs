use std::sync::Arc;

use crate::application::ports::SaudeBanco;
use crate::application::saude::ConsultarSaude;
use crate::infrastructure::persistence::pool::DbPool;

pub struct AppState<S> {
    pub consultar_saude: Arc<ConsultarSaude<S>>,
    pub pool: DbPool,
}

impl<S: SaudeBanco> AppState<S> {
    pub fn novo(consultar_saude: ConsultarSaude<S>, pool: DbPool) -> Self {
        Self {
            consultar_saude: Arc::new(consultar_saude),
            pool,
        }
    }
}

impl<S> Clone for AppState<S> {
    fn clone(&self) -> Self {
        Self {
            consultar_saude: Arc::clone(&self.consultar_saude),
            pool: self.pool.clone(),
        }
    }
}