CREATE TABLE politicas_retencao_escritorio (
    escritorio_id UUID PRIMARY KEY REFERENCES escritorios(id) ON DELETE CASCADE,
    consultas_dias INTEGER NOT NULL DEFAULT 90 CHECK (consultas_dias BETWEEN 1 AND 3650),
    relatorios_dias INTEGER NOT NULL DEFAULT 30 CHECK (relatorios_dias BETWEEN 1 AND 3650),
    auditoria_dias INTEGER NOT NULL DEFAULT 365 CHECK (auditoria_dias BETWEEN 1 AND 3650),
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    atualizado_em TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE consentimentos_lgpd (
    id UUID PRIMARY KEY,
    escritorio_id UUID NOT NULL REFERENCES escritorios(id) ON DELETE CASCADE,
    cliente_id UUID NOT NULL REFERENCES clientes(id) ON DELETE CASCADE,
    tipo VARCHAR(32) NOT NULL CHECK (tipo IN ('tratamento_dados', 'compartilhamento_serpro', 'comunicacoes')),
    versao VARCHAR(16) NOT NULL,
    aceito_em TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    revogado_em TIMESTAMPTZ NULL,
    ip INET NULL,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_politicas_retencao_escritorio ON politicas_retencao_escritorio(escritorio_id);
CREATE INDEX idx_consentimentos_lgpd_cliente ON consentimentos_lgpd(cliente_id);
CREATE INDEX idx_consentimentos_lgpd_escritorio ON consentimentos_lgpd(escritorio_id);
