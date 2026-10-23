use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
pub struct CriarCertificadoRequest {
    pub tipo: String,
    pub titular_cpf_cnpj: String,
    pub conteudo_base64: String,
    pub senha: Option<String>,
    pub algoritmo: Option<String>,
    pub fingerprint: String,
    pub valido_de: chrono::DateTime<chrono::Utc>,
    pub valido_ate: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CertificadoResponse {
    pub id: Uuid,
    pub escritorio_id: Uuid,
    pub tipo: String,
    pub titular_cpf_cnpj: String,
    pub algoritmo: String,
    pub fingerprint: String,
    pub valido_de: chrono::DateTime<chrono::Utc>,
    pub valido_ate: chrono::DateTime<chrono::Utc>,
    pub ativo: bool,
    pub criado_em: chrono::DateTime<chrono::Utc>,
    pub dias_para_expirar: i64,
}

impl From<crate::application::certificados::CertificadoPublico> for CertificadoResponse {
    fn from(c: crate::application::certificados::CertificadoPublico) -> Self {
        Self {
            id: c.id,
            escritorio_id: c.escritorio_id,
            tipo: c.tipo.como_str().to_string(),
            titular_cpf_cnpj: c.titular_cpf_cnpj,
            algoritmo: c.algoritmo,
            fingerprint: c.fingerprint,
            valido_de: c.valido_de,
            valido_ate: c.valido_ate,
            ativo: c.ativo,
            criado_em: c.criado_em,
            dias_para_expirar: c.dias_para_expirar,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ListagemCertificadosResponse {
    pub itens: Vec<CertificadoResponse>,
    pub total: i64,
    pub pagina: i64,
    pub tamanho: i64,
}

impl From<crate::application::certificados::ListagemCertificados> for ListagemCertificadosResponse {
    fn from(l: crate::application::certificados::ListagemCertificados) -> Self {
        Self {
            itens: l.itens.into_iter().map(Into::into).collect(),
            total: l.total,
            pagina: l.pagina,
            tamanho: l.tamanho,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListarCertificadosQuery {
    pub apenas_validos: Option<bool>,
    pub pagina: Option<i64>,
    pub tamanho: Option<i64>,
}

impl ListarCertificadosQuery {
    pub fn para_comando(&self) -> Result<(bool, i64, i64), crate::presentation::error::ApiError> {
        let apenas_validos = self.apenas_validos.unwrap_or(true);
        let pagina = self.pagina.unwrap_or(1).max(1);
        let tamanho = self.tamanho.unwrap_or(20).clamp(1, 100);
        Ok((apenas_validos, pagina, tamanho))
    }
}
