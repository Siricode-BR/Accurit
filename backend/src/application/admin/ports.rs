use std::net::IpAddr;

use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;

use crate::domain::auth::Papel;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RepositorioError {
    #[error("banco de dados indispon\u{00ed}vel")]
    Indisponivel,
    #[error("falha ao acessar dados")]
    Dados,
    #[error("escrit\u{00f3}rio j\u{00e1} existe")]
    EscritorioExistente,
    #[error("usuario j\u{00e1} existe neste escrit\u{00f3}rio")]
    UsuarioExistente,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ProvisionarError {
    #[error("reposit\u{00f3}rio: {0}")]
    Repositorio(#[from] RepositorioError),
    #[error("valida\u{00e7}\u{00e3}o: {0}")]
    Validacao(String),
    #[error("senha inv\u{00e1}lida")]
    Senha,
}

impl From<String> for ProvisionarError {
    fn from(msg: String) -> Self {
        Self::Validacao(msg)
    }
}

impl From<&str> for ProvisionarError {
    fn from(msg: &str) -> Self {
        Self::Validacao(msg.to_owned())
    }
}

pub trait EscritorioRepositorio: Send + Sync {
    fn criar(&self, escritorio: &EscritorioNovo) -> Result<Uuid, RepositorioError>;

    fn buscar_por_slug(&self, slug: &str) -> Result<Option<Uuid>, RepositorioError>;

    /// Listagem paginada (SPEC §10.3: `page`/`pageSize`, ordenação
    /// determinística). Devolve os itens da página e o total.
    fn listar(
        &self,
        paginacao: &Paginacao,
    ) -> Result<(Vec<EscritorioResumo>, i64), RepositorioError>;
}

/// Paginação da listagem de escritórios (SPEC §10.3): página 1-based,
/// até 100 itens. A clamp acontece na apresentação antes do use case.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Paginacao {
    pub pagina: u32,
    pub por_pagina: u32,
}

impl Paginacao {
    pub const POR_PAGINA_PADRAO: u32 = 20;
    pub const POR_PAGINA_MAXIMO: u32 = 100;

    #[must_use]
    pub fn nova(pagina: u32, por_pagina: u32) -> Self {
        let pagina = if pagina == 0 { 1 } else { pagina };
        let por_pagina = if por_pagina == 0 {
            Self::POR_PAGINA_PADRAO
        } else {
            por_pagina
        };
        Self {
            pagina,
            por_pagina: por_pagina.min(Self::POR_PAGINA_MAXIMO),
        }
    }
}

pub trait UsuarioAdminRepositorio: Send + Sync {
    fn criar(&self, usuario: &UsuarioNovo) -> Result<Uuid, RepositorioError>;

    fn buscar_por_escritorio_e_email(
        &self,
        escritorio_id: Uuid,
        email: &str,
    ) -> Result<Option<Uuid>, RepositorioError>;
}

pub trait AuditoriaAdminRepositorio: Send + Sync {
    fn registrar(&self, evento: &EventoAuditoriaAdmin) -> Result<(), RepositorioError>;
}

#[derive(Debug, Clone)]
pub struct EscritorioNovo {
    pub slug: String,
    pub nome: String,
}

/// Resumo de escritório para listagem no backoffice (GET /admin/escritorios):
/// apenas metadados — sem CNPJ, contatos ou dados fiscais (SPEC §8, Fase 3).
#[derive(Debug, Clone)]
pub struct EscritorioResumo {
    pub id: Uuid,
    pub slug: String,
    pub nome: String,
    pub modo_trial_serpro: bool,
    pub versao: i64,
    pub atualizado_em: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct UsuarioNovo {
    pub escritorio_id: Uuid,
    pub email: String,
    pub cpf: String,
    pub senha_hash: String,
    pub papel: Papel,
}

#[derive(Debug, Clone)]
pub struct ProvisionarTenantCommand {
    pub slug: String,
    pub nome: String,
    pub email_titular: String,
    pub cpf_titular: String,
    pub senha_titular: String,
    pub ip: Option<IpAddr>,
    pub administrador_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct ProvisionarTenantResultado {
    pub escritorio_id: Uuid,
    pub slug: String,
    pub titular_id: Uuid,
    pub email_titular: String,
}

#[derive(Debug, Clone)]
pub struct EventoAuditoriaAdmin {
    pub escritorio_id: Option<Uuid>,
    pub administrador_id: Uuid,
    pub acao: &'static str,
    pub entidade: &'static str,
    pub entidade_id: Uuid,
    pub valor_anterior: Option<Value>,
    pub valor_novo: Option<Value>,
    pub ip: Option<IpAddr>,
}
