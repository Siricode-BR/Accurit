pub mod carteira_use_cases;
pub mod importar;
mod models;
mod ports;
mod use_cases;
pub use carteira_use_cases::{
    CarteiraClienteUseCase, GestaoAcessoUsuariosUseCase, GestaoClienteTagsUseCase,
    GestaoFiliaisUseCase, GestaoSociosContatosUseCase, VerificarAcessoUseCase,
};

pub use importar::{
    AnaliseImportacao, ErroLinhaImportacao, FormatoImportacao, ImportarClientesUseCase, MAX_LINHAS,
};
pub use models::PoliticaClientes;
pub use ports::{
    AtualizarClienteCommand, AuditoriaClienteRepositorio, ClienteError, ClientePublico,
    ClienteRepositorio, ConsultaFiscalDoCliente, CriarClienteCommand, EventoAuditoriaCliente,
    ExcluirClienteCommand, FiltroConsultasCliente, ListagemClientes, ListagemConsultasDoCliente,
    ListarClientesCommand, ListarConsultasDoClienteCommand, NovaClienteTagCommand,
    NovoClienteFilialCommand, NovoSocioContatoCommand, ObterClienteCommand,
    PapelSocioContatoDomain, RepositorioAcessoUsuarios, RepositorioCarteiraClientes,
    RepositorioConsultasDoCliente, RepositorioError, SituacaoFiscalDoClienteCommand,
    SituacaoFiscalServico,
};
pub use use_cases::{
    AtualizarClienteUseCase, CriarClienteUseCase, ExcluirClienteUseCase, ListarClientesUseCase,
    ListarConsultasDoClienteUseCase, ObterClienteUseCase, SituacaoFiscalDoClienteUseCase,
};
