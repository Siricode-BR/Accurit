use async_trait::async_trait;
use serde_json::Value;
use uuid::Uuid;

use crate::domain::certificados::Certificado;
use crate::domain::integra_contador::envelope::{
    Identificacao, IntegraContadorRequest, IntegraContadorResponse, VerboIntegraContador,
};
use crate::domain::integra_contador::erros::ErroIntegraContador;
use crate::domain::serpro::ambiente::AmbienteSerpro;

#[derive(Debug, Clone)]
pub struct ContextoIntegraContador {
    pub contratante: Identificacao,
    pub autor_pedido_dados: Identificacao,
    pub contribuinte: Identificacao,
    pub cpf_operador: Option<String>,
    /// Razão social do escritório (Procurador / autor do pedido de dados),
    /// exigida como `autorPedidoDados.nome` no Termo de Autorização.
    pub escritorio_nome: String,
}

/// Identidade da Software House contratante — a plataforma SaaS. Usada como
/// `contratante` no Termo de Autorização e no envelope do modelo C (Software
/// House), nunca como contratante de escritório. Resolvida da configuração
/// (`SERPRO_CONTRATANTE_CNPJ` / `SERPRO_CONTRATANTE_NOME`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContratantePlataforma {
    pub numero: String,
    pub nome: String,
}

#[async_trait]
pub trait CatalogoEscritoriosIntegra: Send + Sync {
    async fn resolver_contexto(
        &self,
        escritorio_id: Uuid,
        usuario_id: Uuid,
        documento_contribuinte: &str,
    ) -> Result<ContextoIntegraContador, ErroIntegraContador>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntregaIntegraContador {
    pub http_status: u16,
    pub estado: String,
    pub resposta: Option<IntegraContadorResponse>,
    pub referencia_resposta: Option<String>,
    pub tempo_espera_segundos: Option<u64>,
    pub etag: Option<String>,
    pub expires: Option<String>,
}

#[async_trait]
pub trait ClienteIntegraContador: Send + Sync {
    async fn executar(
        &self,
        verbo: VerboIntegraContador,
        request: &IntegraContadorRequest,
        ambiente: AmbienteSerpro,
        cpf_operador: Option<&str>,
        request_id: &str,
    ) -> Result<EntregaIntegraContador, ErroIntegraContador>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventoChamadaIntegra {
    pub escritorio_id: Uuid,
    pub usuario_id: Uuid,
    pub verbo: VerboIntegraContador,
    pub id_sistema: String,
    pub id_servico: String,
    pub ambiente: AmbienteSerpro,
    pub estado: String,
    pub status_code: Option<i32>,
    pub referencia_resposta: Option<String>,
    pub request_id: String,
    pub bilhetavel: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReservaIdempotencia {
    pub escritorio_id: Uuid,
    pub chave: String,
    pub verbo: VerboIntegraContador,
    pub id_sistema: String,
    pub id_servico: String,
    pub versao_sistema: String,
    pub ambiente: AmbienteSerpro,
    pub payload_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResultadoReservaIdempotencia {
    Nova,
    Concluida {
        status: Option<i32>,
        resposta_redigida: Option<Value>,
    },
}

#[async_trait]
pub trait RepositorioIdempotencia: Send + Sync {
    async fn reservar(
        &self,
        reserva: &ReservaIdempotencia,
    ) -> Result<ResultadoReservaIdempotencia, ErroIntegraContador>;

    async fn atualizar(
        &self,
        reserva: &ReservaIdempotencia,
        estado: &str,
        status: Option<i32>,
        resposta_redigida: Option<Value>,
    ) -> Result<(), ErroIntegraContador>;
}

#[async_trait]
pub trait AuditoriaIntegraContador: Send + Sync {
    async fn registrar_chamada(
        &self,
        evento: &EventoChamadaIntegra,
    ) -> Result<(), ErroIntegraContador>;

    async fn registrar_mutacao(
        &self,
        evento: &EventoAuditoriaIntegra,
    ) -> Result<(), ErroIntegraContador>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventoAuditoriaIntegra {
    pub escritorio_id: Uuid,
    pub usuario_id: Uuid,
    pub acao: &'static str,
    pub verbo: VerboIntegraContador,
    pub id_sistema: String,
    pub id_servico: String,
    pub ambiente: AmbienteSerpro,
    pub estado: String,
    pub payload_hash: String,
}

#[async_trait]
pub trait ValidadorProcuracao: Send + Sync {
    async fn validar(
        &self,
        contexto: &ContextoIntegraContador,
        codigo_servico: &str,
        ambiente: AmbienteSerpro,
    ) -> Result<bool, ErroIntegraContador>;
}

/// Port para obter o certificado A1 ativo do escritório.
#[async_trait]
pub trait CertificadoA1Provider: Send + Sync {
    async fn obter_ativo(
        &self,
        escritorio_id: Uuid,
    ) -> Result<Option<Certificado>, ErroIntegraContador>;
}

/// Resultado da autenticação do procurador (token JWT retornado pelo SERPRO).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutenticarProcuradorResultado {
    pub jwt_token: String,
    pub expira_em: chrono::DateTime<chrono::Utc>,
}

/// Port para salvar o token JWT obtido via AUTENTICAPROCURADOR.
#[async_trait]
pub trait TokensSerproRepositorio: Send + Sync {
    async fn salvar_jwt_procurador(
        &self,
        escritorio_id: Uuid,
        cliente_id: Uuid,
        ambiente: AmbienteSerpro,
        jwt_token: &str,
        expira_em: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), ErroIntegraContador>;
}

/// Port para o cliente HTTP do endpoint AUTENTICAPROCURADOR (ENVIOXMLASSINADO81).
#[async_trait]
pub trait AutenticaProcuradorCliente: Send + Sync {
    async fn enviar_xml_assinado(
        &self,
        request: &IntegraContadorRequest,
        xml_base64: &str,
        ambiente: AmbienteSerpro,
        request_id: &str,
    ) -> Result<AutenticarProcuradorResultado, ErroIntegraContador>;
}

pub type RespostaRedigida = Value;
