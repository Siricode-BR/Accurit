CREATE TABLE usuarios (
    id UUID PRIMARY KEY,
    escritorio_id UUID NOT NULL REFERENCES escritorios(id),
    email TEXT NOT NULL,
    cpf VARCHAR(11) NOT NULL,
    senha_hash TEXT NOT NULL,
    papel VARCHAR(32) NOT NULL CHECK (papel IN ('CLIENTE', 'OPERADOR')),
    ativo BOOLEAN NOT NULL DEFAULT TRUE,
    ultimo_login_em TIMESTAMPTZ,
    falhas_consecutivas INTEGER NOT NULL DEFAULT 0,
    bloqueado_ate TIMESTAMPTZ,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (cpf ~ '^[0-9]{11}$'),
    UNIQUE (escritorio_id, email),
    UNIQUE (escritorio_id, cpf)
);
CREATE INDEX idx_usuarios_escritorio ON usuarios(escritorio_id);
