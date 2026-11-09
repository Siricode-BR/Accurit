//! Fase F — Casos de uso de relatórios e link seguro de compartilhamento.

pub mod gerador;
pub mod use_cases;

pub use use_cases::{
    AcessarRelatorioPorLinkUseCase, BuscarRelatorioUseCase, ComandoCriarLink,
    ComandoGerarRelatorio, ComandoListarRelatorios, CriarLinkCompartilhamentoUseCase,
    GerarRelatorioUseCase, ListarRelatoriosUseCase, RelatorioParaCompartilhamento,
    RelatorioResumo, ValidarLinkCompartilhamentoUseCase,
};
