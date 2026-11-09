-- Fase E — Gestão de carteira: sócios e contatos de um cliente.
-- Papel "SOCIO" vale para PJ; "CONTATO" vale para PF e PJ. O represetante
-- principal é único por cliente (índice parcial).
CREATE TABLE socio_contatos (
    id UUID PRIMARY KEY,
    escritorio_id UUID NOT NULL REFERENCES escritorios(id) ON DELETE CASCADE,
    cliente_id UUID NOT NULL REFERENCES clientes(id) ON DELETE CASCADE,
    papel VARCHAR(8) NOT NULL CHECK (papel IN ('SOCIO', 'CONTATO')),
    nome TEXT NOT NULL,
    cpf_cnpj VARCHAR(14) CHECK (cpf_cnpj ~ '^[0-9]{11}$' OR cpf_cnpj ~ '^[0-9]{14}$'),
    email TEXT,
    telefone TEXT,
    representante_principal BOOLEAN NOT NULL DEFAULT FALSE,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    atualizado_em TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_socio_contatos_cliente ON socio_contatos(cliente_id);
CREATE INDEX idx_socio_contatos_escritorio ON socio_contatos(escritorio_id);
-- Um único representante principal por cliente.
CREATE UNIQUE INDEX uq_representante_principal
    ON socio_contatos(cliente_id) WHERE representante_principal = TRUE;
