-- Os campos Fase 5 de tokens_serpro e procuracoes já estão nas migrations
-- de criação. Esta migration adiciona apenas idempotência e metadados
-- operacionais, sem alterar tabelas existentes.
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
CREATE INDEX idx_idempotencia_escritorio_criado
    ON idempotencia_operacoes(escritorio_id, criado_em DESC);

CREATE TABLE chamadas_operacionais (
    id UUID PRIMARY KEY,
    escritorio_id UUID NOT NULL REFERENCES escritorios(id),
    usuario_id UUID,
    verbo VARCHAR(16) NOT NULL CHECK (
        verbo IN ('Apoiar', 'Consultar', 'Declarar', 'Emitir', 'Monitorar')
    ),
    id_sistema VARCHAR(32) NOT NULL,
    id_servico VARCHAR(64) NOT NULL,
    ambiente VARCHAR(16) NOT NULL CHECK (ambiente IN ('TRIAL', 'PRODUCAO')),
    estado VARCHAR(32) NOT NULL,
    status_code INTEGER,
    referencia_resposta VARCHAR(128),
    request_id VARCHAR(64),
    bilhetavel BOOLEAN NOT NULL DEFAULT FALSE,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_chamadas_operacionais_escritorio_criado
    ON chamadas_operacionais(escritorio_id, criado_em DESC);
CREATE INDEX idx_chamadas_operacionais_escritorio_estado
    ON chamadas_operacionais(escritorio_id, estado, criado_em DESC);
