CREATE TABLE consultas_fiscais (
    id UUID PRIMARY KEY,
    escritorio_id UUID NOT NULL REFERENCES escritorios(id),
    cliente_id UUID REFERENCES clientes(id),
    produto VARCHAR(32) NOT NULL,
    servico VARCHAR(64) NOT NULL,
    verbo VARCHAR(16),
    ambiente VARCHAR(16) NOT NULL CHECK (ambiente IN ('TRIAL', 'PRODUCAO')),
    chave_consulta_hash CHAR(64),
    chave_consulta_mascarada VARCHAR(20),
    fonte_consulta VARCHAR(32),
    status_code INTEGER,
    estado VARCHAR(32) NOT NULL,
    protocolo TEXT,
    request_redigido JSONB,
    response_redigido JSONB,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_consultas_escritorio_criado
    ON consultas_fiscais(escritorio_id, criado_em DESC);
CREATE INDEX idx_consultas_cliente ON consultas_fiscais(cliente_id);
