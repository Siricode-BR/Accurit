use std::net::IpAddr;

use chrono::{DateTime, Utc};
use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;

use crate::domain::auth::Papel;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RepositorioError {
    #[error("banco de dados indisponível")]
    Indisponivel,
    #[error("falha ao acessar dados")]
    Dados,
    #[error("e-mail já cadastrado")]
    EmailExistente,
    #[error("CPF já cadastrado neste escritório")]
    CpfExistente,
    #[error("escritório inexistente")]
    EscritorioInexistente,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum UsuarioError {
    #[error("validação: {0}")]
    Validacao(String),
    #[error("usuário não encontrado")]
    NaoEncontrado,
    #[error("escritório não encontrado")]
    EscritorioNaoEncontrado,
    #[error("conflito: {0}")]
    Conflito(String),
    #[error("repositório: {0}")]
    Repositorio(#[from] RepositorioError),
    #[error("falha ao processar senha")]
    Senha,
}

/// Quem executa a operação — define o escopo de tenant e como a auditoria é gravada.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrigemAtor {
    /// Titular logado na plataforma do escritório: restrito a este escritório.
    Escritorio(Uuid),
    /// Gestor do backoffice SaaS: enxerga todos os escritórios.
    Backoffice,
}

#[derive(Debug, Clone)]
pub struct Ator {
    pub id: Uuid,
    pub origem: OrigemAtor,
    pub ip: Option<IpAddr>,
}

impl Ator {
    pub(crate) const fn escopo(&self) -> Option<Uuid> {
        match self.origem {
            OrigemAtor::Escritorio(id) => Some(id),
            OrigemAtor::Backoffice => None,
        }
    }
}

/// Usuário com dados do escritório; nunca carrega `senha_hash`.
#[derive(Debug, Clone)]
pub struct UsuarioDetalhe {
    pub id: Uuid,
    pub escritorio_id: Uuid,
    pub escritorio_nome: String,
    pub escritorio_slug: String,
    pub nome: String,
    pub email: String,
    pub cpf: String,
    pub papel: Papel,
    pub ativo: bool,
    pub ultimo_login_em: Option<DateTime<Utc>>,
    pub criado_em: DateTime<Utc>,
}

#[derive(Debug, Clone, Default)]
pub struct FiltroUsuarios {
    pub escritorio_id: Option<Uuid>,
    pub busca: Option<String>,
    pub ativo: Option<bool>,
    /// Fase E: paginação no servidor; `None` mantém o comportamento legado
    /// (todas as linhas, sem contagem).
    pub pagina: Option<i64>,
    pub tamanho: Option<i64>,
}

/// Listagem paginada de usuários (Fase E).
#[derive(Debug, Clone)]
pub struct ListagemUsuarios {
    pub itens: Vec<UsuarioDetalhe>,
    pub total: i64,
    pub pagina: i64,
    pub tamanho: i64,
}

#[derive(Debug, Clone)]
pub struct UsuarioGestaoNovo {
    pub escritorio_id: Uuid,
    pub nome: String,
    pub email: String,
    pub cpf: String,
    pub senha_hash: String,
    pub papel: Papel,
}

#[derive(Debug, Clone, Default)]
pub struct AlteracaoUsuario {
    pub nome: Option<String>,
    pub papel: Option<Papel>,
    pub ativo: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct EventoAuditoriaUsuario {
    pub escritorio_id: Uuid,
    pub ator: Ator,
    pub acao: &'static str,
    pub usuario_alvo_id: Uuid,
    pub valor_anterior: Option<Value>,
    pub valor_novo: Option<Value>,
}

pub trait UsuarioGestaoRepositorio: Send + Sync {
    fn listar(&self, filtro: &FiltroUsuarios) -> Result<Vec<UsuarioDetalhe>, RepositorioError>;

    /// Fase E — listagem com contagem e LIMIT/OFFSET no servidor.
    fn listar_paginado(
        &self,
        filtro: &FiltroUsuarios,
    ) -> Result<ListagemUsuarios, RepositorioError>;

    /// Com `escritorio_id`, só encontra usuários desse escritório (isolamento de tenant).
    fn obter(
        &self,
        id: Uuid,
        escritorio_id: Option<Uuid>,
    ) -> Result<Option<UsuarioDetalhe>, RepositorioError>;

    fn criar(&self, novo: &UsuarioGestaoNovo) -> Result<Uuid, RepositorioError>;

    fn atualizar(&self, id: Uuid, alteracao: &AlteracaoUsuario) -> Result<(), RepositorioError>;

    /// Troca o hash e zera falhas/bloqueio.
    fn definir_senha(&self, id: Uuid, senha_hash: &str) -> Result<(), RepositorioError>;

    fn revogar_sessoes(&self, id: Uuid) -> Result<u64, RepositorioError>;

    /// Titulares (`CLIENTE`) ativos do escritório, ignorando `excluindo`.
    fn contar_titulares_ativos(
        &self,
        escritorio_id: Uuid,
        excluindo: Uuid,
    ) -> Result<i64, RepositorioError>;

    fn registrar_auditoria(&self, evento: &EventoAuditoriaUsuario) -> Result<(), RepositorioError>;
}
