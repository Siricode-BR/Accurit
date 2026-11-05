-- Adiciona cliente_id à tabela tokens_serpro para suportar o modelo Software House
-- onde o token JWT do AUTENTICAPROCURADOR é por cliente (contribuinte)

ALTER TABLE tokens_serpro
    ADD COLUMN cliente_id UUID REFERENCES clientes(id);

-- Índice para buscas por cliente
CREATE INDEX idx_tokens_serpro_cliente_id ON tokens_serpro(cliente_id);

-- Atualiza a constraint única para incluir cliente_id quando tipo_token = 'AUTENTICAR_PROCURADOR'
-- Para outros tipos de token, cliente_id pode ser NULL
ALTER TABLE tokens_serpro
    DROP CONSTRAINT tokens_serpro_escritorio_id_produto_ambiente_tipo_token_autor_pedido_dados_key;

-- Nova constraint única parcial: para AUTENTICAR_PROCURADOR, inclui cliente_id
-- Para outros tipos, mantém a constraint original sem cliente_id
CREATE UNIQUE INDEX uq_tokens_serpro_autenticar_procurador
    ON tokens_serpro (escritorio_id, produto, ambiente, tipo_token, autor_pedido_dados, cliente_id)
    WHERE tipo_token = 'AUTENTICAR_PROCURADOR';

CREATE UNIQUE INDEX uq_tokens_serpro_outros
    ON tokens_serpro (escritorio_id, produto, ambiente, tipo_token, autor_pedido_dados)
    WHERE tipo_token != 'AUTENTICAR_PROCURADOR';