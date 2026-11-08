CREATE TABLE guias_emissao (
    id UUID PRIMARY KEY,
    usuario_id UUID NOT NULL REFERENCES usuarios(id) ON DELETE CASCADE,
    escritorio_id UUID NOT NULL REFERENCES escritorios(id) ON DELETE CASCADE,
    cliente_id UUID NOT NULL REFERENCES clientes(id) ON DELETE CASCADE,
    tipo_guia VARCHAR(16) NOT NULL CHECK (tipo_guia IN ('DAS', 'DARF')),
    competencia VARCHAR(7) NOT NULL CHECK (competencia ~ '^\d{4}-\d{2}$'),
    valor_total NUMERIC(14, 2) NOT NULL,
    pdf_url TEXT,
    status VARCHAR(16) NOT NULL CHECK (status IN ('solicitada', 'emitida', 'baixada', 'falhou')),
    idempotencia_chave VARCHAR(64) NOT NULL,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    atualizado_em TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (idempotencia_chave)
);
CREATE INDEX idx_guias_emissao_usuario_id ON guias_emissao(usuario_id);
CREATE INDEX idx_guias_emissao_escritorio_id ON guias_emissao(escritorio_id);
CREATE INDEX idx_guias_emissao_cliente_id ON guias_emissao(cliente_id);
CREATE INDEX idx_guias_emissao_competencia ON guias_emissao(competencia);
CREATE INDEX idx_guias_emissao_idempotencia ON guias_emissao(idempotencia_chave);
