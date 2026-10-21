use std::sync::Arc;

use crate::application::ports::SaudeBanco;
use crate::application::saude::ConsultarSaude;

pub struct AppState<S> {
    pub consultar_saude: Arc<ConsultarSaude<S>>,
}

impl<S: SaudeBanco> AppState<S> {
    pub fn novo(consultar_saude: ConsultarSaude<S>) -> Self {
        Self {
            consultar_saude: Arc::new(consultar_saude),
        }
    }
}

impl<S> Clone for AppState<S> {
    fn clone(&self) -> Self {
        Self {
            consultar_saude: Arc::clone(&self.consultar_saude),
        }
    }
}
