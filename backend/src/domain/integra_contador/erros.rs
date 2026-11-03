//! Erros do domínio Integra Contador.
//!
//! `ErroAssinatura` vive exclusivamente em `assinatura.rs`; aqui ele é
//! apenas encapsulado. Erros de catálogo (SPEC §5.4) vêm encapsulados de
//! `servicos.rs`. Fail-closed em toda a árvore.

use thiserror::Error;

use crate::domain::integra_contador::servicos::ErroCatalogo;

/// Erros de fluxo de procuração e-CAC (SPEC §6).
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ErroProcuracao {
    #[error("procuração não se aplica ao serviço {0}")]
    NaoSeAplica(String),

    #[error("procuração vencida em {0}")]
    Vencida(String),

    #[error("procuração cancelada pelo outorgante")]
    Cancelada,

    #[error(
        "nenhum dos códigos exigidos ({codigo}) cobre a procuração entre {contribuinte} e {procurador}"
    )]
    Incompativel {
        contribuinte: String,
        procurador: String,
        codigo: String,
    },

    #[error("procuração não verificável: {motivo}")]
    NaoVerificavel { motivo: String },

    #[error("código de serviço não permitido para procuração: {0}")]
    CodigoNaoPermitido(String),
}

/// Erros do domínio Integra Contador — classificados por fase do fluxo
/// (catálogo → gates de idempotência → procuração → transporte →
/// resposta), para mapeamento estável de códigos HTTP pela presentation.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ErroIntegraContador {
    // ===== Catálogo / allowlist (SPEC §5.4, D4, P4) =====
    #[error("serviço inválido no catálogo: {0}")]
    Catalogo(#[from] ErroCatalogo),

    // ===== Envelope / payload (SPEC §5.2) =====
    #[error("payload inválido: {motivo}")]
    PayloadInvalido { motivo: String },

    // ===== Procuração (SPEC §6) =====
    #[error("procuração necessária: autor {autor} difere do contribuinte {contribuinte}")]
    ProcuracaoNecessaria { autor: String, contribuinte: String },

    #[error("procuração: {0}")]
    Procuracao(#[from] ErroProcuracao),

    // ===== Assinatura (SPEC §7.1) =====
    #[error("assinatura eletrônica: {0}")]
    Assinatura(#[from] crate::domain::integra_contador::assinatura::ErroAssinatura),

    // ===== Tokens / autenticação =====
    #[error("token de acesso expirado ou inválido")]
    TokenExpirado,

    #[error("autenticação SAPI falhou: {motivo}")]
    AutenticacaoSapiFalhou { motivo: String },

    // ===== Transporte =====
    #[error("requisição falhou: HTTP {status} — {mensagem}")]
    RequisicaoFalhou { status: u16, mensagem: String },

    #[error("resposta inválida da SERPRO: {motivo}")]
    RespostaInvalida { motivo: String },

    // ===== Semânticas de transporte da SPEC §7 =====
    #[error(
        "estado indeterminado (HTTP 504): a operação pode ou não ter sido aplicada; \
         não reenviar automaticamente"
    )]
    EstadoIndeterminado,

    #[error("processamento assíncrono (HTTP 202): consultar após {tempo_espera}s")]
    ProcessamentoAssincrono { tempo_espera: u64 },

    #[error("processamento não concluído (HTTP 204): reconsultar com eTag")]
    ProcessamentoNaoConcluido,

    #[error("token do procurador ainda válido (HTTP 304): reutilizar token em cache")]
    TokenProcuradorValido,

    // ===== Erros SERPRO =====
    #[error("acesso negado pela SERPRO: {motivo}")]
    AcessoNegadoSerpro { motivo: String },

    #[error("limite de requisições excedido (HTTP 429): tentar novamente em {retry_after}s")]
    RateLimitExcedido { retry_after: u64 },

    #[error("serviço indisponível (HTTP 503): {motivo}")]
    ServicoIndisponivel { motivo: String },

    // ===== Idempotência (verbos Declarar/Emitir, SPEC §5.4) =====
    #[error("operação mutável requer confirmação explícita")]
    ConfirmacaoExplicitaNecessaria,

    #[error("operação mutável requer chave de idempotência")]
    ChaveIdempotenciaNecessaria,

    #[error("chave de idempotência reutilizada com payload diferente")]
    IdempotenciaConflito,

    // ===== Interno =====
    #[error("erro interno: {0}")]
    ErroInterno(String),
}

/// Mesma chave de idempotência com hash de payload diferente
/// ⇒ jamais reaproveitar o resultado anterior.
pub fn idempotencia_e_conflito(hash_antes: &str, hash_agora: &str) -> bool {
    !hash_antes.eq_ignore_ascii_case(hash_agora)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::integra_contador::assinatura::ErroAssinatura;
    use crate::domain::integra_contador::servicos::ErroCatalogo;

    #[test]
    fn catalogo_encapsulado_display() {
        let erro = ErroIntegraContador::from(ErroCatalogo::AguardandoDecisao(
            "PGDASD/CONSDECLARACAO13".to_owned(),
        ));
        assert!(erro.to_string().contains("AguardandoDecisao"));
    }

    #[test]
    fn procuracao_necessaria_display() {
        let erro = ErroIntegraContador::ProcuracaoNecessaria {
            autor: "12345678000190".to_owned(),
            contribuinte: "98765432000190".to_owned(),
        };
        assert!(erro.to_string().contains("procuração necessária"));
    }

    #[test]
    fn estado_indeterminado_menciona_504() {
        assert!(
            ErroIntegraContador::EstadoIndeterminado
                .to_string()
                .contains("504")
        );
    }

    #[test]
    fn assinatura_encapsulada_de_assinatura_rs() {
        let erro = ErroIntegraContador::from(ErroAssinatura::CertificadoNaoIcpBrasil);
        assert!(erro.to_string().contains("assinatura eletrônica"));
        assert!(erro.to_string().contains("ICP-Brasil"));
    }

    #[test]
    fn conflito_idempotencia_detectado() {
        assert!(idempotencia_e_conflito("abc", "abd"));
        assert!(!idempotencia_e_conflito("ABC", "abc"));
    }
}
