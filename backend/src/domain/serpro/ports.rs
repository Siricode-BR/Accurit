use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::serpro::ambiente::AmbienteSerpro;
use crate::domain::serpro::credencial::CredencialSerpro;
use crate::domain::serpro::endpoints::EndpointSerpro;
use crate::domain::serpro::erros::{ErroCredencial, ErroEndpoint};
use crate::domain::serpro::produto::ProdutoSerpro;

#[async_trait]
pub trait SerproCredentialProvider: Send + Sync {
    async fn resolver(
        &self,
        escritorio_id: Uuid,
        produto: ProdutoSerpro,
        ambiente: AmbienteSerpro,
    ) -> Result<CredencialSerpro, ErroCredencial>;
}

#[async_trait]
pub trait SerproAmbienteResolver: Send + Sync {
    async fn resolver_ambiente(
        &self,
        escritorio_id: Uuid,
    ) -> Result<AmbienteSerpro, crate::domain::serpro::erros::ErroAmbiente>;

    async fn obter_modo_trial(
        &self,
        escritorio_id: Uuid,
    ) -> Result<bool, crate::domain::serpro::erros::ErroAmbiente>;
}

/// Resolve o endpoint de acesso por produto+ambiente, sem fallbacks
/// silenciosos: config ausente é erro explícito (ADR-0005 §3; D5/D6).
#[async_trait]
pub trait ResolverEndpointsSerpro: Send + Sync {
    async fn resolver(
        &self,
        produto: ProdutoSerpro,
        ambiente: AmbienteSerpro,
    ) -> Result<EndpointSerpro, ErroEndpoint>;
}

#[derive(Debug, Clone)]
pub struct AlterarModoResultado {
    pub modo_trial_serpro: bool,
    pub ambiente_efetivo: AmbienteSerpro,
    pub versao_atual: i64,
    pub atualizado_em: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn portas_resolvem_credential_impl_mockada() {
        struct ProviderMock;
        #[async_trait]
        impl SerproCredentialProvider for ProviderMock {
            async fn resolver(
                &self,
                _escritorio_id: Uuid,
                _produto: ProdutoSerpro,
                _ambiente: AmbienteSerpro,
            ) -> Result<CredencialSerpro, ErroCredencial> {
                Err(ErroCredencial::NaoEncontrada {
                    escritorio_id: Uuid::nil(),
                    produto: "INTEGRA_CONTADOR".to_owned(),
                    ambiente: "TRIAL".to_owned(),
                })
            }
        }

        assert!(
            ProviderMock
                .resolver(
                    Uuid::nil(),
                    ProdutoSerpro::IntegraContador,
                    AmbienteSerpro::Trial
                )
                .await
                .is_err()
        );
    }
}
