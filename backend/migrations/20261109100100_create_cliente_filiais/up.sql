-- Fase E — Gestão de carteira: filiais/unidades de um cliente PJ (ou PF com CNPJ MEI).
-- O CNPJ da filial é único na plataforma inteira (um CNPJ só pode pertencer a uma
-- matriz/filial de um escritório).
CREATE TABLE cliente_filiais (
    id UUID PRIMARY KEY,
    escritorio_id UUID NOT NULL REFERENCES escritorios(id) ON DELETE CASCADE,
    cliente_id UUID NOT NULL REFERENCES clientes(id) ON DELETE CASCADE,
    cnpj VARCHAR(14) NOT NULL CHECK (cnpj ~ '^[0-9]{14}$'),
    nome_unidade TEXT,
    endereco TEXT,
    telefone TEXT,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    atualizado_em TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_cliente_filiais_cliente ON cliente_filiais(cliente_id);
CREATE INDEX idx_cliente_filiais_escritorio ON cliente_filiais(escritorio_id);
CREATE UNIQUE INDEX uq_cliente_filiais_cliente_cnpj ON cliente_filiais(cliente_id, cnpj);
