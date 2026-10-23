use std::sync::Arc;

use crate::application::serpro::{AlterarModoTrialSerproUseCase, ObterConfiguracaoSerproUseCase};

/// Estado compartilhado das rotas SERPRO: backoffice (leitura + toggle) e
/// portal (leitura).
#[derive(Clone)]
pub struct SerproState {
    pub obter_configuracao: Arc<ObterConfiguracaoSerproUseCase>,
    pub alterar_modo: Arc<AlterarModoTrialSerproUseCase>,
}

impl SerproState {
    pub fn novo(
        obter_configuracao: Arc<ObterConfiguracaoSerproUseCase>,
        alterar_modo: Arc<AlterarModoTrialSerproUseCase>,
    ) -> Self {
        Self {
            obter_configuracao,
            alterar_modo,
        }
    }
}
