//! Fase G — Enums de medição de uso SERpro.

use std::fmt;
use std::str::FromStr;

/// Origem da chamada (manual, rotina agendada, lote).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrigemChamada {
    Manual,
    Rotina,
    Lote,
}

impl OrigemChamada {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::Rotina => "rotina",
            Self::Lote => "lote",
        }
    }
}

impl FromStr for OrigemChamada {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "manual" => Ok(Self::Manual),
            "rotina" => Ok(Self::Rotina),
            "lote" => Ok(Self::Lote),
            _ => Err(format!("origem desconhecida: {s}")),
        }
    }
}

impl fmt::Display for OrigemChamada {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Resultado da chamada SERpro.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultadoChamada {
    Sucesso,
    Erro4xx,
    Erro5xx,
    Timeout,
    Indeterminado,
}

impl ResultadoChamada {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Sucesso => "sucesso",
            Self::Erro4xx => "4xx",
            Self::Erro5xx => "5xx",
            Self::Timeout => "timeout",
            Self::Indeterminado => "indeterminado",
        }
    }
}

impl FromStr for ResultadoChamada {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "sucesso" => Ok(Self::Sucesso),
            "4xx" => Ok(Self::Erro4xx),
            "5xx" => Ok(Self::Erro5xx),
            "timeout" => Ok(Self::Timeout),
            "indeterminado" => Ok(Self::Indeterminado),
            _ => Err(format!("resultado desconhecido: {s}")),
        }
    }
}

impl fmt::Display for ResultadoChamada {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Período de agregação (dia ou mês).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeriodoAgregado {
    Dia,
    Mes,
}

impl PeriodoAgregado {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Dia => "dia",
            Self::Mes => "mes",
        }
    }
}

impl FromStr for PeriodoAgregado {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "dia" => Ok(Self::Dia),
            "mes" => Ok(Self::Mes),
            _ => Err(format!("período desconhecido: {s}")),
        }
    }
}

impl fmt::Display for PeriodoAgregado {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Motivo de bloqueio local (não conta como chamada SERpro).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotivoBloqueio {
    RateLimitInterno,
    ProducaoNaoConfigurada,
    ParametroInvalido,
    /// Limite de chamadas do plano do escritório atingido (Fase B.1 —
    /// bloqueio explícito e auditado pelo job runner).
    LimitePlano,
}

impl MotivoBloqueio {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RateLimitInterno => "rate_limit_interno",
            Self::ProducaoNaoConfigurada => "producao_nao_configurada",
            Self::ParametroInvalido => "parametro_invalido",
            Self::LimitePlano => "limite_plano",
        }
    }
}

impl FromStr for MotivoBloqueio {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "rate_limit_interno" => Ok(Self::RateLimitInterno),
            "producao_nao_configurada" => Ok(Self::ProducaoNaoConfigurada),
            "parametro_invalido" => Ok(Self::ParametroInvalido),
            "limite_plano" => Ok(Self::LimitePlano),
            _ => Err(format!("motivo desconhecido: {s}")),
        }
    }
}

impl fmt::Display for MotivoBloqueio {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
