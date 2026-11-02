//! Erros da integração com a Dívida Ativa (SPEC §10.3). Cada variante
//! tem um código estável para o envelope da API — nunca são expostos
//! detalhes do documento nem do token.

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ErroDividaAtiva {
    #[error("chave de consulta inválida")]
    ChaveInvalida,
    #[error("limite local de consultas excedido para este escritório")]
    LimiteLocalAtingido,
    #[error("consultas em excesso na SERPRO após tentativas e espera")]
    LimiteSerproAtingido,
    #[error("SERPRO trial não configurado")]
    TrialNaoConfigurado,
    #[error("SERPRO produção não configurado")]
    ProducaoNaoConfigurada,
    #[error("autenticação na SERPRO falhou")]
    NaoAutorizado,
    #[error("acesso negado pelo proxy SERPRO")]
    ProxyNegado,
    #[error("serviço SERPRO indisponível após tentativas")]
    ServicoIndisponivel,
    #[error("resultado da consulta é indeterminado após HTTP 504")]
    Indeterminado,
    #[error("resposta da SERPRO fora do contrato")]
    RespostaInvalida,
    #[error("resposta da SERPRO fora do contrato: {0}")]
    RespostaInvalidaDetalhada(String),
    #[error("inscrição não encontrada")]
    NaoEncontrado,
    #[error("falha interna da integração: {0}")]
    ErroInterno(String),
}

pub type ErroConsultaDivida = ErroDividaAtiva;

impl ErroDividaAtiva {
    /// Código estável para o envelope (SPEC §10.3).
    pub const fn codigo(&self) -> &'static str {
        match self {
            Self::ChaveInvalida => "VALIDACAO",
            Self::LimiteLocalAtingido => "MUITAS_REQUISICOES",
            Self::LimiteSerproAtingido => "SERPRO_RATE_LIMITED",
            Self::TrialNaoConfigurado => "SERPRO_TRIAL_NOT_CONFIGURED",
            Self::ProducaoNaoConfigurada => "SERPRO_PROD_NOT_CONFIGURED",
            Self::NaoAutorizado => "SERPRO_UNAUTHORIZED",
            Self::ProxyNegado => "SERPRO_PROXY_DENIED",
            Self::ServicoIndisponivel => "SERPRO_UNAVAILABLE",
            Self::Indeterminado => "SERPRO_INDETERMINATE",
            Self::RespostaInvalida => "SERPRO_BAD_RESPONSE",
            Self::RespostaInvalidaDetalhada(_) => "SERPRO_BAD_RESPONSE",
            Self::NaoEncontrado => "SERPRO_NOT_FOUND",
            Self::ErroInterno(_) => "ERRO_INTERNO",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codigos_estaveis_batem_com_o_envelope() {
        assert_eq!(
            ErroDividaAtiva::TrialNaoConfigurado.codigo(),
            "SERPRO_TRIAL_NOT_CONFIGURED"
        );
        assert_eq!(
            ErroDividaAtiva::ProducaoNaoConfigurada.codigo(),
            "SERPRO_PROD_NOT_CONFIGURED"
        );
        assert_eq!(
            ErroDividaAtiva::NaoAutorizado.codigo(),
            "SERPRO_UNAUTHORIZED"
        );
        assert_eq!(ErroDividaAtiva::ProxyNegado.codigo(), "SERPRO_PROXY_DENIED");
        assert_eq!(
            ErroDividaAtiva::ServicoIndisponivel.codigo(),
            "SERPRO_UNAVAILABLE"
        );
        assert_eq!(
            ErroDividaAtiva::RespostaInvalida.codigo(),
            "SERPRO_BAD_RESPONSE"
        );
        assert_eq!(
            ErroDividaAtiva::LimiteSerproAtingido.codigo(),
            "SERPRO_RATE_LIMITED"
        );
        assert_eq!(ErroDividaAtiva::NaoEncontrado.codigo(), "SERPRO_NOT_FOUND");
    }
}
