use std::sync::Arc;

use crate::application::integra_contador::ExecutarIntegraContadorUseCase;

#[derive(Clone)]
pub struct EstadoIntegraContador {
    pub executar: Arc<ExecutarIntegraContadorUseCase>,
}

impl EstadoIntegraContador {
    pub fn novo(executar: Arc<ExecutarIntegraContadorUseCase>) -> Self {
        Self { executar }
    }
}
