CREATE TABLE escritorios (
    id UUID PRIMARY KEY,
    slug VARCHAR(40) NOT NULL UNIQUE CHECK (slug ~ '^[a-z0-9-]{3,40}$'),
    nome TEXT NOT NULL,
    cnpj VARCHAR(14) UNIQUE CHECK (cnpj IS NULL OR cnpj ~ '^[0-9]{14}$'),
    modo_trial_serpro BOOLEAN NOT NULL DEFAULT TRUE,
    versao BIGINT NOT NULL DEFAULT 1,
    atualizado_em TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
