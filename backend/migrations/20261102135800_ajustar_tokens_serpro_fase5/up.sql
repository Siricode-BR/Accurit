-- Fase 5: Ajustar tokens_serpro para suportar tipo_token AUTENTICAR_PROCURADOR e autor_pedido_dados
-- A tabela já existe com a estrutura correta conforme migration 20261022090300_create_tokens_serpro
-- Verificamos se precisa de ajustes adicionais

-- Adicionar coluna para armazenar o JWT token separado se necessário
-- A estrutura atual já suporta: tipo_token (ACCESS, JWT, AUTENTICAR_PROCURADOR), autor_pedido_dados, token_cifrado, escopo, expira_em

-- Ajustar procuracoes para incluir ambiente e procurador_cpf_cnpj na chave única
-- A migration 20261022090400_create_procuracoes já tem a estrutura correta

-- Criar tabela de idempotência para operações mutáveis (Declarar e Emitir)
CREATE TABLE idempotencia_operacoes (
    id UUID PRIMARY KEY,
    escritorio_id UUID NOT NULL REFERENCES escritorios(id),
    chave_idempotencia VARCHAR(64) NOT NULL,
    verbo VARCHAR(16) NOT NULL CHECK (verbo IN ('DECLARAR', 'EMITIR')),
    id_sistema VARCHAR(32) NOT NULL,
    id_servico VARCHAR(32) NOT NULL,
    ambiente VARCHAR(16) NOT NULL CHECK (ambiente IN ('TRIAL', 'PRODUCAO')),
    payload_hash CHAR(64) NOT NULL,
    resposta_status INTEGER,
    resposta_redigida JSONB,
    processado_em TIMESTAMPTZ,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (escritorio_id, chave_idempotencia)
);
CREATE INDEX idx_idempotencia_escritorio_criado ON idempotencia_operacoes(escritorio_id, criado_em DESC);

-- Criar tabela de auditoria de execuções do Integra Contador
CREATE TABLE auditoria_integra_contador (
    id UUID PRIMARY KEY,
    escritorio_id UUID NOT NULL REFERENCES escritorios(id),
    cliente_id UUID REFERENCES clientes(id),
    verbo VARCHAR(16) NOT NULL CHECK (verbo IN ('APOIAR', 'CONSULTAR', 'DECLARAR', 'EMITIR', 'MONITORAR')),
    id_sistema VARCHAR(32) NOT NULL,
    id_servico VARCHAR(32) NOT NULL,
    versao_sistema VARCHAR(16) NOT NULL,
    ambiente VARCHAR(16) NOT NULL CHECK (ambiente IN ('TRIAL', 'PRODUCAO')),
    autor_pedido_dados VARCHAR(14) NOT NULL,
    contribuinte_cpf_cnpj VARCHAR(14) NOT NULL,
    status_code INTEGER,
    estado VARCHAR(32) NOT NULL,
    protocolo TEXT,
    request_redigido JSONB,
    response_redigido JSONB,
    erro_codigo VARCHAR(64),
    erro_mensagem TEXT,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_auditoria_ic_escritorio_criado ON auditoria_integra_contador(escritorio_id, criado_em DESC);
CREATE INDEX idx_auditoria_ic_cliente ON auditoria_integra_contador(cliente_id);
CREATE INDEX idx_auditoria_ic_servico ON auditoria_integra_contador(id_sistema, id_servico);