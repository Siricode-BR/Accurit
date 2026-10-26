//! Erros da consulta de Dívida Ativa: mapeamento estável para
//! `ApiError` (código + HTTP status + mensagem) conforme SPEC §4.4 e
//! as decisões D2/D5. Mensagens são verbais e internas; o envelope do
//! cliente leva apenas o código estável.

use thiserror::Error;
use uuid::Uuid;

use super::enums::EstadoConsulta;
use crate::domain::serpro::erros::ErroEndpoint;

/// Camada SERPRO/HTTP: o desenrollo da chamada.
#[derive(Debug, Clone, Error)]
pub enum ErroSerpro {
    /// `401` depois de UMA renovação de token (SPEC §3.1.1).
    #[error("SERPRO recusou a autorização")]
    NaoAutorizado,
    /// `404` upstream — chave não encontrada (sem decisão D2 em aberto).
    #[error("chave não encontrada na Dívida Ativa (SERPRO 404)")]
    NaoEncontrado,
    /// `429` upstream com Retry-After (segundos) quando informado.
    #[error("limite de requisições atingido na SERPRO (429)")]
    LimiteExcedido { retry_after: Option<u64> },
    /// `5xx` upstream (sem Retry-After relevante).
    #[error("serviço SERPRO indisponível ({0})")]
    Indisponivel(u16),
    /// Connect/request timeout ou falha de transporte.
    #[error("timeout ou falha de transporte no gateway SERPRO")]
    TimeoutOuRede,
    /// `504` — estado indeterminado (SPEC §3.3): nunca reenviar imediatamente.
    #[error("resposta do gateway SERPRO indeterminada (504)")]
    Indeterminado,
}

