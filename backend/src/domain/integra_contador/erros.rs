use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ErroIntegraContador {
    #[error("serviço não está habilitado")]
    ServicoNaoHabilitado,
    #[error("versão do serviço ainda não foi confirmada")]
    VersaoNaoConfirmada,
    #[error("payload inválido")]
    PayloadInvalido,
    #[error("confirmação explícita necessária")]
    ConfirmacaoNecessaria,
    #[error("chave de idempotência necessária")]
    IdempotenciaNecessaria,
    #[error("chave de idempotência conflitante ou já reservada")]
    ConflitoIdempotencia,
    #[error("autorização/procuração ausente ou não verificável")]
    ProcuracaoNegada,
    #[error("ambiente de produção bloqueado até confirmação contratual")]
    ProducaoBloqueada,
    #[error("contexto do escritório incompleto")]
    ConfiguracaoIncompleta,
    #[error("SERPRO não autorizado")]
    NaoAutorizado,
    #[error("SERPRO negou autorização")]
    AcessoNegado,
    #[error("limite SERPRO excedido")]
    LimiteExcedido,
    #[error("estado indeterminado")]
    Indeterminado,
    #[error("resposta SERPRO inválida")]
    RespostaInvalida,
    #[error("SERPRO indisponível")]
    Indisponivel,
    #[error("erro de persistência")]
    Persistencia,
}
