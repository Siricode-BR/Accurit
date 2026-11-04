use std::sync::Arc;

use crate::application::integra_contador::use_cases::ExecutarIntegraContadorUseCase;
use crate::presentation::serpro::ambiente::AmbienteSerpro;

/// Estado compartilhado das rotas Integra Contador.
#[derive(Clone)]
pub struct EstadoIntegraContador {
    pub executar: Arc<ExecutarIntegraContadorUseCase>,
}

impl EstadoIntegraContador {
    pub fn novo(
        executar: Arc<ExecutarIntegraContadorUseCase>,
    ) -> Self {
        Self { executar }
    }
}