//! Fase F — Enums do domínio de relatórios.

/// Tipo de relatório que pode ser gerado.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TipoRelatorio {
    SituacaoFiscal,
    HistoricoConsultas,
    PendenciasCarteira,
}

impl TipoRelatorio {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SituacaoFiscal => "situacao_fiscal",
            Self::HistoricoConsultas => "historico_consultas",
            Self::PendenciasCarteira => "pendencias_carteira",
        }
    }
}

/// Formato de exportação do relatório.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatoRelatorio {
    Pdf,
    Xlsx,
}

impl FormatoRelatorio {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pdf => "PDF",
            Self::Xlsx => "XLSX",
        }
    }
}

/// Status de processamento de um relatório.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusRelatorio {
    Gerando,
    Concluido,
    Falhou,
}

impl StatusRelatorio {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Gerando => "gerando",
            Self::Concluido => "concluido",
            Self::Falhou => "falhou",
        }
    }
}
