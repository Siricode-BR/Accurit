//! Portas do consultivo de Dívida Ativa (Fase 4): transporte HTTP,
//! catálogos de tenant, histórico `consultas_fiscais`, rate limit
//! local e a chave alternativa do fallback D2. Nada de HTTP/Diesel
//! aqui — só contratos.

use async_trait::async_trait;
use uuid::Uuid;

use crate::application::serpro::ports::RepositorioError;
use crate::domain::serpro::endpoints::EndpointSerpro;

/// Serviço (path) da Dívida Ativa consultado na chamada.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServicoDividaSerpro {
    /// `GET /devedor/{cpfCnpj}` — inscrições do devedor.
    Devedor,
    /// `GET /inscricao/{numeroInscricao}` — detalhe da inscrição.
    Inscricao,
}

impl ServicoDividaSerpro {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Devedor => "DEVEDOR",
            Self::Inscricao => "INSCRICAO",
        }
    }
}

/// Solicitação completa para o transporte HTTP: endpoint já
/// resolvido (URL + estratégia de autenticação), chave normalizada,
/// correlação e o cabeçalho operacional quando habilitado.
#[derive(Debug, Clone)]
pub struct SolicitacaoSerproHttp {
    pub endpoint: EndpointSerpro,
    pub servico: ServicoDividaSerpro,
    /// Somente dígitos (CPF/CNPJ ou número de inscrição).
    pub chave_normalizada: String,
    /// ID de correlação: vai apenas em `X-Request-Id` e no
    /// registro redigido — nunca a chave de consulta cru.
    pub request_id: Uuid,
    /// CPF do usuário (11 dígitos) quando `SERPRO_OPERATOR_HEADER_ENABLED`
    /// estiver habilitado e a ficha do usuário tiver CPF cadastrado.
    pub cpf_usuario: Option<String>,
}

/// Resultado do HTTP: corpo como `Value` (a interpretação do
/// contrato vive na aplicação) ou erro com o status upstream
/// observado, para a trilha de auditoria.
pub type RespostaHttpSerpro = Result<serde_json::Value, ErroHttpSerpro>;

#[derive(Debug, Clone)]
pub struct ErroHttpSerpro {
    pub erro: crate::domain::divida_ativa::erros::ErroConsultaDivida,
    /// Status HTTP reportado pelo gateway, quando houve resposta.
    pub status_upstream: Option<i32>,
}

/// Transporte HTTP da consulta (fase 4): um `GET` por solicitação,
/// com Authorization/x-cpf-usuario/x-request-id e mapeamento de
/// status → estados (SPEC §3.3/§4.4).
#[async_trait]
pub trait ConsultaDividaAtivaHttp: Send + Sync {
    async fn consultar(&self, solicitacao: SolicitacaoSerproHttp) -> RespostaHttpSerpro;
}

/// Catálogo de clientes por documento (o vínculo do histórico).
pub trait CatalogoClientesPorDocumento: Send + Sync {
    fn buscar_por_documento(
        &self,
        escritorio_id: Uuid,
        documento_digitos: &str,
    ) -> Result<Option<Uuid>, RepositorioError>;
}

/// CPF nacional do operador (coluna `usuarios.cpf`): alimenta
/// `x-cpf-usuario` apenas quando habilitado por configuração.
pub trait CatalogoOperadores: Send + Sync {
    fn buscar_cpf(&self, usuario_id: Uuid) -> Result<Option<String>, RepositorioError>;
}

/// Registro único na trilha `consultas_fiscais` (um por tentativa
/// HTTP — SPEC §4.4 passo 8 e fallback D2).
#[derive(Debug, Clone)]
pub struct RegistroConsultaFiscal {
    pub escritorio_id: Uuid,
    pub cliente_id: Option<Uuid>,
    pub produto: &'static str,
    pub servico: &'static str,
    pub verbo: Option<&'static str>,
    pub ambiente: &'static str,
    pub chave_consulta_hash: Option<String>,
    pub chave_consulta_mascarada: Option<String>,
    pub fonte_consulta: Option<&'static str>,
    pub status_code: Option<i32>,
    pub estado: &'static str,
    pub protocolo: Option<String>,
    pub request_redigido: Option<serde_json::Value>,
    pub response_redigido: Option<serde_json::Value>,
}

pub trait RepositorioConsultasFiscais: Send + Sync {
    fn registrar(&self, registro: RegistroConsultaFiscal) -> Result<Uuid, RepositorioError>;
    /// Apaga registros anteriores ao limite de retenção (SPEC §9.11).
    fn expurgar_antes_de(
        &self,
        limite: chrono::DateTime<chrono::Utc>,
    ) -> Result<u64, RepositorioError>;
}

/// Fallback D2: resolve o "Código de Contribuinte" alternativo do
/// cliente, quando existir catálogo interno. `None` = sem chave
/// alternativa; o uso exige a flag de configuração habilitada.
#[async_trait]
pub trait ResolvedorChaveAlternativa: Send + Sync {
    async fn resolver(
        &self,
        escritorio_id: Uuid,
        cliente_id: Option<Uuid>,
        cpf_cnpj_digitos: &str,
    ) -> Result<Option<String>, RepositorioError>;
}

/// Rate limit local por escritório (SPEC §3.3): falha rápida antes
/// de qualquer chamada upstream — tentativas bloqueadas não geram
/// registro de auditoria (não há chamada externa).
#[allow(clippy::result_unit_err)]
pub trait LimitadorConsultas: Send + Sync {
    fn adquirir(&self, escritorio_id: Uuid) -> Result<(), ()>;
}
