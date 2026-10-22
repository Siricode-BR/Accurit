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

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtualizarClienteRequest {
    pub tipo: Option<TipoCliente>,
    pub porte_regime: Option<Option<String>>,
    pub cpf_cnpj: Option<String>,
    pub nome_razao_social: Option<String>,
    pub nome_fantasia: Option<Option<String>>,
    pub email: Option<Option<String>>,
    pub telefone: Option<Option<String>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListarClientesQuery {
    pub filtro: Option<String>,
    pub pagina: Option<i64>,
    pub tamanho: Option<i64>,
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