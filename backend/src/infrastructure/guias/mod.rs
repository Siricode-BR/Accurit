//! Fase D — Infraestrutura de emissão de guias.
//! Provedor de emissão de guia via SERPRO Integra Contador (ainda não implementado).

use crate::domain::guias::enums::TipoGuia;
use crate::domain::guias::erros::ErroGuia;
use crate::domain::guias::ports::ProvedorEmissaoGuia;

/// Provedor de emissão de guia não implementado (placeholder).
/// A emissão real via SERPRO Integra Contador (verbo Emitir) será implementada
/// quando o contrato de produção for confirmado (P4/P5).
#[derive(Clone)]
pub struct ProvedorEmissaoGuiaNaoImplementado;

#[async_trait::async_trait]
impl ProvedorEmissaoGuia for ProvedorEmissaoGuiaNaoImplementado {
    async fn emitir_guia(
        &self,
        _cliente_cpf_cnpj: &str,
        _tipo_guia: TipoGuia,
        _competencia: &str,
    ) -> Result<(String, Option<String>), ErroGuia> {
        Err(ErroGuia::ProvedorIndisponivel(
            "Emissão de guias via SERPRO Integra Contador não implementada. Aguardando confirmação de contrato de produção (P4/P5).".into(),
        ))
    }
}
