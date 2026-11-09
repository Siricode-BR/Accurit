-- Fase E — Gestão de carteira: vínculos de carteira por usuário
-- (restrição explícita: com vínculos, o usuário só enxerga esses clientes;
-- sem vínculos, enxerga toda a carteira do escritório).
CREATE TABLE carteira_usuarios (
    id UUID PRIMARY KEY,
    escritorio_id UUID NOT NULL REFERENCES escritorios(id) ON DELETE CASCADE,
    usuario_id UUID NOT NULL REFERENCES usuarios(id) ON DELETE CASCADE,
    cliente_id UUID NOT NULL REFERENCES clientes(id) ON DELETE CASCADE,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (usuario_id, cliente_id)
);
CREATE INDEX idx_carteira_usuarios_usuario ON carteira_usuarios(usuario_id);
CREATE INDEX idx_carteira_usuarios_cliente ON carteira_usuarios(cliente_id);
