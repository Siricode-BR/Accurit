mod ports;
mod use_case;

pub use ports::{
    AuditoriaAdminRepositorio, EscritorioNovo, EscritorioRepositorio, EscritorioResumo,
    EventoAuditoriaAdmin, ProvisionarError, ProvisionarTenantCommand, ProvisionarTenantResultado,
    RepositorioError, UsuarioAdminRepositorio, UsuarioNovo,
};
pub use use_case::{ListarEscritoriosUseCase, ProvisionarTenantUseCase};
