//! Portas da aplicação Integra Contador (SPEC §5–§7).
//!
//! As portas de auditoria/limites (`RepositorioConsultasFiscais`,
//! `LimitadorConsultas`, `HashDeChaveConsulta`, `CatalogoOperadores`,
//! `CatalogoClientesPorDocumento`) são **reaproveitadas** do domínio
//! Fase 4 — a bilhetagem operacional do Integra Contador vai para
//! `consultas_fiscais` com `produto = "INTEGRA_CONTADOR"`.

use async_trait::async_trait;
use uuid::Uuid;

use crate::application::serpro::ports::RepositorioError;
use crate::domain::integra_contador::assinatura::TermoAutorizacao;
use crate::domain::integra_contador::assinatura::{ErroAssinatura, XmlAssinado};
use crate::domain::integra_contador::envelope::{
    Identificacao, IntegraContadorRequest, IntegraContadorResponse, VerboIntegraContador,
};
use crate::domain::integra_contador::erros::{ErroIntegraContador, ErroProcuracao};
use crate::domain::integra_contador::regras_procuracao::ValidacaoProcuracao;
use crate::domain::serpro::ambiente::AmbienteSerpro;

/// Entrega de uma chamada transportada ao Integra Contador.
///
/// `corpo` carrega o envelope da SPEC quando o HTTP devolveu corpo
/// (200/202/…). 204/304 não têm corpo; 202 traz `tempoEspera`; 429
/// `Retry-After`; 204 um eTag de continuidade.
pub struct EntregaIntegraContador {
    pub http_status: u16,
    pub corpo: Option<IntegraContadorResponse>,
    /// Tempo de espera em segundos para 202 (processamento assíncrono).
    pub tempo_espera: Option<u64>,
    /// `Retry-After` em segundos para 429.
    pub retry_after: Option<u64>,
    /// eTag para 204 (continuidade de processamento).
    pub etag: Option<String>,
}

/// Cliente de transporte do Integra Contador.
///
/// A autenticação (Bearer de trial ou token SAPI em produção, com
/// renovação em 401 no máximo uma vez) fica dentro da implementação da
/// infraestrutura. O `token_procurador` (quando há procurador atuando) é
/// enviado no header `autenticar-procurador-token` — nunca no corpo.
#[async_trait]
pub trait ClienteIntegraContador: Send + Sync {
    async fn executar(
        &self,
        escritorio_id: Uuid,
        verbo: VerboIntegraContador,
        request: &IntegraContadorRequest,
        token_procurador: Option<&str>,
        ambiente: AmbienteSerpro,
        request_id: &str,
    ) -> Result<EntregaIntegraContador, ErroIntegraContador>;
}

/// Verifica a procuração e-CAC do par outorgante/outorgado para o
/// serviço, via `OBTERPROCURACAO41` (SPEC §6). A escolha da regra
/// (`AnyOf`/`Condicional`) e a comparação do código são responsabilidade
/// do caso de uso — a porta só devolve o registro oficial.
#[async_trait]
pub trait ValidadorProcuracao: Send + Sync {
    async fn validar(
        &self,
        escritorio_id: Uuid,
        contribuinte: &Identificacao,
        procurador: &Identificacao,
        ambiente: AmbienteSerpro,
        request_id: &str,
    ) -> Result<ValidacaoProcuracao, ErroProcuracao>;
}

/// Autentica o procurador via `ENVIOXMLASSINADO81` (Apoiar): assina o
/// termo, obtém `autenticar_procurador_token` (36 bytes, válido até a
/// meia-noite seguinte em horário de Brasília) e o mantém em cache
/// particionado por (tenant, ambiente, outorgante, outorgado).
#[async_trait]
pub trait AutenticadorProcurador: Send + Sync {
    async fn obter_token(
        &self,
        escritorio_id: Uuid,
        contratante: &Identificacao,
        autor: &Identificacao,
        contribuinte: &Identificacao,
        ambiente: AmbienteSerpro,
        request_id: &str,
    ) -> Result<String, ErroIntegraContador>;
}

