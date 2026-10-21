CREATE TABLE auditoria_eventos (
    id UUID PRIMARY KEY,
    escritorio_id UUID NOT NULL REFERENCES escritorios(id),
    usuario_id UUID,
    acao VARCHAR(64) NOT NULL,
    entidade VARCHAR(64),
    entidade_id UUID,
    valor_anterior JSONB,
    valor_novo JSONB,
    ip INET,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_auditoria_escritorio_criado ON auditoria_eventos(escritorio_id, criado_em DESC);
