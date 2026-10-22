CREATE TABLE clientes (
    id UUID PRIMARY KEY,
    escritorio_id UUID NOT NULL REFERENCES escritorios(id),
    tipo VARCHAR(2) NOT NULL CHECK (tipo IN ('PF', 'PJ')),
    porte_regime VARCHAR(32),
    cpf_cnpj VARCHAR(14) NOT NULL,
    nome_razao_social TEXT NOT NULL,
    nome_fantasia TEXT,
    email TEXT,
    telefone TEXT,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    atualizado_em TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (
      (tipo = 'PF' AND porte_regime IS NULL) OR
      (tipo = 'PJ' AND (porte_regime IS NULL OR porte_regime IN
        ('MEI','SIMPLES_NACIONAL','LUCRO_PRESUMIDO','LUCRO_REAL')))
    ),
    CHECK (cpf_cnpj ~ '^[0-9]{11}$' OR cpf_cnpj ~ '^[0-9]{14}$'),
    UNIQUE (escritorio_id, cpf_cnpj)
);
CREATE INDEX idx_clientes_escritorio_tipo ON clientes(escritorio_id, tipo);
CREATE INDEX idx_clientes_escritorio_regime ON clientes(escritorio_id, porte_regime);
