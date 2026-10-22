use std::net::IpAddr;

use uuid::Uuid;

use crate::domain::clientes::{FiltroClientes, Paginacao, TipoCliente};

#[derive(Debug, Clone)]
pub struct PoliticaClientes {
    pub max_nome_len: usize,
    pub max_fantasia_len: usize,
    pub max_email_len: usize,
    pub max_telefone_len: usize,
}

impl Default for PoliticaClientes {
    fn default() -> Self {
        Self {
            max_nome_len: 200,
            max_fantasia_len: 200,
            max_email_len: 254,
            max_telefone_len: 30,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CriarClienteCommand {
    pub escritorio_id: Uuid,
    pub tipo: TipoCliente,
    pub porte_regime: Option<crate::domain::clientes::PorteRegime>,
    pub cpf_cnpj: String,
    pub nome_razao_social: String,
    pub nome_fantasia: Option<String>,
    pub email: Option<String>,
    pub telefone: Option<String>,
    pub ip: Option<IpAddr>,
    pub usuario_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct ObterClienteCommand {
    pub escritorio_id: Uuid,
    pub id: Uuid,
}

#[derive(Debug, Clone)]
pub struct ListarClientesCommand {
    pub escritorio_id: Uuid,
    pub filtro: FiltroClientes,
    pub paginacao: Paginacao,
}

#[derive(Debug, Clone)]
pub struct AtualizarClienteCommand {
    pub escritorio_id: Uuid,
    pub id: Uuid,
    pub tipo: Option<TipoCliente>,
    pub porte_regime: Option<Option<crate::domain::clientes::PorteRegime>>,
    pub cpf_cnpj: Option<String>,
    pub nome_razao_social: Option<String>,
    pub nome_fantasia: Option<Option<String>>,
    pub email: Option<Option<String>>,
    pub telefone: Option<Option<String>>,
    pub ip: Option<IpAddr>,
    pub usuario_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct ExcluirClienteCommand {
    pub escritorio_id: Uuid,
    pub id: Uuid,
    pub ip: Option<IpAddr>,
    pub usuario_id: Uuid,
}

pub(crate) fn normalizar_cpf_cnpj(valor: &str) -> String {
    valor
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect()
}

pub(crate) fn validar_nome(valor: &str, max_len: usize) -> Result<(), String> {
    let limpo = valor.trim();
    if limpo.is_empty() || limpo.chars().count() > max_len {
        return Err("nome deve ter entre 1 e 200 caracteres".into());
    }
    Ok(())
}

pub(crate) fn validar_opcional_max_len(valor: &Option<String>, max_len: usize, campo: &str) -> Result<(), String> {
    if let Some(v) = valor {
        let limpo = v.trim();
        if !limpo.is_empty() && limpo.chars().count() > max_len {
            return Err(format!("{} deve ter no m\u{00e1}ximo {} caracteres", campo, max_len));
        }
    }
    Ok(())
}