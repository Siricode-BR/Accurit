-- Fase F — Relatórios e link seguro de compartilhamento.
-- Relatório gerado por um usuário do escritório (situação fiscal, histórico
-- de consultas, pendências por carteira), exportado em PDF ou XLSX.
CREATE TABLE relatorios (
    id UUID PRIMARY KEY,
    escritorio_id UUID NOT NULL REFERENCES escritorios(id) ON DELETE CASCADE,
    usuario_id UUID NOT NULL REFERENCES usuarios(id) ON DELETE CASCADE,
    cliente_id UUID NULL REFERENCES clientes(id) ON DELETE SET NULL,
    tipo_relatorio VARCHAR(32) NOT NULL CHECK (
        tipo_relatorio IN ('situacao_fiscal', 'historico_consultas', 'pendencias_carteira')
    ),
    formato VARCHAR(16) NOT NULL CHECK (formato IN ('PDF', 'XLSX')),
    status VARCHAR(16) NOT NULL CHECK (status IN ('gerando', 'concluido', 'falhou')),
    conteudo BYTEA NULL,
    nome_arquivo TEXT NULL,
    competencia VARCHAR(7) NULL CHECK (competencia ~ '^\d{4}-\d{2}$'),
    expira_em TIMESTAMPTZ NOT NULL,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    atualizado_em TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_relatorios_escritorio_id ON relatorios(escritorio_id);
CREATE INDEX idx_relatorios_usuario_id ON relatorios(usuario_id);
CREATE INDEX idx_relatorios_status ON relatorios(status);
CREATE INDEX idx_relatorios_criado_em ON relatorios(criado_em DESC);

-- Link seguro e expirável para compartilhar um relatório com o cliente final
-- (somente leitura, sem exigir autenticação no portal).
CREATE TABLE links_compartilhamento_relatorios (
    id UUID PRIMARY KEY,
    relatorio_id UUID NOT NULL REFERENCES relatorios(id) ON DELETE CASCADE,
    token VARCHAR(64) NOT NULL UNIQUE,
    expira_em TIMESTAMPTZ NOT NULL,
    acessado_em TIMESTAMPTZ NULL,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_links_compartilhamento_relatorios_relatorio_id ON links_compartilhamento_relatorios(relatorio_id);
CREATE INDEX idx_links_compartilhamento_relatorios_token ON links_compartilhamento_relatorios(token);
CREATE INDEX idx_links_compartilhamento_relatorios_expira_em ON links_compartilhamento_relatorios(expira_em);
