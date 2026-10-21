use thiserror::Error;

use super::ports::{SaudeBanco, SaudeBancoError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("banco de dados indisponível")]
pub struct SaudeIndisponivel;

pub struct ConsultarSaude<S> {
    banco: S,
}

impl<S: SaudeBanco> ConsultarSaude<S> {
    pub const fn novo(banco: S) -> Self {
        Self { banco }
    }

    pub async fn executar(&self) -> Result<(), SaudeIndisponivel> {
        match self.banco.checar().await {
            Ok(()) => Ok(()),
            Err(SaudeBancoError) => Err(SaudeIndisponivel),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct BancoOk;

    impl SaudeBanco for BancoOk {
        async fn checar(&self) -> Result<(), SaudeBancoError> {
            Ok(())
        }
    }

    struct BancoIndisponivel;

    impl SaudeBanco for BancoIndisponivel {
        async fn checar(&self) -> Result<(), SaudeBancoError> {
            Err(SaudeBancoError)
        }
    }

    #[tokio::test]
    async fn retorna_ok_quando_o_banco_esta_saudavel() {
        let caso = ConsultarSaude::novo(BancoOk);
        assert!(caso.executar().await.is_ok());
    }

    #[tokio::test]
    async fn retorna_erro_quando_o_banco_esta_indisponivel() {
        let caso = ConsultarSaude::novo(BancoIndisponivel);
        assert_eq!(caso.executar().await, Err(SaudeIndisponivel));
    }
}
