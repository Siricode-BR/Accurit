//! Portas da aplicação Integra Contador.

use async_trait::async_trait;
use uuid::Uuid;

use crate::application::serpro::ports::RepositorioError;
use crate::domain::integra_contador::envelope::{
    Identificacao, IntegraContadorRequest, IntegraContadorResponse, VerboIntegraContador,
};
use crate::domain::integra_contador::erros::{ErroIntegraContador, ErroProcuracao};
use crate::domain::integra_contador::procuração::ValidacaoProcuracao;
use crate::domain::integra_contador::assinatura::XmlAssinado;
use crate::domain::serpro::ambiente::AmbienteSerpro;

/// Cliente HTTP para chamadas ao Integra Contador.
#[async_trait]
pub trait ClienteIntegraContador: Send + Sync {
    async fn executar(
        &self,
        request: &IntegraContadorRequest,
        verbo: VerboIntegraContador,
        ambiente: AmbienteSerpro,
        request_id: &str,
    ) -> Result<IntegraContadorResponse, ErroIntegraContador>;
}

/// Validador de procuração via OBTERPROCURACAO41.
#[async_trait]
pub trait ValidadorProcuracao: Send + Sync {
    async fn validar(
        &self,
        contribuinte: &Identificacao,
        procurador: &Identificacao,
        codigo_servico: &str,
        ambiente: AmbienteSerpro,
        request_id: &str,
    ) -> Result<ValidacaoProcuracao, ErroProcuracao>;
}

/// Gerenciador de tokens SERPRO (OAuth2 + SAPI + autenticar_procurador).
#[async_trait]
pub trait GerenciadorTokensSerpro: Send + Sync {
    async fn obter_access_token(
        &self,
        escritorio_id: Uuid,
        produto: &str,
        ambiente: AmbienteSerpro,
    ) -> Result<String, ErroIntegraContador>;

    async fn obter_jwt_token(
        &self,
        escritorio_id: Uuid,
        produto: &str,
        ambiente: AmbienteSerpro,
    ) -> Result<String, ErroIntegraContador>;

    async fn obter_autenticar_procurador_token(
        &self,
        escritorio_id: Uuid,
        termo_assinado: &XmlAssinado,
        ambiente: AmbienteSerpro,
        request_id: &str,
    ) -> Result<String, ErroIntegraContador>;

    async fn invalidar_cache(&self, escritorio_id: Uuid, ambiente: AmbienteSerpro) -> Result<(), RepositorioError>;
}

/// Assinador de XML para Termo de Autorização.
#[async_trait]
pub trait AssinadorTermoAutorizacao: Send + Sync {
    async fn assinar(
        &self,
        termo: &crate::domain::integra_contador::assinatura::TermoAutorizacao,
        escritorio_id: Uuid,
    ) -> Result<XmlAssinado, ErroIntegraContador>;
}

/// Repositório de idempotência para operações mutáveis.
#[async_trait]
pub trait RepositorioIdempotencia: Send + Sync {
    /// Verifica se a chave já foi processada.
    /// Retorna Some(resposta) se já processada, None se não.
    async fn verificar(
        &self,
        escritorio_id: Uuid,
        chave: &str,
    ) -> Result<Option<IntegraContadorResponse>, RepositorioError>;

    /// Registra uma nova chave de idempotência.
    async fn registrar(
        &self,
        escritorio_id: Uuid,
        chave: &str,
        verbo: VerboIntegraContador,
        id_sistema: &str,
        id_servico: &str,
        ambiente: AmbienteSerpro,
        payload_hash: &str,
    ) -> Result<(), RepositorioError>;

    /// Atualiza o resultado de uma operação processada.
    async fn finalizar(
        &self,
        escritorio_id: Uuid,
        chave: &str,
        resposta: &IntegraContadorResponse,
        status_code: Option<u16>,
    ) -> Result<(), RepositorioError>;
}

/// Repositório de auditoria do Integra Contador.
#[async_trait]
pub trait RepositorioAuditoriaIntegraContador: Send + Sync {
    async fn registrar(
        &self,
        escritorio_id: Uuid,
        cliente_id: Option<Uuid>,
        verbo: VerboIntegraContador,
        id_sistema: &str,
        id_servico: &str,
        versao_sistema: &str,
        ambiente: AmbienteSerpro,
        autor_pedido_dados: &str,
        contribuinte_cpf_cnpj: &str,
        status_code: Option<u16>,
        estado: &str,
        protocolo: Option<String>,
        request_redigido: serde_json::Value,
        response_redigido: Option<serde_json::Value>,
        erro_codigo: Option<String>,
        erro_mensagem: Option<String>,
    ) -> Result<(), RepositorioError>;
}

/// Resolvedor de credencial do escritório para Integra Contador.
#[async_trait]
pub trait CredencialIntegraContadorProvider: Send + Sync {
    async fn resolver(
        &self,
        escritorio_id: Uuid,
        ambiente: AmbienteSerpro,
    ) -> Result<CredencialIntegraContador, ErroIntegraContador>;
}

#[derive(Debug, Clone)]
pub struct CredencialIntegraContador {
    pub escritorio_id: Uuid,
    pub consumer_key: String,
    pub consumer_secret: String,
    pub certificado_pfx: Option<Vec<u8>>,
    pub certificado_senha: Option<String>,
    pub certificado_fingerprint: Option<String>,
}

/// CPF do operador autenticado (para header x-cpf-usuario).
#[async_trait]
pub trait CatalogoOperadoresIntegra: Send + Sync {
    async fn buscar_cpf(&self, usuario_id: Uuid) -> Result<Option<String>, RepositorioError>;
}

/// Catálogo de clientes por documento (para vincular auditoria).
#[async_trait]
pub trait CatalogoClientesPorDocumentoIntegra: Send + Sync {
    async fn buscar_por_documento(
        &self,
        escritorio_id: Uuid,
        documento_digitos: &str,
    ) -> Result<Option<Uuid>, RepositorioError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credencial_integra_contador_cria() {
        let cred = CredencialIntegraContador {
            escritorio_id: Uuid::nil(),
            consumer_key: "ck".to_owned(),
            consumer_secret: "cs".to_owned(),
            certificado_pfx: None,
            certificado_senha: None,
            certificado_fingerprint: None,
        };
        assert_eq!(cred.consumer_key, "ck");
    }
}