CREATE TABLE tokens_serpro (
    id UUID PRIMARY KEY,
    escritorio_id UUID NOT NULL REFERENCES escritorios(id),
    produto VARCHAR(32) NOT NULL CHECK (produto IN ('INTEGRA_CONTADOR', 'DIVIDA_ATIVA')),
    ambiente VARCHAR(16) NOT NULL CHECK (ambiente IN ('TRIAL', 'PRODUCAO')),
    tipo_token VARCHAR(32) NOT NULL CHECK (tipo_token IN ('ACCESS', 'JWT', 'AUTENTICAR_PROCURADOR')),
    autor_pedido_dados VARCHAR(14) NOT NULL DEFAULT '',
    token_cifrado BYTEA NOT NULL,
    escopo TEXT,
    expira_em TIMESTAMPTZ NOT NULL,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (escritorio_id, produto, ambiente, tipo_token, autor_pedido_dados)
);
