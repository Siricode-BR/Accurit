use std::sync::Arc;

use crate::application::integra_contador::ports::ProvedorCatalogoIntegra;
use crate::application::integra_contador::{
    AutenticarProcuradorUseCase, ExecutarIntegraContadorUseCase, ListarProcuracoesUseCase,
    ValidarProcuracoesUseCase,
};

#[derive(Clone)]
pub struct EstadoIntegraContador {
    pub executar: Arc<ExecutarIntegraContadorUseCase>,
    pub autenticar_procurador: Arc<AutenticarProcuradorUseCase>,
    pub validar_procuracao: Arc<ValidarProcuracoesUseCase>,
    pub listar_procuracao: Arc<ListarProcuracoesUseCase>,
    pub catalogo: Arc<dyn ProvedorCatalogoIntegra>,
}

impl EstadoIntegraContador {
    pub fn novo(
        executar: Arc<ExecutarIntegraContadorUseCase>,
        autenticar_procurador: Arc<AutenticarProcuradorUseCase>,
        validar_procuracao: Arc<ValidarProcuracoesUseCase>,
        listar_procuracao: Arc<ListarProcuracoesUseCase>,
        catalogo: Arc<dyn ProvedorCatalogoIntegra>,
    ) -> Self {
        Self {
            executar,
            autenticar_procurador,
            validar_procuracao,
            listar_procuracao,
            catalogo,
        }
    }
}
