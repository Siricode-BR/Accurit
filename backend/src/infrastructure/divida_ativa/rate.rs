//! Limitador local de consultas por escritório (SPEC §3.3): token
//! bucket (governor) com chave = `escritorio_id`. Rejeição local vira
//! erro de domínio — nunca um burst contra a SERPRO.

use std::num::NonZeroU32;
use std::sync::Arc;

use governor::{DefaultKeyedRateLimiter, Quota, RateLimiter};
use uuid::Uuid;

use crate::domain::divida_ativa::erros::ErroDividaAtiva;
use crate::domain::divida_ativa::ports::LimitadorConsultas;

#[derive(Clone)]
pub struct LimitadorPorEscritorio {
    limitador: Arc<DefaultKeyedRateLimiter<Uuid>>,
}

impl LimitadorPorEscritorio {
    /// `por_segundo` > 0 — a config já valida inteiro positivo.
    pub fn novo(por_segundo: u32) -> Self {
        let quota = Quota::per_second(NonZeroU32::new(por_segundo).unwrap_or(NonZeroU32::MIN));
        Self {
            limitador: Arc::new(RateLimiter::dashmap(quota)),
        }
    }
}

impl LimitadorConsultas for LimitadorPorEscritorio {
    fn permitir(&self, escritorio_id: Uuid) -> Result<(), ErroDividaAtiva> {
        match self.limitador.check_key(&escritorio_id) {
            Ok(()) => Ok(()),
            Err(_atraso) => Err(ErroDividaAtiva::LimiteLocalAtingido),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn primeiro_uso_e_permitido() {
        let limitador = LimitadorPorEscritorio::novo(4);
        let resultado = LimitadorConsultas::permitir(&limitador, Uuid::now_v7());
        assert_eq!(resultado, Ok(()));
    }

    #[tokio::test]
    async fn burst_acima_da_cota_e_rejeitado() {
        let escritorio = Uuid::now_v7();
        let limitador = LimitadorPorEscritorio::novo(1);
        assert_eq!(LimitadorConsultas::permitir(&limitador, escritorio), Ok(()));
        let segundo = LimitadorConsultas::permitir(&limitador, escritorio);
        assert_eq!(segundo, Err(ErroDividaAtiva::LimiteLocalAtingido));
    }

    #[tokio::test]
    async fn escritorios_nao_disputam_a_mesma_cota() {
        let limitador = LimitadorPorEscritorio::novo(1);
        assert_eq!(
            LimitadorConsultas::permitir(&limitador, Uuid::now_v7()),
            Ok(())
        );
        assert_eq!(
            LimitadorConsultas::permitir(&limitador, Uuid::now_v7()),
            Ok(())
        );
    }
}
