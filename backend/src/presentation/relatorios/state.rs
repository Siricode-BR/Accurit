//! Fase F — Estado de apresentação de relatórios.

use std::sync::Arc;

use crate::application::relatorios::use_cases::{
    AcessarRelatorioPorLinkUseCase, BuscarRelatorioUseCase, CriarLinkCompartilhamentoUseCase,
    GerarRelatorioUseCase, ListarRelatoriosUseCase, RevogarLinksRelatorioUseCase,
    ValidarLinkCompartilhamentoUseCase,
};

#[derive(Clone)]
pub struct EstadoRelatorios {
    pub gerar: Arc<GerarRelatorioUseCase>,
    pub listar: Arc<ListarRelatoriosUseCase>,
    pub buscar: Arc<BuscarRelatorioUseCase>,
    pub criar_link: Arc<CriarLinkCompartilhamentoUseCase>,
    pub validar_link: Arc<ValidarLinkCompartilhamentoUseCase>,
    pub acessar_por_link: Arc<AcessarRelatorioPorLinkUseCase>,
    pub revogar_links: Arc<RevogarLinksRelatorioUseCase>,
}

impl EstadoRelatorios {
    pub fn novo(
        gerar: Arc<GerarRelatorioUseCase>,
        listar: Arc<ListarRelatoriosUseCase>,
        buscar: Arc<BuscarRelatorioUseCase>,
        criar_link: Arc<CriarLinkCompartilhamentoUseCase>,
        validar_link: Arc<ValidarLinkCompartilhamentoUseCase>,
        acessar_por_link: Arc<AcessarRelatorioPorLinkUseCase>,
        revogar_links: Arc<RevogarLinksRelatorioUseCase>,
    ) -> Self {
        Self {
            gerar,
            listar,
            buscar,
            criar_link,
            validar_link,
            acessar_por_link,
            revogar_links,
        }
    }
}
