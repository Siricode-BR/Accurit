CREATE TABLE tokens_serpro (
    id UUID PRIMARY KEY,
    escritorio_id UUID NOT NULL REFERENCES escritorios(id),
    -- Vínculo com o contribuinte (modelo Software House): exigido para o
    -- token do AUTENTICAPROCURADOR (um jwt_token por cliente por escritório
    -- e ambiente); nulo para tokens ACCESS/JWT do contratante SaaS, que não
    -- pertencem a um contribuinte específico.
    cliente_id UUID REFERENCES clientes(id),
    produto VARCHAR(32) NOT NULL CHECK (produto IN ('INTEGRA_CONTADOR', 'DIVIDA_ATIVA')),
    ambiente VARCHAR(16) NOT NULL CHECK (ambiente IN ('TRIAL', 'PRODUCAO')),
    tipo_token VARCHAR(32) NOT NULL CHECK (tipo_token IN ('ACCESS', 'JWT', 'AUTENTICAR_PROCURADOR')),
    autor_pedido_dados VARCHAR(14) NOT NULL DEFAULT '',
    token_cifrado BYTEA NOT NULL,
    escopo TEXT,
    expira_em TIMESTAMPTZ NOT NULL,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- O token de autenticação de procurador é sempre vinculado a um cliente.
    CONSTRAINT tokens_serpro_autenticar_procurador_exige_cliente CHECK (
        tipo_token <> 'AUTENTICAR_PROCURADOR' OR cliente_id IS NOT NULL
    ),
    -- NULLS NOT DISTINCT: tokens de contratante (cliente_id NULL) também
    -- obedecem à unicidade por (escritório, produto, ambiente, tipo, autor).
    CONSTRAINT uq_tokens_serpro UNIQUE NULLS NOT DISTINCT (
        escritorio_id, cliente_id, produto, ambiente, tipo_token, autor_pedido_dados
    )
);
