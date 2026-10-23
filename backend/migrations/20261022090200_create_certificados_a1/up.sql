CREATE TABLE certificados_a1 (
    id UUID PRIMARY KEY,
    escritorio_id UUID NOT NULL REFERENCES escritorios(id),
    tipo VARCHAR(2) NOT NULL CHECK (tipo IN ('A1', 'A3')),
    titular_cpf_cnpj VARCHAR(14) NOT NULL,
    conteudo_cifrado BYTEA NOT NULL,
    senha_cifrada BYTEA,
    algoritmo VARCHAR(32) NOT NULL DEFAULT 'AES-256-GCM',
    fingerprint_sha256 CHAR(64) NOT NULL,
    valido_de TIMESTAMPTZ NOT NULL,
    valido_ate TIMESTAMPTZ NOT NULL,
    ativo BOOLEAN NOT NULL DEFAULT TRUE,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (titular_cpf_cnpj ~ '^[0-9]{11}$' OR titular_cpf_cnpj ~ '^[0-9]{14}$'),
    CHECK (valido_ate > valido_de),
    UNIQUE (escritorio_id, fingerprint_sha256)
);
CREATE INDEX idx_certificados_escritorio_validade
    ON certificados_a1(escritorio_id, valido_ate);
