//! Tipos de trabalho, status de execução e status de lote.

/// Tipo de tarefa fiscal agendada.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TipoTrabalho {
    ConsultaDividaAtiva,
    ValidarProcuracao,
    ConsultaIntegraContador,
}

impl TipoTrabalho {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ConsultaDividaAtiva => "consulta_divida_ativa",
            Self::ValidarProcuracao => "validar_procuracao",
            Self::ConsultaIntegraContador => "consulta_integra_contador",
        }
    }
}

/// Status de uma execução de trabalho agendado.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusExecucao {
    Pendente,
    Executando,
    Concluido,
    Falhou,
    /// Execução interrompida por bloqueio (limite do plano, rate limit
    /// persistente). Explicitamente distinguida de "falhou" (Fase B.1).
    Bloqueado,
}

impl StatusExecucao {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pendente => "pendente",
            Self::Executando => "executando",
            Self::Concluido => "concluido",
            Self::Falhou => "falhou",
            Self::Bloqueado => "bloqueado",
        }
    }
}

/// Status de uma consulta em lote.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusLote {
    Pendente,
    Executando,
    Concluido,
    Falhou,
    Parcial,
    /// Lote encerrado sem execução: todos os itens bloqueados (limite do
    /// plano). Explicitamente distinguido de "falhou" (Fase B.1).
    Bloqueado,
}

impl StatusLote {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pendente => "pendente",
            Self::Executando => "executando",
            Self::Concluido => "concluido",
            Self::Falhou => "falhou",
            Self::Parcial => "parcial",
            Self::Bloqueado => "bloqueado",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tipos_trabalho_sao_estaveis_e_cabem_na_coluna() {
        for tipo in [
            TipoTrabalho::ConsultaDividaAtiva,
            TipoTrabalho::ValidarProcuracao,
            TipoTrabalho::ConsultaIntegraContador,
        ] {
            let s = tipo.as_str();
            assert!(s.len() <= 32, "tipo {s:?} excede VARCHAR(32)");
        }
    }

    #[test]
    fn status_execucao_sao_estaveis() {
        for status in [
            StatusExecucao::Pendente,
            StatusExecucao::Executando,
            StatusExecucao::Concluido,
            StatusExecucao::Falhou,
            StatusExecucao::Bloqueado,
        ] {
            let s = status.as_str();
            assert!(s.len() <= 16, "status {s:?} excede VARCHAR(16)");
        }
    }

    #[test]
    fn status_lote_sao_estaveis() {
        for status in [
            StatusLote::Pendente,
            StatusLote::Executando,
            StatusLote::Concluido,
            StatusLote::Falhou,
            StatusLote::Parcial,
            StatusLote::Bloqueado,
        ] {
            let s = status.as_str();
            assert!(s.len() <= 16, "status {s:?} excede VARCHAR(16)");
        }
    }
}
