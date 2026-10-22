CREATE TABLE procuracoes (
    id UUID PRIMARY KEY,
    escritorio_id UUID NOT NULL REFERENCES escritorios(id),
    ambiente VARCHAR(16) NOT NULL CHECK (ambiente IN ('TRIAL', 'PRODUCAO')),
    contribuinte_cpf_cnpj VARCHAR(14) NOT NULL,
    procurador_cpf_cnpj VARCHAR(14) NOT NULL,
    codigo_servico VARCHAR(5) NOT NULL,
    status VARCHAR(32) NOT NULL,
    valido_de TIMESTAMPTZ,
    valido_ate TIMESTAMPTZ,
    verificado_em TIMESTAMPTZ NOT NULL,
    origem VARCHAR(32) NOT NULL DEFAULT 'OBTERPROCURACAO41',
    UNIQUE (escritorio_id, ambiente, contribuinte_cpf_cnpj, procurador_cpf_cnpj, codigo_servico)
);
CREATE INDEX idx_procuracoes_escritorio_servico
    ON procuracoes(escritorio_id, ambiente, codigo_servico);