#[derive(Debug, Clone, Error)]
pub enum ErroConsultaDivida {
    /// Chave inválida/ausente no path: 400 com mensagem explícita.
    #[error("parâmetro inválido: {0}")]
    ParametroInvalido(&'static str),

    /// Endpoint/ambiente não resolveu (config ausente ou produção
    /// bloqueada) — desfolha os códigos estáveis de `ErroEndpoint`.
    #[error("endpoint SERPRO indisponível: {0}")]
    Endpoint(#[from] ErroEndpoint),

    /// Desdobramento do HTTP para estados (SPEC §4.4/§3.3).
    #[error("SERPRO: {0}")]
    Serpro(#[from] ErroSerpro),

    /// Corpo veio como JSON mas viola o contrato: item borrado (moeda
    /// inválida) ou objeto sem campos de negócio.
    #[error("resposta da SERPRO fora do contrato esperado")]
    RespostaInvalida(String),

    /// Limite local por escritório excedido (SPEC §3.3) — falhar rápido.
    #[error("limite local de consultas por escritório excedido")]
    LimiteLocalExcedido,

    /// Persistência do histórico/leitura do cliente falhou.
    #[error("persistência do histórico de consultas indisponível")]
    PersistenciaIndisponivel,
}

impl ErroSerpro {
    pub const fn estado_consulta(&self) -> EstadoConsulta {
        match self {
            Self::NaoAutorizado => EstadoConsulta::NaoAutorizado,
            Self::NaoEncontrado => EstadoConsulta::NaoEncontrado,
            Self::LimiteExcedido { .. } => EstadoConsulta::LimiteExcedido,
            Self::Indisponivel(_) | Self::TimeoutOuRede => EstadoConsulta::Indisponivel,
            Self::Indeterminado => EstadoConsulta::Indeterminado,
        }
    }

    /// Código estável do `ErrorEnvelope` para o portal.
    pub const fn codigo(&self) -> &'static str {
        match self {
            Self::NaoAutorizado => "SERPRO_NAO_AUTORIZADO",
            Self::NaoEncontrado => "SERPRO_NAO_ENCONTRADO",
            Self::LimiteExcedido { .. } => "SERPRO_LIMITE_EXCEDIDO",
            Self::Indisponivel(_) | Self::TimeoutOuRede => "SERPRO_INDISPONIVEL",
            Self::Indeterminado => "SERPRO_INDETERMINADO",
        }
    }

    /// HTTP status correspondente no portal: ser corporateamente separado
    /// — não revelar detalhes upstream; usar states de integração.
    pub const fn status_http(&self) -> u16 {
        match self {
            Self::NaoAutorizado => 503,
            Self::NaoEncontrado => 404,
            Self::LimiteExcedido { .. } => 429,
            Self::Indisponivel(_) | Self::TimeoutOuRede => 503,
            Self::Indeterminado => 504,
        }
    }

    pub const fn retry_after_segundos(&self) -> Option<u64> {
        match self {
            Self::LimiteExcedido { retry_after } => *retry_after,
            _ => None,
        }
    }
}

impl ErroConsultaDivida {
    /// Código estável do envelope (nunca vaza detalhe de configuração).
    pub const fn codigo(&self) -> &'static str {
        match self {
            Self::ParametroInvalido(_) => "PARAMETRO_INVALIDO",
            Self::Endpoint(erro) => erro.codigo(),
            Self::Serpro(erro) => erro.codigo(),
            Self::RespostaInvalida(_) => "SERPRO_RESPOSTA_INVALIDA",
            Self::LimiteLocalExcedido => "LIMITE_REQUISICOES",
            Self::PersistenciaIndisponivel => "ERRO_INTERNO",
        }
    }

    pub const fn status_http(&self) -> u16 {
        match self {
            Self::ParametroInvalido(_) => 400,
            Self::Endpoint(erro) => match erro {
                ErroEndpoint::DividaAtivaProducaoBloqueada => 503,
                ErroEndpoint::Indefinido(_) | ErroEndpoint::Invalida { .. } => 503,
            },
            Self::Serpro(erro) => erro.status_http(),
            Self::RespostaInvalida(_) => 502,
            Self::LimiteLocalExcedido => 429,
            Self::PersistenciaIndisponivel => 503,
        }
    }

    /// Estado registrado em `consultas_fiscais.estado`.
    pub const fn estado_consulta(&self) -> EstadoConsulta {
        match self {
            Self::ParametroInvalido(_) => EstadoConsulta::ErroInterno,
            Self::Endpoint(_) => EstadoConsulta::Indisponivel,
            Self::Serpro(erro) => erro.estado_consulta(),
            Self::RespostaInvalida(_) => EstadoConsulta::RespostaInvalida,
            Self::LimiteLocalExcedido => EstadoConsulta::LimiteExcedido,
            Self::PersistenciaIndisponivel => EstadoConsulta::ErroInterno,
        }
    }

    /// `Retry-After` do SERPRO 429 (segundos), para ecoar no portal.
    pub const fn retry_after_segundos(&self) -> Option<u64> {
        match self {
            Self::Serpro(erro) => erro.retry_after_segundos(),
            _ => None,
        }
    }
}

/// Contexto mínimo para mensagens de log com identificação segura.
#[derive(Debug, Clone, Copy)]
pub struct ContextoConsulta {
    pub escritorio_id: Uuid,
    pub ambiente: crate::domain::serpro::ambiente::AmbienteSerpro,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::serpro::erros::ErroEndpoint;

    #[test]
    fn codigo_um_codigo_estavel_para_producao_bloqueada() {
        let erro = ErroEndpoint::DividaAtivaProducaoBloqueada;
        let consulta = ErroConsultaDivida::from(erro);

        assert_eq!(consulta.codigo(), "SERPRO_PROD_NOT_CONFIGURED");
        assert_eq!(consulta.status_http(), 503);
        assert_eq!(consulta.estado_consulta().as_str(), "INDISPONIVEL");
    }

    #[test]
    fn codigo_endpoint_indefinido_estavel() {
        let erro = ErroEndpoint::Indefinido("sem url".to_owned());
        let consulta = ErroConsultaDivida::from(erro);

        assert_eq!(consulta.codigo(), "SERPRO_ENDPOINT_NOT_CONFIGURED");
        assert_eq!(consulta.status_http(), 503);
    }

    #[test]
    fn retry_after_apenas_no_429() {
        assert_eq!(
            ErroConsultaDivida::from(ErroSerpro::LimiteExcedido {
                retry_after: Some(7)
            })
            .retry_after_segundos(),
            Some(7)
        );
        assert_eq!(
            ErroConsultaDivida::from(ErroSerpro::NaoAutorizado).retry_after_segundos(),
            None
        );
    }
}
