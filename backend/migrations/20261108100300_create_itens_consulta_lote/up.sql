CREATE TABLE itens_consulta_lote (
    id UUID PRIMARY KEY,
    consulta_lote_id UUID NOT NULL REFERENCES consultas_em_lote(id) ON DELETE CASCADE,
    documento VARCHAR(14) NOT NULL,
    tipo VARCHAR(2) NOT NULL CHECK (tipo IN ('PF', 'PJ')),
    status VARCHAR(16) NOT NULL CHECK (status IN ('pendente', 'executando', 'concluido', 'falhou', 'bloqueado')),
    resultado_redigido JSONB,
    erro TEXT,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    processado_em TIMESTAMPTZ,
    -- Job runner (Fase B.1): tentativas do item, backoff exponencial
    -- (proxima_tentativa_em) e lease/heartbeat com expiração.
    tentativas INTEGER NOT NULL DEFAULT 0,
    proxima_tentativa_em TIMESTAMPTZ,
    lease_expira_em TIMESTAMPTZ
);
CREATE INDEX idx_itens_consulta_lote_lote ON itens_consulta_lote(consulta_lote_id);
CREATE INDEX idx_itens_consulta_lote_status ON itens_consulta_lote(status, criado_em DESC);
CREATE INDEX idx_itens_consulta_lote_lease ON itens_consulta_lote(status, lease_expira_em);
