CREATE TABLE execucoes_trabalho (
    id UUID PRIMARY KEY,
    trabalho_agendado_id UUID NOT NULL REFERENCES trabalhos_agendados(id) ON DELETE CASCADE,
    escritorio_id UUID NOT NULL REFERENCES escritorios(id) ON DELETE CASCADE,
    status VARCHAR(16) NOT NULL CHECK (status IN ('pendente', 'executando', 'concluido', 'falhou')),
    iniciado_em TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    finalizado_em TIMESTAMPTZ,
    erro_mensagem TEXT,
    registros_processados INTEGER NOT NULL DEFAULT 0,
    registros_com_erro INTEGER NOT NULL DEFAULT 0,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_execucoes_trabalho_escritorio ON execucoes_trabalho(escritorio_id);
CREATE INDEX idx_execucoes_trabalho_trabalho ON execucoes_trabalho(trabalho_agendado_id, criado_em DESC);
CREATE INDEX idx_execucoes_trabalho_status ON execucoes_trabalho(status, criado_em DESC);
