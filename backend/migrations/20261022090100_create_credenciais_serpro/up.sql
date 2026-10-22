CREATE TABLE credenciais_serpro (
    id UUID PRIMARY KEY,
    escritorio_id UUID NOT NULL REFERENCES escritorios(id),
    produto VARCHAR(32) NOT NULL CHECK (produto IN ('INTEGRA_CONTADOR', 'DIVIDA_ATIVA')),
    ambiente VARCHAR(16) NOT NULL CHECK (ambiente IN ('TRIAL', 'PRODUCAO')),
    consumer_key_cifrado BYTEA NOT NULL,
    consumer_secret_cifrado BYTEA NOT NULL,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    atualizado_em TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (escritorio_id, produto, ambiente)
);
CREATE INDEX idx_credenciais_escritorio_produto
    ON credenciais_serpro(escritorio_id, produto, ambiente);
