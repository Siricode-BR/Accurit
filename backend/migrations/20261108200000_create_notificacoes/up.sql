CREATE TABLE notificacoes (
    id UUID PRIMARY KEY,
    usuario_id UUID NOT NULL REFERENCES usuarios(id) ON DELETE CASCADE,
    escritorio_id UUID NOT NULL REFERENCES escritorios(id) ON DELETE CASCADE,
    canal VARCHAR(16) NOT NULL CHECK (canal IN ('in_app', 'email', 'whatsapp')),
    tipo VARCHAR(32) NOT NULL CHECK (tipo IN (
        'pendencia_divida_ativa',
        'mensagem_caixa_postal',
        'procuracao_vencendo',
        'certificado_a1_vencendo',
        'das_vencendo',
        'falha_rotina'
    )),
    titulo VARCHAR(255) NOT NULL,
    corpo TEXT NOT NULL,
    lida BOOLEAN NOT NULL DEFAULT FALSE,
    lida_em TIMESTAMPTZ,
    referencia_tipo VARCHAR(32),
    referencia_id UUID,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_notificacoes_usuario_id ON notificacoes(usuario_id);
CREATE INDEX idx_notificacoes_usuario_nao_lida ON notificacoes(usuario_id, lida) WHERE lida = FALSE;
CREATE INDEX idx_notificacoes_escritorio_id ON notificacoes(escritorio_id);
CREATE INDEX idx_notificacoes_criado_em ON notificacoes(criado_em DESC);
