use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ErroCredencial {
    #[error(
        "credencial não encontrada para escritório {escritorio_id}, produto {produto}, ambiente {ambiente}"
    )]
    NaoEncontrada {
        escritorio_id: Uuid,
        produto: String,
        ambiente: String,
    },
    #[error("credencial inválida: {0}")]
    Invalida(String),
    #[error("certificado não encontrado, fora de validade ou revogado")]
    CertificadoInvalido,
    #[error("erro de criptografia: {0}")]
    Criptografia(String),
    #[error("repositório indisponível")]
    RepositorioIndisponivel,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ErroAmbiente {
    #[error("escritório não encontrado: {0}")]
    EscritorioNaoEncontrado(Uuid),
    #[error("conflito de versão ao alterar modo: esperado {esperado}, atual {atual}")]
    ConflitoVersao { esperado: i64, atual: i64 },
    #[error("não é possível alternar para produção: {0}")]
    ProducaoBloqueada(String),
    #[error("repositório indisponível")]
    RepositorioIndisponivel,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ErroEndpoint {
    #[error("configuração SERPRO ausente para o produto/ambiente: {0}")]
    Indefinido(String),
    #[error(
        "Serviço de Dívida Ativa em produção permanece indisponível até decisão da SERPRO (URL de produção indefinida)"
    )]
    DividaAtivaProducaoBloqueada,
    #[error("configuração inválida: {variavel} — {motivo}")]
    Invalida { variavel: String, motivo: String },
}

impl ErroEndpoint {
    /// Código estável para desenvelope de erro (fase 4+). Detalhes técnicos
    /// (nomes de variáveis) ficam nos logs do servidor, não na resposta.
    pub const fn codigo(&self) -> &'static str {
        match self {
            Self::Indefinido(_) | Self::Invalida { .. } => "SERPRO_ENDPOINT_NOT_CONFIGURED",
            Self::DividaAtivaProducaoBloqueada => "SERPRO_PROD_NOT_CONFIGURED",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn erro_credencial_nao_encontrada_display() {
        let err = ErroCredencial::NaoEncontrada {
            escritorio_id: Uuid::nil(),
            produto: "INTEGRA_CONTADOR".to_string(),
            ambiente: "TRIAL".to_string(),
        };
        assert!(err.to_string().contains("não encontrada"));
    }

    #[test]
    fn erro_credencial_certificado_invalido_display() {
        let err = ErroCredencial::CertificadoInvalido;
        assert!(err.to_string().contains("certificado"));
    }

    #[test]
    fn erro_ambiente_producao_bloqueada_display() {
        let err = ErroAmbiente::ProducaoBloqueada("credencial A1 ausente".to_string());
        assert!(err.to_string().contains("credencial A1 ausente"));
    }

    #[test]
    fn erro_ambiente_conflito_versao_display() {
        let err = ErroAmbiente::ConflitoVersao {
            esperado: 1,
            atual: 3,
        };
        assert!(err.to_string().contains("atual 3"));
    }

    #[test]
    fn erro_endpoint_codigos_estaveis() {
        assert_eq!(
            ErroEndpoint::DividaAtivaProducaoBloqueada.codigo(),
            "SERPRO_PROD_NOT_CONFIGURED"
        );
        assert_eq!(
            ErroEndpoint::Indefinido("url trial".to_owned()).codigo(),
            "SERPRO_ENDPOINT_NOT_CONFIGURED"
        );
    }

    #[test]
    fn erro_endpoint_producao_bloqueada_explica_divida_ativa() {
        let mensagem = ErroEndpoint::DividaAtivaProducaoBloqueada.to_string();
        assert!(mensagem.contains("indisponível"));
        assert!(mensagem.contains("Dívida Ativa"));
    }
}
