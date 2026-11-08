mod ports;
mod use_case;

pub use ports::{
    AuditoriaAdminRepositorio, EscritorioNovo, EscritorioRepositorio, EscritorioResumo,
    EventoAuditoriaAdmin, Paginacao, ProvisionarError, ProvisionarTenantCommand,
    ProvisionarTenantResultado, RepositorioError, UsuarioAdminRepositorio, UsuarioNovo,
};
pub use use_case::{ListarEscritoriosUseCase, ProvisionarTenantUseCase};
pub(crate) use use_case::{validar_cpf, validar_email, validar_nome, validar_senha};
