-- Fase E — Gestão de carteira: tags de clientes do escritório.
CREATE TABLE cliente_tags (
    id UUID PRIMARY KEY,
    escritorio_id UUID NOT NULL REFERENCES escritorios(id) ON DELETE CASCADE,
    nome VARCHAR(64) NOT NULL,
    cor VARCHAR(16),
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    atualizado_em TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_cliente_tags_escritorio ON cliente_tags(escritorio_id);
-- Nome de tag único no escritório, case-insensitive.
CREATE UNIQUE INDEX uq_cliente_tags_nome ON cliente_tags(escritorio_id, lower(nome));
