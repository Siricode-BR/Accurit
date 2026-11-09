-- Fase E — Gestão de carteira: vínculo de tag a cliente.
CREATE TABLE cliente_tags_vinculos (
    id UUID PRIMARY KEY,
    escritorio_id UUID NOT NULL REFERENCES escritorios(id) ON DELETE CASCADE,
    tag_id UUID NOT NULL REFERENCES cliente_tags(id) ON DELETE CASCADE,
    cliente_id UUID NOT NULL REFERENCES clientes(id) ON DELETE CASCADE,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (tag_id, cliente_id)
);
CREATE INDEX idx_cliente_tags_vinculos_cliente ON cliente_tags_vinculos(cliente_id);
CREATE INDEX idx_cliente_tags_vinculos_tag ON cliente_tags_vinculos(tag_id);
