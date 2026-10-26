//! Estados e fontes da consulta de Dívida Ativa.
//!
//! `EstadoConsulta` descreve o desfecho de UMA tentativa registrada em
//! `consultas_fiscais.estado` (coluna VARCHAR(32)). Esses valores são o
//! contrato do histórico: o frontend distingue "sem pendências" de erro
//! de integração e de falha de autorização (SPEC §4.5).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EstadoConsulta {
    /// SERPRO respondeu dados do devedor/inscrição.
    Sucesso,
    /// Resposta 200 inequivocamente vazia (SPEC §4.4 passo 7).
    SemPendencias,
    /// RESPRO respondeu 404 para a chave consultada (D2: live test em
    /// aberto determina se "sem pendências" virá como 404; o registro
    /// mantém o estado distinto até a confirmação).
    NaoEncontrado,
    /// SERPRO recusou a autorização mesmo após UMA renovação (SPEC §3.1.1).
    NaoAutorizado,
    /// SERPRO respondeu 429 (Retry-After respeitado pelo chamador).
    LimiteExcedido,
    /// 5xx, timeout ou falha de rede — serviço indisponível.
    Indisponivel,
    /// Corpo fora do contrato (moeda pt-BR inválida, JSON sem campos).
    RespostaInvalida,
    /// HTTP 504 — estado indeterminado; não há reenvio imediato (SPEC §3.3).
    Indeterminado,
    /// Falha local (persistência do registro, por exemplo).
    ErroInterno,
}

impl EstadoConsulta {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Sucesso => "SUCESSO",
            Self::SemPendencias => "SEM_PENDENCIAS",
            Self::NaoEncontrado => "NAO_ENCONTRADO",
            Self::NaoAutorizado => "NAO_AUTORIZADO",
            Self::LimiteExcedido => "LIMITE_EXCEDIDO",
            Self::Indisponivel => "INDISPONIVEL",
            Self::RespostaInvalida => "RESPOSTA_INVALIDA",
            Self::Indeterminado => "INDETERMINADO",
            Self::ErroInterno => "ERRO_INTERNO",
        }
    }
}

/// Origem da chave utilizada na consulta (coluna
/// `consultas_fiscais.fonte_consulta`, VARCHAR(32)); o fallback do
/// "Código de Contribuinte" (D2) usa `CODIGO_CONTRIBUINTE` e só ocorre
/// explicitamente habilitado.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FonteConsulta {
    CpfCnpj,
    CodigoContribuinte,
}

impl FonteConsulta {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CpfCnpj => "CPF_CNPJ",
            Self::CodigoContribuinte => "CODIGO_CONTRIBUINTE",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{EstadoConsulta, FonteConsulta};

    #[test]
    fn estados_sao_estaveis_e_cabem_na_coluna() {
        let estados = [
            EstadoConsulta::Sucesso.as_str(),
            EstadoConsulta::SemPendencias.as_str(),
            EstadoConsulta::NaoEncontrado.as_str(),
            EstadoConsulta::NaoAutorizado.as_str(),
            EstadoConsulta::LimiteExcedido.as_str(),
            EstadoConsulta::Indisponivel.as_str(),
            EstadoConsulta::RespostaInvalida.as_str(),
            EstadoConsulta::Indeterminado.as_str(),
            EstadoConsulta::ErroInterno.as_str(),
        ];

        for estado in estados {
            assert!(estado.len() <= 32, "estado {estado} excede VARCHAR(32)");
            for byte in estado.bytes() {
                assert!(byte.is_ascii_uppercase() || byte == b'_');
            }
        }
    }

    #[test]
    fn fontes_sao_estaveis() {
        assert_eq!(FonteConsulta::CpfCnpj.as_str(), "CPF_CNPJ");
        assert_eq!(
            FonteConsulta::CodigoContribuinte.as_str(),
            "CODIGO_CONTRIBUINTE"
        );
    }
}
