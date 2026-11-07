use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::clientes::{FiltroClientes, Paginacao, TipoCliente};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClienteResponse {
    pub id: Uuid,
    pub escritorio_id: Uuid,
    pub tipo: TipoCliente,
    pub porte_regime: Option<String>,
    pub cpf_cnpj: String,
    pub nome_razao_social: String,
    pub nome_fantasia: Option<String>,
    pub email: Option<String>,
    pub telefone: Option<String>,
    pub criado_em: chrono::DateTime<chrono::Utc>,
    pub atualizado_em: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListagemClientesResponse {
    pub itens: Vec<ClienteResponse>,
    pub total: i64,
    pub pagina: i64,
    pub tamanho: i64,
}

impl From<crate::application::clientes::ClientePublico> for ClienteResponse {
    fn from(cliente: crate::application::clientes::ClientePublico) -> Self {
        Self {
            id: cliente.id,
            escritorio_id: cliente.escritorio_id,
            tipo: cliente.tipo,
            porte_regime: cliente.porte_regime.map(|p| p.como_str().to_owned()),
            cpf_cnpj: cliente.cpf_cnpj,
            nome_razao_social: cliente.nome_razao_social,
            nome_fantasia: cliente.nome_fantasia,
            email: cliente.email,
            telefone: cliente.telefone,
            criado_em: cliente.criado_em,
            atualizado_em: cliente.atualizado_em,
        }
    }
}

impl From<crate::application::clientes::ListagemClientes> for ListagemClientesResponse {
    fn from(listagem: crate::application::clientes::ListagemClientes) -> Self {
        Self {
            itens: listagem.itens.into_iter().map(Into::into).collect(),
            total: listagem.total,
            pagina: listagem.pagina,
            tamanho: listagem.tamanho,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CriarClienteRequest {
    pub tipo: TipoCliente,
    pub porte_regime: Option<String>,
    pub cpf_cnpj: String,
    pub nome_razao_social: String,
    pub nome_fantasia: Option<String>,
    pub email: Option<String>,
    pub telefone: Option<String>,
}

mod opcoes_duas {
    use serde::{Deserialize, Deserializer};

    pub fn duas<'de, T, D>(deserializador: D) -> Result<Option<Option<T>>, D::Error>
    where
        T: Deserialize<'de>,
        D: Deserializer<'de>,
    {
        Deserialize::deserialize(deserializador).map(Some)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtualizarClienteRequest {
    pub tipo: Option<TipoCliente>,
    pub porte_regime: Option<Option<String>>,
    pub cpf_cnpj: Option<String>,
    pub nome_razao_social: Option<String>,
    #[serde(default, deserialize_with = "opcoes_duas::duas")]
    pub nome_fantasia: Option<Option<String>>,
    #[serde(default, deserialize_with = "opcoes_duas::duas")]
    pub email: Option<Option<String>>,
    #[serde(default, deserialize_with = "opcoes_duas::duas")]
    pub telefone: Option<Option<String>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListarClientesQuery {
    #[serde(alias = "page")]
    pub pagina: Option<i64>,
    #[serde(alias = "pageSize")]
    pub tamanho: Option<i64>,
    pub filtro: Option<String>,
}

impl ListarClientesQuery {
    pub fn para_filtro_e_paginacao(
        self,
    ) -> Result<(FiltroClientes, Paginacao), crate::presentation::error::ApiError> {
        let filtro = self
            .filtro
            .unwrap_or_default()
            .parse::<FiltroClientes>()
            .map_err(|_| crate::presentation::error::ApiError::filtro_invalido())?;

        let paginacao = Paginacao::nova(self.pagina, self.tamanho)
            .map_err(|_| crate::presentation::error::ApiError::paginacao_invalida())?;

        Ok((filtro, paginacao))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProvisionarTenantRequest {
    pub slug: String,
    pub nome: String,
    pub email_titular: String,
    pub cpf_titular: String,
    pub senha_titular: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProvisionarTenantResponse {
    pub escritorio_id: Uuid,
    pub slug: String,
    pub titular_id: Uuid,
    pub email_titular: String,
}

// ===== Consultas fiscais do cliente (SPEC §9.3 — somente leitura) =====

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsultaFiscalResponse {
    pub id: Uuid,
    pub produto: String,
    pub servico: String,
    pub verbo: Option<String>,
    pub ambiente: String,
    pub estado: String,
    pub fonte_consulta: Option<String>,
    pub chave_consulta_mascarada: Option<String>,
    pub status_code: Option<i32>,
    pub criado_em: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListagemConsultasResponse {
    pub itens: Vec<ConsultaFiscalResponse>,
    pub total: i64,
    pub pagina: i64,
    pub tamanho: i64,
}

/// Base do cartão por serviço fiscal: última tentativa de cada serviço.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SituacaoFiscalServicoResponse {
    pub servico: String,
    pub estado: String,
    pub ambiente: String,
    pub fonte_consulta: Option<String>,
    pub chave_consulta_mascarada: Option<String>,
    pub status_code: Option<i32>,
    pub criado_em: chrono::DateTime<chrono::Utc>,
}

/// Aviso consultivo permanente (ADR-0002): leitura pura da Dívida Ativa
/// — nunca negociação, parcelamento, pagamento ou emissão de DARF.
/// Texto canônico: `presentation::divida_ativa::dto::AVISO_CONSULTIVO`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SituacaoFiscalResponse {
    pub aviso: &'static str,
    pub servicos: Vec<SituacaoFiscalServicoResponse>,
}

impl From<crate::application::clientes::ConsultaFiscalDoCliente> for ConsultaFiscalResponse {
    fn from(consulta: crate::application::clientes::ConsultaFiscalDoCliente) -> Self {
        Self {
            id: consulta.id,
            produto: consulta.produto,
            servico: consulta.servico,
            verbo: consulta.verbo,
            ambiente: consulta.ambiente,
            estado: consulta.estado,
            fonte_consulta: consulta.fonte_consulta,
            chave_consulta_mascarada: consulta.chave_consulta_mascarada,
            status_code: consulta.status_code,
            criado_em: consulta.criado_em,
        }
    }
}

impl From<crate::application::clientes::ListagemConsultasDoCliente> for ListagemConsultasResponse {
    fn from(listagem: crate::application::clientes::ListagemConsultasDoCliente) -> Self {
        Self {
            itens: listagem.itens.into_iter().map(Into::into).collect(),
            total: listagem.total,
            pagina: listagem.pagina,
            tamanho: listagem.tamanho,
        }
    }
}

impl From<crate::application::clientes::SituacaoFiscalServico> for SituacaoFiscalServicoResponse {
    fn from(servico: crate::application::clientes::SituacaoFiscalServico) -> Self {
        Self {
            servico: servico.servico,
            estado: servico.estado,
            ambiente: servico.ambiente,
            fonte_consulta: servico.fonte_consulta,
            chave_consulta_mascarada: servico.chave_consulta_mascarada,
            status_code: servico.status_code,
            criado_em: servico.criado_em,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListarConsultasQuery {
    #[serde(alias = "page")]
    pub pagina: Option<i64>,
    #[serde(alias = "pageSize")]
    pub tamanho: Option<i64>,
    pub estado: Option<String>,
}

impl ListarConsultasQuery {
    pub fn para_filtro_e_paginacao(
        self,
    ) -> Result<
        (
            crate::application::clientes::FiltroConsultasCliente,
            Paginacao,
        ),
        crate::presentation::error::ApiError,
    > {
        let estado = match self.estado.as_deref() {
            None | Some("") => None,
            Some(valor) if valor_e_estado_valido(valor) => Some(valor.to_owned()),
            Some(_) => {
                return Err(crate::presentation::error::ApiError::validacao(
                    "estado deve ser um código SCREAMING_SNAKE de até 32 caracteres",
                ));
            }
        };

        let paginacao = Paginacao::nova(self.pagina, self.tamanho)
            .map_err(|_| crate::presentation::error::ApiError::paginacao_invalida())?;

        Ok((
            crate::application::clientes::FiltroConsultasCliente { estado },
            paginacao,
        ))
    }
}

/// Mesmo shape aceito na coluna `estado` (VARCHAR(32), SCREAMING_SNAKE).
fn valor_e_estado_valido(valor: &str) -> bool {
    !valor.is_empty()
        && valor.len() <= 32
        && valor.bytes().all(|b| b.is_ascii_uppercase() || b == b'_')
}

#[cfg(test)]
mod tests_consultas {
    use super::{ListarConsultasQuery, valor_e_estado_valido};

    #[test]
    fn aceita_estados_screaming_snake() {
        assert!(valor_e_estado_valido("SUCESSO"));
        assert!(valor_e_estado_valido("SEM_PENDENCIAS"));
        assert!(valor_e_estado_valido("_"));
        assert!(!valor_e_estado_valido(""));
        assert!(!valor_e_estado_valido("sucesso"));
        assert!(!valor_e_estado_valido(" estado"));
        assert!(!valor_e_estado_valido(
            &(0..40).map(|_| 'A').collect::<String>()
        ));
    }

    #[test]
    fn filtros_meta_e_paginacao_invalida_sao_reportados() {
        let consulta = ListarConsultasQuery {
            pagina: Some(0),
            tamanho: None,
            estado: None,
        };
        assert!(consulta.clone().para_filtro_e_paginacao().is_err());

        let consulta = ListarConsultasQuery {
            pagina: Some(1),
            tamanho: Some(2),
            estado: Some("VALIDO".into()),
        };
        let (filtro, paginacao) = consulta.para_filtro_e_paginacao().expect("válido");
        assert_eq!(filtro.estado.as_deref(), Some("VALIDO"));
        assert_eq!(paginacao.pagina, 1);
        assert_eq!(paginacao.tamanho, 2);
    }
}
