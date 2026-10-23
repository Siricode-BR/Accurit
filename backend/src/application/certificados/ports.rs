use crate::domain::certificados::{
    Certificado, CertificadoInvalido, FingerprintInvalido, TipoCertificado,
};
use crate::domain::tenancy::EscritorioId;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RepositorioError {
    #[error("banco de dados indispon\u{00ed}vel")]
    Indisponivel,
    #[error("erro de dados")]
    Dados,
    #[error("certificado n\u{00e3}o encontrado")]
    NaoEncontrado,
    #[error("fingerprint j\u{00e1} existe para este escrit\u{00f3}rio")]
    FingerprintDuplicado,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CertificadoError {
    #[error(transparent)]
    Dominio(#[from] CertificadoInvalido),
    #[error(transparent)]
    Repositorio(#[from] RepositorioError),
    #[error(transparent)]
    Fingerprint(#[from] FingerprintInvalido),
    #[error("criptografia falhou")]
    Criptografia,
    #[error("acesso negado")]
    AcessoNegado,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CertificadoPublico {
    pub id: Uuid,
    pub escritorio_id: Uuid,
    pub tipo: TipoCertificado,
    pub titular_cpf_cnpj: String,
    pub algoritmo: String,
    pub fingerprint: String,
    pub valido_de: chrono::DateTime<chrono::Utc>,
    pub valido_ate: chrono::DateTime<chrono::Utc>,
    pub ativo: bool,
    pub criado_em: chrono::DateTime<chrono::Utc>,
    pub dias_para_expirar: i64,
}

impl From<&Certificado> for CertificadoPublico {
    fn from(c: &Certificado) -> Self {
        Self {
            id: c.id,
            escritorio_id: c.escritorio_id.into(),
            tipo: c.tipo,
            titular_cpf_cnpj: c.titular_cpf_cnpj.clone(),
            algoritmo: c.algoritmo.como_str().to_string(),
            fingerprint: c.fingerprint.como_str().to_string(),
            valido_de: c.valido_de,
            valido_ate: c.valido_ate,
            ativo: c.ativo,
            criado_em: c.criado_em,
            dias_para_expirar: c.dias_para_expirar(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CriarCertificadoCommand {
    pub escritorio_id: EscritorioId,
    pub tipo: TipoCertificado,
    pub titular_cpf_cnpj: String,
    pub conteudo: Vec<u8>,
    pub senha: Option<String>,
    pub algoritmo: String,
    pub fingerprint: String,
    pub valido_de: chrono::DateTime<chrono::Utc>,
    pub valido_ate: chrono::DateTime<chrono::Utc>,
    pub ip: Option<std::net::IpAddr>,
    pub usuario_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct ListarCertificadosCommand {
    pub escritorio_id: EscritorioId,
    pub apenas_validos: bool,
    pub pagina: i64,
    pub tamanho: i64,
}

#[derive(Debug, Clone)]
pub struct ObterCertificadoCommand {
    pub escritorio_id: EscritorioId,
    pub id: Uuid,
}

#[derive(Debug, Clone)]
pub struct RevogarCertificadoCommand {
    pub escritorio_id: EscritorioId,
    pub id: Uuid,
    pub ip: Option<std::net::IpAddr>,
    pub usuario_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct ListagemCertificados {
    pub itens: Vec<CertificadoPublico>,
    pub total: i64,
    pub pagina: i64,
    pub tamanho: i64,
}

#[derive(Debug, Clone)]
pub struct EventoAuditoriaCertificado {
    pub escritorio_id: EscritorioId,
    pub usuario_id: Uuid,
    pub acao: String,
    pub entidade: String,
    pub entidade_id: Uuid,
    pub valor_anterior: Option<serde_json::Value>,
    pub valor_novo: Option<serde_json::Value>,
    pub ip: Option<std::net::IpAddr>,
}

pub trait CertificadoRepositorio: Send + Sync {
    fn criar(
        &self,
        escritorio_id: EscritorioId,
        certificado: &Certificado,
    ) -> Result<(), RepositorioError>;

    fn buscar_por_id(
        &self,
        escritorio_id: EscritorioId,
        id: Uuid,
    ) -> Result<Option<Certificado>, RepositorioError>;

    fn buscar_por_fingerprint(
        &self,
        escritorio_id: EscritorioId,
        fingerprint: &str,
    ) -> Result<Option<Certificado>, RepositorioError>;

    fn listar(
        &self,
        escritorio_id: EscritorioId,
        apenas_validos: bool,
        pagina: i64,
        tamanho: i64,
    ) -> Result<ListagemCertificados, RepositorioError>;

    fn revogar(&self, escritorio_id: EscritorioId, id: Uuid) -> Result<bool, RepositorioError>;

    fn atualizar(
        &self,
        escritorio_id: EscritorioId,
        certificado: &Certificado,
    ) -> Result<(), RepositorioError>;
}

pub trait AuditoriaCertificadoRepositorio: Send + Sync {
    fn registrar(&self, evento: &EventoAuditoriaCertificado) -> Result<(), RepositorioError>;
}

pub trait CertificadoCriptografia: Send + Sync {
    fn cifrar(&self, dados: &[u8]) -> Result<Vec<u8>, CertificadoError>;
    fn decifrar(&self, dados_cifrados: &[u8]) -> Result<Vec<u8>, CertificadoError>;
    fn gerar_fingerprint(&self, conteudo: &[u8]) -> String;
}

#[allow(dead_code)]
#[derive(Clone, Default)]
pub struct PoliticaCertificados {
    pub max_titular_len: usize,
    pub max_conteudo_bytes: usize,
}

impl PoliticaCertificados {
    #[allow(dead_code)]
    pub const fn nova() -> Self {
        Self {
            max_titular_len: 14,
            max_conteudo_bytes: 50_000,
        }
    }
}
