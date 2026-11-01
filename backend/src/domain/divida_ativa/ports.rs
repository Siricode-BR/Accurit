use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::divida_ativa::{ResultadoConsultaDevedor, InscricaoDetalhe, RegistroConsulta, FonteConsulta, ErroDividaAtiva};
use crate::domain::clientes::CpfCnpj;
use crate::application::serpro::ports::RepositorioError;

#[async_trait]
pub trait ClienteDividaAtiva: Send + Sync {
    async fn consultar_devedor(&self, chave: &CpfCnpj, cpf_operador: Option<&str>) -> Result<ResultadoConsultaDevedor, ErroDividaAtiva>;
    async fn consultar_inscricao(&self, numero_inscricao: &str, cpf_operador: Option<&str>) -> Result<InscricaoDetalhe, ErroDividaAtiva>;
}

pub trait ResolvedorChaveAlternativa: Send + Sync {
    fn resolver(&self, chave: &CpfCnpj) -> Option<String>;
}

pub trait ConsultasFiscaisRepositorio: Send + Sync {
    fn registrar_consulta(&self, consulta: &RegistroConsulta) -> Result<(), RepositorioError>;
}

pub struct RegistroConsulta {
    pub escritorio_id: Uuid,
    pub cliente_id: Option<Uuid>,
    pub produto: String,
    pub servico: String,
    pub verbo: String,
    pub ambiente: String,
    pub chave_hash: String,
    pub chave_mascarada: String,
    pub fonte: FonteConsulta,
    pub status_code: Option<i32>,
    pub estado: String,
    pub protocolo: Option<String>,
}
