use std::future::Future;

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("banco de dados indisponível")]
pub struct SaudeBancoError;

pub trait SaudeBanco: Send + Sync {
    fn checar(&self) -> impl Future<Output = Result<(), SaudeBancoError>> + Send;
}
