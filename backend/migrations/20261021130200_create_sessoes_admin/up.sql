CREATE TABLE sessoes_admin (
    id UUID PRIMARY KEY,
    administrador_id UUID NOT NULL REFERENCES administradores(id),
    familia_id UUID NOT NULL,
    refresh_hash CHAR(64) NOT NULL UNIQUE,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expira_em TIMESTAMPTZ NOT NULL,
    revogada_em TIMESTAMPTZ,
    ip INET,
    user_agent TEXT,
    CHECK (expira_em > criado_em)
);
CREATE INDEX idx_sessoes_admin_familia ON sessoes_admin(familia_id);
