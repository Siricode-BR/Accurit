mod administrador;
mod audiencia;
mod cpf;
mod papel;
mod usuario;

/// O CNPJ do escritório é o mesmo value object do domínio de tenancy.
pub use crate::domain::tenancy::{Cnpj, CnpjInvalido};
pub use administrador::{Administrador, SessaoAdmin, SessaoAdminComAdministrador};
pub use audiencia::{Audiencia, AudienciaInvalida};
pub use cpf::{Cpf, CpfInvalido};
pub use papel::{Papel, PapelInvalido};
pub use usuario::{Escritorio, Sessao, SessaoComUsuario, Usuario};
