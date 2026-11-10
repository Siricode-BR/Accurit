//! Tipos de notificação, canais e status.

/// Canal de envio da notificação.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanalNotificacao {
    InApp,
    Email,
    WhatsApp,
}

impl CanalNotificacao {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InApp => "in_app",
            Self::Email => "email",
            Self::WhatsApp => "whatsapp",
        }
    }
}

/// Tipo de gatilho de notificação.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TipoNotificacao {
    PendenciaDividaAtiva,
    MensagemCaixaPostal,
    ProcuracaoVencendo,
    CertificadoA1Vencendo,
    DasVencendo,
    FalhaRotina,
    LimiteConsumoSerpro,
}

impl TipoNotificacao {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PendenciaDividaAtiva => "pendencia_divida_ativa",
            Self::MensagemCaixaPostal => "mensagem_caixa_postal",
            Self::ProcuracaoVencendo => "procuracao_vencendo",
            Self::CertificadoA1Vencendo => "certificado_a1_vencendo",
            Self::DasVencendo => "das_vencendo",
            Self::FalhaRotina => "falha_rotina",
            Self::LimiteConsumoSerpro => "limite_consumo_serpro",
        }
    }

    pub fn todos() -> Vec<Self> {
        vec![
            Self::PendenciaDividaAtiva,
            Self::MensagemCaixaPostal,
            Self::ProcuracaoVencendo,
            Self::CertificadoA1Vencendo,
            Self::DasVencendo,
            Self::FalhaRotina,
            Self::LimiteConsumoSerpro,
        ]
    }
}

/// Status de leitura da notificação.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusNotificacao {
    NaoLida,
    Lida,
}

impl StatusNotificacao {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NaoLida => "nao_lida",
            Self::Lida => "lida",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tipos_notificacao_sao_estaveis() {
        for tipo in TipoNotificacao::todos() {
            let s = tipo.as_str();
            assert!(s.len() <= 32, "tipo {s:?} excede VARCHAR(32)");
        }
    }

    #[test]
    fn canais_notificacao_sao_estaveis() {
        for canal in [
            CanalNotificacao::InApp,
            CanalNotificacao::Email,
            CanalNotificacao::WhatsApp,
        ] {
            let s = canal.as_str();
            assert!(s.len() <= 16, "canal {s:?} excede VARCHAR(16)");
        }
    }
}
