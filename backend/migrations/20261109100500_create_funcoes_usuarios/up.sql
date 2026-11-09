-- Fase E — Gestão de carteira: função granular do usuário dentro do escritório.
-- Linha opcional: sem registro, a função deriva do papel
-- (CLIENTE -> GESTOR; OPERADOR -> ANALISTA_FISCAL).
CREATE TABLE funcoes_usuarios (
    usuario_id UUID PRIMARY KEY REFERENCES usuarios(id) ON DELETE CASCADE,
    escritorio_id UUID NOT NULL REFERENCES escritorios(id) ON DELETE CASCADE,
    funcao VARCHAR(24) NOT NULL CHECK (funcao IN ('GESTOR', 'ANALISTA_FISCAL', 'OPERADOR_LEITURA')),
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    atualizado_em TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_funcoes_usuarios_escritorio ON funcoes_usuarios(escritorio_id);
