use uuid::Uuid;
use crate::application::serpro::ports::RepositorioError;

pub trait CpfOperadorRepositorio: Send + Sync {
    fn buscar_cpf_operador(&self, usuario_id: Uuid) -> Result<Option<String>, RepositorioError>;
}
