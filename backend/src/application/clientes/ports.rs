use std::net::IpAddr;

use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;

use crate::domain::clientes::{Cliente, CpfCnpjInvalido, FiltroClientes, Paginacao, TipoCliente};

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RepositorioError {
    #[error("banco de dados indispon\u{00ed}vel")]
    Indisponivel,
    #[error("falha ao acessar dados de clientes")]
    Dados,
    #[error("cliente n\u{00e3}o encontrado")]
    NaoEncontrado,
    #[error("CPF/CNPJ j\u{00e1} cadastrado neste escrit\u{00f3}rio")]
    Duplicado,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ClienteError {
    #[error("valida\u{00e7}\u{00e3}o de dom\u{00ed}nio: {0}")]
    Dominio(#[from] crate::domain::clientes::ClienteInvalido),
    #[error("CPF/CNPJ inv\u{00e1}lido: {0}")]
    CpfCnpj(#[from] CpfCnpjInvalido),
    #[error("reposit\u{00f3}rio: {0}")]
    Repositorio(#[from] RepositorioError),
    #[error("valida\u{00e7}\u{00e3}o: {0}")]
    Validacao(String),
    #[error("acesso negado: cliente pertence a outro escrit\u{00f3}rio")]
    AcessoNegado,
}

impl From<String> for ClienteError {
    fn from(msg: String) -> Self {
        Self::Validacao(msg)
    }
}

impl From<&str> for ClienteError {
    fn from(msg: &str) -> Self {
        Self::Validacao(msg.to_owned())
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ClientePublico {
    pub id: Uuid,
    pub escritorio_id: Uuid,
    pub tipo: TipoCliente,
    pub porte_regime: Option<crate::domain::clientes::PorteRegime>,
    pub cpf_cnpj: String,
    pub nome_razao_social: String,
    pub nome_fantasia: Option<String>,
    pub representante_nome: Option<String>,
    pub email: Option<String>,
    pub telefone: Option<String>,
    pub criado_em: chrono::DateTime<chrono::Utc>,
    pub atualizado_em: chrono::DateTime<chrono::Utc>,
}

impl From<&Cliente> for ClientePublico {
    fn from(cliente: &Cliente) -> Self {
        Self {
            id: cliente.id,
            escritorio_id: cliente.escritorio_id,
            tipo: cliente.tipo,
            porte_regime: cliente.porte_regime,
            cpf_cnpj: cliente.documento.como_str().to_owned(),
            nome_razao_social: cliente.nome_razao_social.clone(),
            nome_fantasia: cliente.nome_fantasia.clone(),
            representante_nome: cliente.representante_nome.clone(),
            email: cliente.email.clone(),
            telefone: cliente.telefone.clone(),
            criado_em: cliente.criado_em,
            atualizado_em: cliente.atualizado_em,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ListagemClientes {
    pub itens: Vec<ClientePublico>,
    pub total: i64,
    pub pagina: i64,
    pub tamanho: i64,
}

pub trait ClienteRepositorio: Send + Sync {
    fn criar(&self, escritorio_id: Uuid, cliente: &Cliente) -> Result<(), RepositorioError>;

    fn buscar_por_id(
        &self,
        escritorio_id: Uuid,
        id: Uuid,
    ) -> Result<Option<Cliente>, RepositorioError>;

    fn buscar_por_cpf_cnpj(
        &self,
        escritorio_id: Uuid,
        cpf_cnpj: &str,
    ) -> Result<Option<Cliente>, RepositorioError>;

    fn listar(
        &self,
        escritorio_id: Uuid,
        filtro: FiltroClientes,
        paginacao: Paginacao,
    ) -> Result<ListagemClientes, RepositorioError>;

    fn atualizar(&self, escritorio_id: Uuid, cliente: &Cliente) -> Result<(), RepositorioError>;

    fn excluir(&self, escritorio_id: Uuid, id: Uuid) -> Result<bool, RepositorioError>;
}

#[derive(Debug, Clone)]
pub struct CriarClienteCommand {
    pub escritorio_id: Uuid,
    pub tipo: TipoCliente,
    pub porte_regime: Option<crate::domain::clientes::PorteRegime>,
    pub cpf_cnpj: String,
    pub nome_razao_social: String,
    pub nome_fantasia: Option<String>,
    pub representante_nome: Option<String>,
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
    pub porte_regime: Option<crate::domain::clientes::PorteRegime>,
    pub cpf_cnpj: Option<String>,
    pub nome_razao_social: Option<String>,
    pub nome_fantasia: Option<Option<String>>,
    pub representante_nome: Option<Option<String>>,
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

pub trait AuditoriaClienteRepositorio: Send + Sync {
    fn registrar(&self, evento: &EventoAuditoriaCliente) -> Result<(), RepositorioError>;
}

#[derive(Debug, Clone)]
pub struct EventoAuditoriaCliente {
    pub escritorio_id: Uuid,
    pub usuario_id: Uuid,
    pub acao: &'static str,
    pub entidade: &'static str,
    pub entidade_id: Uuid,
    pub valor_anterior: Option<Value>,
    pub valor_novo: Option<Value>,
    pub ip: Option<IpAddr>,
}

// ===== Leitura de consultas fiscais por cliente (SPEC §9.3) =====

/// Uma tentativa de consulta já registrada em `consultas_fiscais`, no
/// formato público e redigido: NUNCA expõe `request_redigido` nem
/// `response_redigido` (payloads originais ficam retidos no banco).
#[derive(Debug, Clone)]
pub struct ConsultaFiscalDoCliente {
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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FiltroConsultasCliente {
    /// Estado exato da coluna `estado` (SCREAMING_SNAKE_CASE), já
    /// validado na borda de apresentação antes de chegar aqui.
    pub estado: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ListagemConsultasDoCliente {
    pub itens: Vec<ConsultaFiscalDoCliente>,
    pub total: i64,
    pub pagina: i64,
    pub tamanho: i64,
}

/// Behaviors da última tentativa por serviço (`DISTINCT ON (servico)`) —
/// alimenta os cartões de situação fiscal na tela de detalhes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SituacaoFiscalServico {
    pub servico: String,
    pub estado: String,
    pub ambiente: String,
    pub fonte_consulta: Option<String>,
    pub chave_consulta_mascarada: Option<String>,
    pub status_code: Option<i32>,
    pub criado_em: chrono::DateTime<chrono::Utc>,
}

/// Leitura histórica das consultas fiscais vinculadas a um cliente do
/// MESMO escritório. Toda query leva `escritorio_id` do contexto —
/// nunca parâmetro do corpo da requisição (ADR de tenant).
pub trait RepositorioConsultasDoCliente: Send + Sync {
    fn listar(
        &self,
        escritorio_id: Uuid,
        cliente_id: Uuid,
        filtro: FiltroConsultasCliente,
        paginacao: Paginacao,
    ) -> Result<ListagemConsultasDoCliente, RepositorioError>;

    fn situacao_fiscal(
        &self,
        escritorio_id: Uuid,
        cliente_id: Uuid,
    ) -> Result<Vec<SituacaoFiscalServico>, RepositorioError>;
}

#[derive(Debug, Clone)]
pub struct ListarConsultasDoClienteCommand {
    pub escritorio_id: Uuid,
    pub cliente_id: Uuid,
    pub filtro: FiltroConsultasCliente,
    pub paginacao: Paginacao,
}

#[derive(Debug, Clone)]
pub struct SituacaoFiscalDoClienteCommand {
    pub escritorio_id: Uuid,
    pub cliente_id: Uuid,
}
