CREATE TABLE templates_notificacao (
    id UUID PRIMARY KEY,
    tipo VARCHAR(32) NOT NULL UNIQUE CHECK (tipo IN (
        'pendencia_divida_ativa',
        'mensagem_caixa_postal',
        'procuracao_vencendo',
        'certificado_a1_vencendo',
        'das_vencendo',
        'falha_rotina'
    )),
    assunto VARCHAR(255) NOT NULL,
    corpo_texto TEXT NOT NULL,
    corpo_html TEXT,
    variaveis JSONB NOT NULL DEFAULT '[]'::jsonb,
    ativo BOOLEAN NOT NULL DEFAULT TRUE,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    atualizado_em TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_templates_notificacao_tipo ON templates_notificacao(tipo);
