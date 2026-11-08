CREATE TABLE preferencias_notificacao (
    id UUID PRIMARY KEY,
    usuario_id UUID NOT NULL UNIQUE REFERENCES usuarios(id) ON DELETE CASCADE,
    canal_email BOOLEAN NOT NULL DEFAULT TRUE,
    canal_whatsapp BOOLEAN NOT NULL DEFAULT FALSE,
    canal_in_app BOOLEAN NOT NULL DEFAULT TRUE,
    tipos_ativados JSONB NOT NULL DEFAULT '[]'::jsonb,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    atualizado_em TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_preferencias_notificacao_usuario_id ON preferencias_notificacao(usuario_id);
