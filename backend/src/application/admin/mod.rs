mod ports;
mod use_case;
mod use_case_administradores;

pub use ports::{
    AdministradorAtualizar, AdministradorDto, AdministradorNovo, AdministradorRepositorio,
    AuditoriaAdminRepositorio, EscritorioNovo, EscritorioRepositorio, EscritorioResumo,
    EventoAuditoriaAdmin, Paginacao, ProvisionarError, ProvisionarTenantCommand,
    ProvisionarTenantResultado, RepositorioError, UsuarioAdminRepositorio, UsuarioNovo,
};
pub use use_case::{ListarEscritoriosUseCase, ProvisionarTenantUseCase};
pub(crate) use use_case::{validar_cpf, validar_email, validar_nome, validar_senha};
pub use use_case_administradores::GestaoAdministradoresUseCase;
