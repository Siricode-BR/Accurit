//! Erros do domínio Integra Contador.

use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ErroIntegraContador {
    #[error("configuração de endpoint ausente para {id_sistema}/{id_servico}")]
    EndpointNaoConfigurado { id_sistema: String, id_servico: String },

    #[error("serviço não suportado: {id_sistema}/{id_servico}")]
    ServicoNaoSuportado { id_sistema: String, id_servico: String },

    #[error("versão de sistema inválida: {versao} para {id_sistema}/{id_servico}")]
    VersaoInvalida { versao: String, id_sistema: String, id_servico: String },

    #[error("payload inválido para {id_sistema}/{id_servico}: {motivo}")]
    PayloadInvalido { id_sistema: String, id_servico: String, motivo: String },

    #[error("procuração necessária: autor {autor} difere do contribuinte {contribuinte} para serviço {id_servico}")]
    ProcuracaoNecessaria { autor: String, contribuinte: String, id_servico: String },

    #[error("procuração inválida ou vencida: {motivo}")]
    ProcuracaoInvalida { motivo: String },

    #[error("procuração não encontrada para contribuinte {contribuinte}, procurador {procurador}, serviço {codigo_servico}")]
    ProcuracaoNaoEncontrada { contribuinte: String, procurador: String, codigo_servico: String },

    #[error("token de procurador ausente ou expirado")]
    TokenProcuradorAusente,

    #[error("assinatura XML inválida: {motivo}")]
    AssinaturaInvalida { motivo: String },

    #[error("certificado A1 não encontrado, expirado ou revogado")]
    CertificadoInvalido,

    #[error("falha na autenticação SAPI: {motivo}")]
    AutenticacaoSapiFalhou { motivo: String },

    #[error("token OAuth2 expirado ou inválido")]
    TokenExpirado,

    #[error("requisição HTTP falhou: {status} - {mensagem}")]
    RequisicaoFalhou { status: u16, mensagem: String },

    #[error("resposta inválida da SERPRO: {motivo}")]
    RespostaInvalida { motivo: String },

    #[error("estado indeterminado (HTTP 504): não reenviar automaticamente")]
    EstadoIndeterminado,

    #[error("processamento assíncrono (HTTP 202): aguardar {tempo_espera}s antes de consultar")]
    ProcessamentoAssincrono { tempo_espera: u32 },

    #[error("processamento não concluído (HTTP 204): aguardar tempo estimado no eTag")]
    ProcessamentoNaoConcluido,

    #[error("token de procurador ainda válido (HTTP 304): reutilizar token em cache")]
    TokenProcuradorValido,

    #[error("acesso negado pela SERPRO (403/ICGERENCIADOR-022): {motivo}")]
    AcessoNegadoSerpro { motivo: String },

    #[error("rate limit excedido (HTTP 429): aguardar {retry_after}s")]
    RateLimitExcedido { retry_after: u64 },

    #[error("serviço indisponível (HTTP 503): {motivo}")]
    ServicoIndisponivel { motivo: String },

    #[error("erro interno: {0}")]
    ErroInterno(String),

    #[error("operação mutável requer confirmação explícita e idempotência")]
    RequerConfirmacaoEIdempotencia,

    #[error("chave de idempotência já processada: {chave}")]
    IdempotenciaJaProcessada { chave: String },

    #[error("auditoria falhou: {0}")]
    AuditoriaFalhou(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ErroProcuracao {
    #[error("procuração não se aplica ao serviço {codigo_servico}")]
    NaoSeAplica { codigo_servico: String },

    #[error("procuração vencida em {valido_ate}")]
    Vencida { valido_ate: String },

    #[error("procuração cancelada")]
    Cancelada,

    #[error("procuração incompatível: contribuinte {contribuinte}, procurador {procurador}, serviço {codigo_servico}")]
    Incompativel { contribuinte: String, procurador: String, codigo_servico: String },

    #[error("procuração não verificável: {motivo}")]
    NaoVerificavel { motivo: String },

    #[error("código de serviço não permitido para procuração: {codigo_servico}")]
    CodigoNaoPermitido { codigo_servico: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ErroAssinatura {
    #[error("certificado não é ICP-Brasil série A (A1/A3)")]
    CertificadoNaoIcpBrasil,

    #[error("certificado expirado em {valido_ate}")]
    CertificadoExpirado { valido_ate: String },

    #[error("certificado revogado")]
    CertificadoRevogado,

    #[error("cadeia de certificação inválida: {motivo}")]
    CadeiaInvalida { motivo: String },

    #[error("digest SHA-256 divergente")]
    DigestDivergente,

    #[error("assinatura RSA-SHA256 inválida")]
    AssinaturaInvalida,

    #[error("canonicalização C14N falhou: {motivo}")]
    CanonicalizacaoFalhou { motivo: String },

    #[error("transformação Enveloped Signature falhou: {motivo}")]
    TransformacaoFalhou { motivo: String },

    #[error("XML malformado: {motivo}")]
    XmlMalformado { motivo: String },

    #[error("texto do termo diverge do oficial: {campo}")]
    TermoDivergente { campo: String },

    #[error("chave privada não disponível para assinatura")]
    ChavePrivadaIndisponivel,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn erro_procuracao_necessaria_display() {
        let err = ErroIntegraContador::ProcuracaoNecessaria {
            autor: "12345678000190".to_owned(),
            contribuinte: "98765432000190".to_owned(),
            id_servico: "TRANSDECLARACAO11".to_owned(),
        };
        assert!(err.to_string().contains("procuração necessária"));
    }

    #[test]
    fn erro_estado_indeterminado_codigo() {
        let err = ErroIntegraContador::EstadoIndeterminado;
        assert!(err.to_string().contains("504"));
    }

    #[test]
    fn erro_assinatura_certificado_display() {
        let err = ErroAssinatura::CertificadoNaoIcpBrasil;
        assert!(err.to_string().contains("ICP-Brasil"));
    }
}