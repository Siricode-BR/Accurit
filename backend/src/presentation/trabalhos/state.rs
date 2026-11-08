//! Fase B — Estado de apresentação para trabalhos agendados e consultas em lote.

use std::sync::Arc;

use crate::application::trabalhos::{
    BuscarConsultaLoteUseCase, BuscarTrabalhoAgendadoUseCase, CriarConsultaLoteUseCase,
    CriarTrabalhoAgendadoUseCase, HabilitarDesabilitarTrabalhoUseCase, ListarConsultasLoteUseCase,
    ListarExecucoesTrabalhoUseCase, ListarItensConsultaLoteUseCase,
    ListarTrabalhosAgendadosUseCase,
};

#[derive(Clone)]
pub struct EstadoTrabalhos {
    pub criar_trabalho: Arc<CriarTrabalhoAgendadoUseCase>,
    pub listar_trabalhos: Arc<ListarTrabalhosAgendadosUseCase>,
    pub buscar_trabalho: Arc<BuscarTrabalhoAgendadoUseCase>,
    pub habilitar_desabilitar: Arc<HabilitarDesabilitarTrabalhoUseCase>,
    pub listar_execucoes: Arc<ListarExecucoesTrabalhoUseCase>,
    pub criar_lote: Arc<CriarConsultaLoteUseCase>,
    pub listar_lotes: Arc<ListarConsultasLoteUseCase>,
    pub buscar_lote: Arc<BuscarConsultaLoteUseCase>,
    pub listar_itens_lote: Arc<ListarItensConsultaLoteUseCase>,
}

#[allow(clippy::too_many_arguments)]
impl EstadoTrabalhos {
    pub fn novo(
        criar_trabalho: Arc<CriarTrabalhoAgendadoUseCase>,
        listar_trabalhos: Arc<ListarTrabalhosAgendadosUseCase>,
        buscar_trabalho: Arc<BuscarTrabalhoAgendadoUseCase>,
        habilitar_desabilitar: Arc<HabilitarDesabilitarTrabalhoUseCase>,
        listar_execucoes: Arc<ListarExecucoesTrabalhoUseCase>,
        criar_lote: Arc<CriarConsultaLoteUseCase>,
        listar_lotes: Arc<ListarConsultasLoteUseCase>,
        buscar_lote: Arc<BuscarConsultaLoteUseCase>,
        listar_itens_lote: Arc<ListarItensConsultaLoteUseCase>,
    ) -> Self {
        Self {
            criar_trabalho,
            listar_trabalhos,
            buscar_trabalho,
            habilitar_desabilitar,
            listar_execucoes,
            criar_lote,
            listar_lotes,
            buscar_lote,
            listar_itens_lote,
        }
    }
}
