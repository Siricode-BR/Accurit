//! Rate limit local por escritório para consultas SERPRO (SPEC
//! §3.3): falha rápida antes de qualquer chamada upstream —
//! requisições bloqueadas não geram trilha (não há chamada externa)
//! e não chegam ao gateway.

use std::num::NonZeroU32;
use std::sync::Arc;

use governor::clock::DefaultClock;
use governor::state::keyed::DefaultKeyedStateStore;
use governor::{Quota, RateLimiter};
use uuid::Uuid;

use crate::application::divida_ativa::ports::LimitadorConsultas;

type LimitadorConsultivo = RateLimiter<Uuid, DefaultKeyedStateStore<Uuid>, DefaultClock>;

/// Quota por escritório: `por_segundo` consultas/s com burst curto
/// (`burst = por_segundo`), imune a vazamento de quotas entre
/// tenants.
#[derive(Clone)]
pub struct LimitadorPorEscritorio {
    limitador: Arc<LimitadorConsultivo>,
}

impl LimitadorPorEscritorio {
    /// Constrói quota; valor inválido (0) degrada a 1/s — nunca
    /// bloqueia a operação por falha de configuração.
    pub fn novo(por_segundo: u32) -> Self {
        let por_segundo = NonZeroU32::new(por_segundo.max(1)).expect("1/s mínimo");
        let burst = NonZeroU32::new(por_segundo.get()).expect("burst ≥ 1");
        Self {
            limitador: Arc::new(RateLimiter::dashmap(
                Quota::per_second(por_segundo).allow_burst(burst),
            )),
        }
    }
}

impl LimitadorConsultas for LimitadorPorEscritorio {
    fn adquirir(&self, escritorio_id: Uuid) -> Result<(), ()> {
        self.limitador.check_key(&escritorio_id).map_err(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quota_por_escritorio_isolada() {
        let limitador = LimitadorPorEscritorio::novo(1);
        let a = Uuid::nil();
        let b = Uuid::now_v7();

        assert!(limitador.adquirir(a).is_ok());
        assert!(limitador.adquirir(a).is_err()); // 1/s esgotado
        assert!(limitador.adquirir(b).is_ok()); // outro tenant passa
    }

    #[test]
    fn quota_zero_degrada_para_um() {
        let limitador = LimitadorPorEscritorio::novo(0);
        let chave = Uuid::nil();
        assert!(limitador.adquirir(chave).is_ok());
        assert!(limitador.adquirir(chave).is_err());
    }
}
