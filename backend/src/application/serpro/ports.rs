use std::net::IpAddr;

use serde_json::Value;
use uuid::Uuid;

use crate::domain::serpro::ambiente::AmbienteSerpro;
use crate::domain::serpro::erros::ErroAmbiente;
use crate::domain::serpro::ports::AlterarModoResultado;
use crate::domain::serpro::produto::ProdutoSerpro;

pub trait EscritorioSerproRepositorio: Send + Sync {
    fn buscar_modo_trial(
        &self,
        escritorio_id: Uuid,
    ) -> Result<Option<(bool, i64, chrono::DateTime<chrono::Utc>)>, RepositorioError>;

    fn atualizar_modo_trial(
        &self,
        escritorio_id: Uuid,
        modo_trial: bool,
        versao_esperada: i64,
    ) -> Result<(i64, chrono::DateTime<chrono::Utc>), RepositorioError>;

    fn buscar_credencial(
        &self,
        escritorio_id: Uuid,
        produto: ProdutoSerpro,
        ambiente: AmbienteSerpro,
    ) -> Result<Option<CredencialArmazenada>, RepositorioError>;
}

/// Consulta de certificado A1 ativo e dentro do prazo de validade do
/// escritório (usada para permitir a saída do trial — ADR-0005 §3).
pub trait CertificadosSerproConsulta: Send + Sync {
    fn buscar_certificado_em_validade(
        &self,
        escritorio_id: Uuid,
        agora: chrono::DateTime<chrono::Utc>,
    ) -> Result<Option<CertificadoAtivoResumo>, RepositorioError>;
}

#[derive(Debug, Clone)]
pub struct CertificadoAtivoResumo {
    pub id: Uuid,
    pub fingerprint_sha256: String,
    pub valido_ate: chrono::DateTime<chrono::Utc>,
}

/// Pós-troca de modo: invalida caches voláteis persistidos do escritório
/// (tokens e procurações) — ADR-0005 §6. Falha é logada, mas não desfaz a
/// troca: tokens são particionados por ambiente.
pub trait InvalicadorCachesSerpro: Send + Sync {
    fn invalidar_todos_do_tenant(&self, escritorio_id: Uuid) -> Result<(), RepositorioError>;
}

#[derive(Debug, Clone)]
pub struct CredencialArmazenada {
    pub id: Uuid,
    pub escritorio_id: Uuid,
    pub produto: ProdutoSerpro,
    pub ambiente: AmbienteSerpro,
    pub consumer_key_cifrado: Vec<u8>,
    pub consumer_secret_cifrado: Vec<u8>,
    pub criado_em: chrono::DateTime<chrono::Utc>,
    pub atualizado_em: chrono::DateTime<chrono::Utc>,
}

pub trait AuditoriaSerproRepositorio: Send + Sync {
    fn registrar(&self, evento: &EventoAuditoriaSerpro) -> Result<(), RepositorioError>;
}

#[derive(Debug, Clone)]
pub struct EventoAuditoriaSerpro {
    pub escritorio_id: Uuid,
    pub administrador_id: Uuid,
    pub acao: &'static str,
    pub entidade: &'static str,
    pub entidade_id: Uuid,
    pub valor_anterior: Option<Value>,
    pub valor_novo: Option<Value>,
    pub ip: Option<IpAddr>,
}

/// Porta de criptografia simétrica para credenciais/tokens SERPRO em repouso.
/// `decifrar` devolve `Zeroizing` para garantir limpeza da memória.
pub trait CriptografiaSerpro: Send + Sync {
    fn cifrar(&self, dados: &[u8]) -> Result<Vec<u8>, CriptografiaError>;
    fn decifrar(
        &self,
        dados_cifrados: &[u8],
    ) -> Result<zeroize::Zeroizing<Vec<u8>>, CriptografiaError>;
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CriptografiaError {
    #[error("falha ao cifrar: {0}")]
    Cifrar(String),
    #[error("falha ao decifrar: {0}")]
    Decifrar(String),
}

pub struct ObterConfiguracaoSerproCommand {
    pub escritorio_id: Uuid,
}

pub struct ObterConfiguracaoSerproResultado {
    pub escritorio_id: Uuid,
    pub modo_trial_serpro: bool,
    pub ambiente_efetivo: AmbienteSerpro,
    pub versao: i64,
    pub atualizado_em: chrono::DateTime<chrono::Utc>,
}

pub struct AlterarModoTrialSerproCommand {
    pub escritorio_id: Uuid,
    pub modo_trial_serpro: bool,
    pub administrador_id: Uuid,
    pub ip: Option<IpAddr>,
    pub versao_esperada: i64,
}

pub type AlterarModoTrialSerproResultado = AlterarModoResultado;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RepositorioError {
    #[error("banco de dados indisponível")]
    Indisponivel,
    #[error("falha ao acessar dados")]
    Dados,
    #[error("conflito de concorrência: versão divergente")]
    Concorrencia,
    #[error("não encontrado")]
    NaoEncontrado,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ObterConfiguracaoError {
    #[error("repositório: {0}")]
    Repositorio(#[from] RepositorioError),
    #[error("escritório não encontrado")]
    EscritorioNaoEncontrado,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AlterarModoTrialError {
    #[error("repositório: {0}")]
    Repositorio(#[from] RepositorioError),
    #[error("ambiente: {0}")]
    Ambiente(#[from] ErroAmbiente),
    #[error("auditoria: {0}")]
    Auditoria(String),
}
