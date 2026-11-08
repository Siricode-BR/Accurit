CREATE TABLE consultas_em_lote (
    id UUID PRIMARY KEY,
    escritorio_id UUID NOT NULL REFERENCES escritorios(id) ON DELETE CASCADE,
    nome VARCHAR(255) NOT NULL,
    produto VARCHAR(32) NOT NULL CHECK (produto IN ('INTEGRA_CONTADOR', 'DIVIDA_ATIVA')),
    servico VARCHAR(64) NOT NULL,
    verbo VARCHAR(16),
    status VARCHAR(16) NOT NULL CHECK (status IN ('pendente', 'executando', 'concluido', 'falhou', 'parcial')),
    total_itens INTEGER NOT NULL DEFAULT 0,
    itens_concluidos INTEGER NOT NULL DEFAULT 0,
    itens_com_erro INTEGER NOT NULL DEFAULT 0,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    iniciado_em TIMESTAMPTZ,
    finalizado_em TIMESTAMPTZ
);
CREATE INDEX idx_consultas_em_lote_escritorio ON consultas_em_lote(escritorio_id);
CREATE INDEX idx_consultas_em_lote_status ON consultas_em_lote(status, criado_em DESC);
