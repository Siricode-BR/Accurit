CREATE TABLE sessoes (
    id UUID PRIMARY KEY,
    escritorio_id UUID NOT NULL REFERENCES escritorios(id),
    usuario_id UUID NOT NULL REFERENCES usuarios(id),
    audiencia VARCHAR(8) NOT NULL CHECK (audiencia IN ('CLIENT')),
    familia_id UUID NOT NULL,
    refresh_hash CHAR(64) NOT NULL UNIQUE,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expira_em TIMESTAMPTZ NOT NULL,
    revogada_em TIMESTAMPTZ,
    ip INET,
    user_agent TEXT,
    CHECK (expira_em > criado_em)
);
CREATE INDEX idx_sessoes_escritorio_usuario ON sessoes(escritorio_id, usuario_id);
CREATE INDEX idx_sessoes_familia ON sessoes(familia_id);
