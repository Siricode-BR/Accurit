CREATE TABLE itens_consulta_lote (
    id UUID PRIMARY KEY,
    consulta_lote_id UUID NOT NULL REFERENCES consultas_em_lote(id) ON DELETE CASCADE,
    documento VARCHAR(14) NOT NULL,
    tipo VARCHAR(2) NOT NULL CHECK (tipo IN ('PF', 'PJ')),
    status VARCHAR(16) NOT NULL CHECK (status IN ('pendente', 'concluido', 'falhou')),
    resultado_redigido JSONB,
    erro TEXT,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    processado_em TIMESTAMPTZ
);
CREATE INDEX idx_itens_consulta_lote_lote ON itens_consulta_lote(consulta_lote_id);
CREATE INDEX idx_itens_consulta_lote_status ON itens_consulta_lote(status, criado_em DESC);
