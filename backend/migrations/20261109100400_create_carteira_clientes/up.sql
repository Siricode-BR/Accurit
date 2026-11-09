-- Fase E — Gestão de carteira: responsável interno e observações por cliente
-- (a "carteira" do usuário). Uma linha por cliente, criada sob demanda.
CREATE TABLE carteira_clientes (
    cliente_id UUID PRIMARY KEY REFERENCES clientes(id) ON DELETE CASCADE,
    escritorio_id UUID NOT NULL REFERENCES escritorios(id) ON DELETE CASCADE,
    responsavel_usuario_id UUID REFERENCES usuarios(id) ON DELETE SET NULL,
    observacoes TEXT,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    atualizado_em TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_carteira_clientes_escritorio ON carteira_clientes(escritorio_id);
CREATE INDEX idx_carteira_clientes_responsavel ON carteira_clientes(responsavel_usuario_id);
