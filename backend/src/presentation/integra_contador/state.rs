use std::sync::Arc;

use crate::application::integra_contador::{AutenticarProcuradorUseCase, ExecutarIntegraContadorUseCase};

#[derive(Clone)]
pub struct EstadoIntegraContador {
    pub executar: Arc<ExecutarIntegraContadorUseCase>,
    pub autenticar_procurador: Arc<AutenticarProcuradorUseCase>,
}

impl EstadoIntegraContador {
    pub fn novo(
        executar: Arc<ExecutarIntegraContadorUseCase>,
        autenticar_procurador: Arc<AutenticarProcuradorUseCase>,
    ) -> Self {
        Self {
            executar,
            autenticar_procurador,
        }
    }
}
