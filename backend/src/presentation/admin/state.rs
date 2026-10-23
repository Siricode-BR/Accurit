use std::sync::Arc;

use crate::application::admin::{ListarEscritoriosUseCase, ProvisionarTenantUseCase};
use crate::presentation::serpro::state::SerproState;

#[derive(Clone)]
pub struct AdminState {
    pub provisionar_tenant: Arc<ProvisionarTenantUseCase>,
    pub listar_escritorios: Arc<ListarEscritoriosUseCase>,
    pub serpro: SerproState,
}

impl AdminState {
    pub fn novo(
        provisionar_tenant: Arc<ProvisionarTenantUseCase>,
        listar_escritorios: Arc<ListarEscritoriosUseCase>,
        serpro: SerproState,
    ) -> Self {
        Self {
            provisionar_tenant,
            listar_escritorios,
            serpro,
        }
    }
}
