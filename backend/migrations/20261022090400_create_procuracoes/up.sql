CREATE TABLE procuracoes (
    id UUID PRIMARY KEY,
    escritorio_id UUID NOT NULL REFERENCES escritorios(id) ON DELETE CASCADE,
    -- Cliente do escritório ao qual a procuração se aplica (NULL quando o
    -- contribuinte ainda não está cadastrado). Edita o CREATE TABLE durante
    -- o desenvolvimento (política pré-MVP: sem ALTER), FASE A.
    cliente_id UUID REFERENCES clientes(id) ON DELETE SET NULL,
    ambiente VARCHAR(16) NOT NULL CHECK (ambiente IN ('TRIAL', 'PRODUCAO')),
    contribuinte_cpf_cnpj VARCHAR(14) NOT NULL,
    procurador_cpf_cnpj VARCHAR(14) NOT NULL,
    -- Código do serviço de procuração (ex.: 00146). NULL = registro agregado
    -- da OBTERPROCURACAO41 enquanto o formato de resposta não for confirmado
    -- por teste live (P7) — a correspondência por serviço fica fail-closed.
    codigo_servico VARCHAR(5),
    status VARCHAR(32) NOT NULL,
    valido_de TIMESTAMPTZ,
    valido_ate TIMESTAMPTZ,
    verificado_em TIMESTAMPTZ NOT NULL,
    origem VARCHAR(32) NOT NULL DEFAULT 'OBTERPROCURACAO41',
    -- Conteúdo tolerante da resposta (campos crus, sem interpretação
    -- contratual) para inspeção e auditoria.
    payload_redigido JSONB,
    UNIQUE NULLS NOT DISTINCT (escritorio_id, ambiente, contribuinte_cpf_cnpj, procurador_cpf_cnpj, codigo_servico)
);
CREATE INDEX idx_procuracoes_escritorio_servico
    ON procuracoes(escritorio_id, ambiente, codigo_servico);
CREATE INDEX idx_procuracoes_cliente
    ON procuracoes(escritorio_id, cliente_id, verificado_em DESC);
CREATE INDEX idx_procuracoes_vigencia
    ON procuracoes(ambiente, contribuinte_cpf_cnpj, procurador_cpf_cnpj, codigo_servico, status, verificado_em DESC);
