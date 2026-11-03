-- Fase 5 (Integra Contador): idempotência de operações mutáveis
-- (SPEC §9 — `idempotencia_operacoes`). A bilhetagem operacional NÃO
-- recebe tabela nova: seus registros vão para `consultas_fiscais` com
-- `produto = 'INTEGRA_CONTADOR'` (mesma tabela/retenção da Fase 4) e as
-- operações mutáveis também registram eventos em `auditoria_eventos`.

CREATE TABLE idempotencia_operacoes (
    id UUID PRIMARY KEY,
    escritorio_id UUID NOT NULL REFERENCES escritorios(id),
    chave_idempotencia VARCHAR(64) NOT NULL,
    verbo VARCHAR(16) NOT NULL CHECK (verbo IN ('Declarar', 'Emitir')),
    id_sistema VARCHAR(32) NOT NULL,
    id_servico VARCHAR(32) NOT NULL,
    versao_sistema VARCHAR(16) NOT NULL,
    ambiente VARCHAR(16) NOT NULL CHECK (ambiente IN ('TRIAL', 'PRODUCAO')),
    payload_hash CHAR(64) NOT NULL,
    estado VARCHAR(32) NOT NULL,
    resposta_redigida JSONB,
    resposta_status INTEGER,
    processado_em TIMESTAMPTZ,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (escritorio_id, chave_idempotencia)
);
CREATE INDEX idx_idempotencia_escritorio_criado ON idempotencia_operacoes(escritorio_id, criado_em DESC);