/// Assinador XMLDSig com certificado A1 do escritório (SPEC §7.1,
/// ADR-0004). A chave privada nunca sai da memória e é zerada após o uso.
#[async_trait]
pub trait AssinadorTermoAutorizacao: Send + Sync {
    async fn assinar(
        &self,
        escritorio_id: Uuid,
        termo: &TermoAutorizacao,
    ) -> Result<XmlAssinado, ErroAssinatura>;
}

/// Operação já registrada em `idempotencia_operacoes`.
#[derive(Debug, Clone)]
pub struct ReservaIdempotencia {
    pub payload_hash: String,
    /// Resultado pré-computado — `None` enquanto em execução ou quando o
    /// estado registrado é indeterminado (504: nunca reexecutar).
    pub resposta: Option<IntegraContadorResponse>,
    pub http_status: Option<u16>,
    pub estado: String,
    /// `true` apenas quando esta chamada criou a reserva atomicamente.
    /// Se outra requisição ganhou a corrida, ela deve reproduzir o estado
    /// persistido ou falhar fechada, nunca transmitir novamente.
    pub criada_agora: bool,
}

/// Idempotência de verbos mutáveis (Declarar/Emitir): mesma chave com o
/// mesmo payload ⇒ repete o resultado pré-computado; payload diferente ⇒
/// conflito. Escopo por (`escritorio_id`, `chave_idempotencia`).
#[async_trait]
pub trait RepositorioIdempotencia: Send + Sync {
    /// Leitura antecipada para detectar replay/conflito antes de gastar
    /// chamadas. `None` quando a chave é inédita no tenant.
    async fn verificar(
        &self,
        escritorio_id: Uuid,
        chave: &str,
    ) -> Result<Option<ReservaIdempotencia>, RepositorioError>;

    /// Reserva a chave idempotente (atomicamente; em corrida devolve a
    /// reserva existente). Escrita com hash do payload canônico.
    async fn reservar(
        &self,
        escritorio_id: Uuid,
        chave: &str,
        verbo: VerboIntegraContador,
        id_sistema: &str,
        id_servico: &str,
        versao_sistema: &str,
        ambiente: AmbienteSerpro,
        payload_hash: &str,
    ) -> Result<ReservaIdempotencia, RepositorioError>;

    /// Persiste o resultado pré-computado (inclusive o estado
    /// `INDETERMINADO`, com `resposta = None`).
    async fn finalizar(
        &self,
        escritorio_id: Uuid,
        chave: &str,
        estado: &str,
        resposta: Option<&IntegraContadorResponse>,
        http_status: Option<u16>,
    ) -> Result<(), RepositorioError>;
}

/// CNPJ do escritório, terreno do `contratante` do envelope (SPEC §5.2).
/// O `contratante` **nunca** vem do corpo da requisição — vem só do
/// cadastro do tenant (isolamento de tenant).
#[async_trait]
pub trait CatalogoEscritoriosIntegra: Send + Sync {
    async fn buscar_cnpj(&self, escritorio_id: Uuid) -> Result<Option<String>, RepositorioError>;
}

/// Complementos de auditoria tenant-isolada (Fase 4).
pub use crate::application::divida_ativa::ports as DividaAtivaReuse;

#[cfg(test)]
mod tests {
    use crate::domain::integra_contador::assinatura::TermoAutorizacao;

    #[test]
    fn termo_construido_para_assinador() {
        let termo = TermoAutorizacao::novo_oficial(
            "12345678000190",
            "Escritorio Teste",
            "12345678909",
            "Procurador Teste",
            "PF",
            "20261103",
            "20271103",
        );
        assert_eq!(termo.destinatario_numero, "12345678000190");
        assert_eq!(termo.assinado_por_papel, "autor pedido de dados");
    }
}
