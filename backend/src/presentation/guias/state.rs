//! Fase D — Estado de apresentação para guias.

use std::sync::Arc;

use crate::application::guias::{
    BaixarGuiaUseCase, BuscarGuiaUseCase, EmitirGuiaUseCase, ListarGuiasUseCase,
};

#[derive(Clone)]
pub struct EstadoGuias {
    pub emitir_guia: Arc<EmitirGuiaUseCase>,
    pub listar_guias: Arc<ListarGuiasUseCase>,
    pub buscar_guia: Arc<BuscarGuiaUseCase>,
    pub baixar_guia: Arc<BaixarGuiaUseCase>,
}

impl EstadoGuias {
    pub fn novo(
        emitir_guia: Arc<EmitirGuiaUseCase>,
        listar_guias: Arc<ListarGuiasUseCase>,
        buscar_guia: Arc<BuscarGuiaUseCase>,
        baixar_guia: Arc<BaixarGuiaUseCase>,
    ) -> Self {
        Self {
            emitir_guia,
            listar_guias,
            buscar_guia,
            baixar_guia,
        }
    }
}
