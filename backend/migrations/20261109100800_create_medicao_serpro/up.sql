-- Fase G — Medição de uso SERPRO (chamadas, agregados, planos, bloqueios).
-- Tabela bruta de chamadas (append-only, não contém payload sensível).
CREATE TABLE chamadas_serpro (
    id UUID PRIMARY KEY,
    escritorio_id UUID NOT NULL REFERENCES escritorios(id) ON DELETE CASCADE,
    produto VARCHAR(32) NOT NULL CHECK (produto IN ('INTEGRA_CONTADOR', 'DIVIDA_ATIVA')),
    verbo VARCHAR(32) NOT NULL,
    servico VARCHAR(64) NULL,
    ambiente VARCHAR(16) NOT NULL CHECK (ambiente IN ('TRIAL', 'PRODUCAO')),
    cliente_id UUID NULL REFERENCES clientes(id) ON DELETE SET NULL,
    usuario_id UUID NULL REFERENCES usuarios(id) ON DELETE SET NULL,
    origem VARCHAR(16) NOT NULL CHECK (origem IN ('manual', 'rotina', 'lote')),
    resultado VARCHAR(16) NOT NULL CHECK (resultado IN ('sucesso', '4xx', '5xx', 'timeout', 'indeterminado')),
    status_http INTEGER NULL,
    duracao_ms INTEGER NOT NULL,
    request_id VARCHAR(64) NOT NULL,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_chamadas_serpro_escritorio ON chamadas_serpro(escritorio_id);
CREATE INDEX idx_chamadas_serpro_criado_em ON chamadas_serpro(criado_em DESC);
CREATE INDEX idx_chamadas_serpro_produto_ambiente ON chamadas_serpro(produto, ambiente);

-- Agregados de consumo por escritório/produto/ambiente/período (atualizados de forma atômica).
CREATE TABLE agregados_consumo_serpro (
    id UUID PRIMARY KEY,
    escritorio_id UUID NOT NULL REFERENCES escritorios(id) ON DELETE CASCADE,
    produto VARCHAR(32) NOT NULL CHECK (produto IN ('INTEGRA_CONTADOR', 'DIVIDA_ATIVA')),
    ambiente VARCHAR(16) NOT NULL CHECK (ambiente IN ('TRIAL', 'PRODUCAO')),
    periodo VARCHAR(16) NOT NULL CHECK (periodo IN ('dia', 'mes')),
    referencia_periodo DATE NOT NULL,
    total_chamadas INTEGER NOT NULL DEFAULT 0,
    chamadas_sucesso INTEGER NOT NULL DEFAULT 0,
    chamadas_4xx INTEGER NOT NULL DEFAULT 0,
    chamadas_5xx INTEGER NOT NULL DEFAULT 0,
    chamadas_timeout INTEGER NOT NULL DEFAULT 0,
    chamadas_indeterminado INTEGER NOT NULL DEFAULT 0,
    duracao_total_ms BIGINT NOT NULL DEFAULT 0,
    atualizado_em TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (escritorio_id, produto, ambiente, periodo, referencia_periodo)
);
CREATE INDEX idx_agregados_consumo_escritorio ON agregados_consumo_serpro(escritorio_id);
CREATE INDEX idx_agregados_consumo_referencia ON agregados_consumo_serpro(referencia_periodo);

-- Planos de consumo por escritório.
CREATE TABLE planos_escritorio (
    id UUID PRIMARY KEY,
    nome VARCHAR(64) NOT NULL,
    descricao TEXT NULL,
    chamadas_mes INTEGER NOT NULL,
    preco_mensal_cents INTEGER NOT NULL DEFAULT 0,
    ativo BOOL NOT NULL DEFAULT TRUE,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    atualizado_em TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Assinatura de cada escritório (qual plano está ativo e qual limite de chamadas/mês).
CREATE TABLE assinaturas_escritorio (
    id UUID PRIMARY KEY,
    escritorio_id UUID NOT NULL UNIQUE REFERENCES escritorios(id) ON DELETE CASCADE,
    plano_id UUID NOT NULL REFERENCES planos_escritorio(id) ON DELETE CASCADE,
    chamadas_mes_limite INTEGER NOT NULL,
    bloqueio_ao_exceder BOOL NOT NULL DEFAULT TRUE,
    ativo BOOL NOT NULL DEFAULT TRUE,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    atualizado_em TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Chamadas bloqueadas localmente (não contam como chamada SERPRO, registradas apenas para diagnóstico).
CREATE TABLE bloqueios_serpro (
    id UUID PRIMARY KEY,
    escritorio_id UUID NOT NULL REFERENCES escritorios(id) ON DELETE CASCADE,
    produto VARCHAR(32) NOT NULL CHECK (produto IN ('INTEGRA_CONTADOR', 'DIVIDA_ATIVA')),
    motivo VARCHAR(64) NOT NULL CHECK (motivo IN ('rate_limit_interno', 'producao_nao_configurada', 'parametro_invalido')),
    contexto TEXT NULL,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_bloqueios_serpro_escritorio ON bloqueios_serpro(escritorio_id);
CREATE INDEX idx_bloqueios_serpro_criado_em ON bloqueios_serpro(criado_em DESC);
