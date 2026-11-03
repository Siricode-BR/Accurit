use async_trait::async_trait;
use serde_json::Value;
use uuid::Uuid;

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

pub type RespostaRedigida = Value;
