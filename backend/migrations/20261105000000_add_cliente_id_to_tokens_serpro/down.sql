-- Rollback: remove cliente_id e restaura constraint única original

DROP INDEX IF EXISTS uq_tokens_serpro_autenticar_procurador;
DROP INDEX IF EXISTS uq_tokens_serpro_outros;
DROP INDEX IF EXISTS idx_tokens_serpro_cliente_id;

ALTER TABLE tokens_serpro
    DROP COLUMN IF EXISTS cliente_id;

-- Restaura constraint única original
ALTER TABLE tokens_serpro
    ADD CONSTRAINT tokens_serpro_escritorio_id_produto_ambiente_tipo_token_autor_pedido_dados_key
    UNIQUE (escritorio_id, produto, ambiente, tipo_token, autor_pedido_dados);