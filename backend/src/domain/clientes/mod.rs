mod cliente;
mod cpf_cnpj;
mod filtro;

pub use cliente::{Cliente, ClienteInvalido, NovoCliente, PorteRegime, TipoCliente};
pub use cpf_cnpj::{CpfCnpj, CpfCnpjInvalido, TipoDocumento};
pub use filtro::{FiltroClientes, Paginacao, PaginacaoInvalida};
