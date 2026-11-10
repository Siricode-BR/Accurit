CREATE TABLE escritorios (
    id UUID PRIMARY KEY,
    nome TEXT NOT NULL,
    cnpj VARCHAR(14) UNIQUE NOT NULL CHECK (cnpj ~ '^[0-9]{14}$'),
    modo_trial_serpro BOOLEAN NOT NULL DEFAULT TRUE,
    versao BIGINT NOT NULL DEFAULT 1,
    atualizado_em TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
