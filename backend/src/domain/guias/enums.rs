//! Tipos de guia e status de emissão.

/// Tipo de guia fiscal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TipoGuia {
    Das,
    Darf,
}

impl TipoGuia {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Das => "DAS",
            Self::Darf => "DARF",
        }
    }
}

/// Status de uma emissão de guia.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusGuia {
    Solicitada,
    Emitida,
    Baixada,
    Falhou,
}

impl StatusGuia {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Solicitada => "solicitada",
            Self::Emitida => "emitida",
            Self::Baixada => "baixada",
            Self::Falhou => "falhou",
        }
    }
}
