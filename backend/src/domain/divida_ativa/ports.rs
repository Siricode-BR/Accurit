//! Portas da Dívida Ativa. Todas definidas no domínio e implementadas
//! pela infrastructure (HTTP SERPRO, Diesel, rate limit): o domínio
//! nunca importa Axum/Diesel/reqwest.

use async_trait::async_trait;
use uuid::Uuid;

use crate::application::serpro::ports::RepositorioError;
use crate::domain::clientes::CpfCnpj;
use crate::domain::divida_ativa::devedor::{
    InscricaoDetalhe, RegistroConsultaFiscal, ResultadoConsultaDevedor,
};
use crate::domain::divida_ativa::erros::ErroDividaAtiva;
use crate::domain::serpro::endpoints::EndpointSerpro;

/// Cliente da SERPRO: faz as chamadas `GET` para o endpoint já resolvido
/// (URL base + estratégia de autenticação) e devolve VOs de domínio —
/// nunca o JSON bruto.
#[async_trait]
pub trait ClienteDividaAtiva: Send + Sync {
    async fn consultar_devedor(
        &self,
        endpoint: &EndpointSerpro,
        chave: &CpfCnpj,
        cpf_operador: Option<String>,
        request_id: &str,
    ) -> Result<ResultadoConsultaDevedor, ErroDividaAtiva>;

    async fn consultar_inscricao(
        &self,
        endpoint: &EndpointSerpro,
        numero_inscricao: &str,
        cpf_operador: Option<String>,
        request_id: &str,
    ) -> Result<InscricaoDetalhe, ErroDividaAtiva>;
}

/// Limitador local de consultas por escritório (SPEC §3.3).
pub trait LimitadorConsultas: Send + Sync {
    fn permitir(&self, escritorio_id: Uuid) -> Result<(), ErroDividaAtiva>;
}

/// Persistência do histórico `consultas_fiscais` (SPEC §9.11).
pub trait RepositorioConsultasFiscais: Send + Sync {
    fn registrar(&self, registro: &RegistroConsultaFiscal) -> Result<Uuid, RepositorioError>;
    fn expurgar_antes_de(
        &self,
        limite: chrono::DateTime<chrono::Utc>,
    ) -> Result<u64, RepositorioError>;
}

/// Vínculo do documento consultado a um cliente do tenant (auditoria e
/// IDOR: só clientes do próprio escritório são referenciados).
pub trait CatalogoClientesPorDocumento: Send + Sync {
    fn buscar_por_documento(
        &self,
        escritorio_id: Uuid,
        documento_digitos: &str,
    ) -> Result<Option<Uuid>, RepositorioError>;
}

/// CPF do operador autenticado (`usuarios.cpf`), usado no
/// `x-cpf-usuario` quando D3 está habilitado via flag.
pub trait CatalogoOperadores: Send + Sync {
    fn buscar_cpf(&self, usuario_id: Uuid) -> Result<Option<String>, RepositorioError>;
}

/// Resolvedor da chave alternativa D2 (Código de Contribuinte). Sem
/// contrato real fica sem fio: devolve `None` até o teste live definir
/// o caminho (200 vazio ou 404) — DECISOES-PENDENTES D2.
pub trait ResolvedorChaveAlternativa: Send + Sync {
    fn resolver(&self, chave: &CpfCnpj) -> Option<String>;
}

/// Hash SHA-256 com pepper de config para `chave_consulta_hash` — o
/// documento em claro **nunca** vai para o banco ou para logs.
pub trait HashDeChaveConsulta: Send + Sync {
    /// `None` quando o pepper não está configurado (a auditoria segue,
    /// mas sem hash — comportamento documentado em GO-LIVE-SERPRO).
    fn hash(&self, chave: &str) -> Option<String>;
}
