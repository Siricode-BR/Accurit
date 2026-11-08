//! Erros do domínio de trabalhos agendados e consultas em lote.

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ErroTrabalho {
    #[error("trabalho agendado não encontrado")]
    TrabalhoNaoEncontrado,
    #[error("execução de trabalho não encontrada")]
    ExecucaoNaoEncontrada,
    #[error("consulta em lote não encontrada")]
    ConsultaLoteNaoEncontrada,
    #[error("item de lote não encontrado")]
    ItemLoteNaoEncontrado,
    #[error("trabalho já está desabilitado")]
    JaDesabilitado,
    #[error("trabalho já está habilitado")]
    JaHabilitado,
    #[error("não é possível cancelar execução em andamento")]
    ExecucaoEmAndamento,
    #[error("escritório não possui permissão para este recurso")]
    SemPermissao,
    #[error("falha de persistência: {0}")]
    Persistencia(String),
    #[error("erro interno: {0}")]
    Interno(String),
}

impl ErroTrabalho {
    pub const fn codigo(&self) -> &'static str {
        match self {
            Self::TrabalhoNaoEncontrado => "TRABALHO_NAO_ENCONTRADO",
            Self::ExecucaoNaoEncontrada => "EXECUCAO_NAO_ENCONTRADA",
            Self::ConsultaLoteNaoEncontrada => "CONSULTA_LOTE_NAO_ENCONTRADA",
            Self::ItemLoteNaoEncontrado => "ITEM_LOTE_NAO_ENCONTRADO",
            Self::JaDesabilitado => "TRABALHO_JA_DESABILITADO",
            Self::JaHabilitado => "TRABALHO_JA_HABILITADO",
            Self::ExecucaoEmAndamento => "EXECUCAO_EM_ANDAMENTO",
            Self::SemPermissao => "SEM_PERMISSAO",
            Self::Persistencia(_) => "PERSISTENCIA",
            Self::Interno(_) => "ERRO_INTERNO",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codigos_estaveis() {
        assert_eq!(
            ErroTrabalho::TrabalhoNaoEncontrado.codigo(),
            "TRABALHO_NAO_ENCONTRADO"
        );
        assert_eq!(
            ErroTrabalho::ExecucaoNaoEncontrada.codigo(),
            "EXECUCAO_NAO_ENCONTRADA"
        );
        assert_eq!(
            ErroTrabalho::ConsultaLoteNaoEncontrada.codigo(),
            "CONSULTA_LOTE_NAO_ENCONTRADA"
        );
        assert_eq!(ErroTrabalho::SemPermissao.codigo(), "SEM_PERMISSAO");
        assert_eq!(
            ErroTrabalho::Persistencia("x".into()).codigo(),
            "PERSISTENCIA"
        );
    }
}
