CREATE TABLE trabalhos_agendados (
    id UUID PRIMARY KEY,
    escritorio_id UUID NOT NULL REFERENCES escritorios(id) ON DELETE CASCADE,
    nome VARCHAR(255) NOT NULL,
    tipo VARCHAR(32) NOT NULL CHECK (tipo IN ('consulta_divida_ativa', 'validar_procuracao', 'consulta_integra_contador')),
    produto VARCHAR(32) NOT NULL CHECK (produto IN ('INTEGRA_CONTADOR', 'DIVIDA_ATIVA')),
    ambiente VARCHAR(16) NOT NULL CHECK (ambiente IN ('TRIAL', 'PRODUCAO')),
    cron_expression VARCHAR(64) NOT NULL,
    habilitado BOOLEAN NOT NULL DEFAULT TRUE,
    proxima_execucao TIMESTAMPTZ,
    ultima_execucao TIMESTAMPTZ,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    atualizado_em TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_trabalhos_agendados_escritorio ON trabalhos_agendados(escritorio_id);
CREATE INDEX idx_trabalhos_agendados_proxima_execucao ON trabalhos_agendados(proxima_execucao) WHERE habilitado = TRUE;
