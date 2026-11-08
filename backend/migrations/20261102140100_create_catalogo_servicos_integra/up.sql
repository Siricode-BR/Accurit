CREATE TABLE catalogo_servicos_integra (
    id UUID PRIMARY KEY,
    id_sistema VARCHAR(32) NOT NULL,
    id_servico VARCHAR(32) NOT NULL,
    verbo VARCHAR(16) NOT NULL CHECK (verbo IN ('Apoiar', 'Consultar', 'Declarar', 'Emitir', 'Monitorar')),
    -- versaoSistema confirmado no catálogo oficial; NULL = versão pendente
    -- de confirmação (P4) e, portanto, habilitado = FALSE (fail-closed).
    versao_sistema VARCHAR(16),
    versao_confirmada BOOLEAN NOT NULL DEFAULT FALSE,
    habilitado BOOLEAN NOT NULL DEFAULT FALSE,
    -- Restrição de escopo: com TRUE, apenas o verbo Consultar é viável
    -- (Integra-Parcelamentos, decisão do projeto — ADR-0013).
    apenas_consulta BOOLEAN NOT NULL DEFAULT FALSE,
    requer_procuracao BOOLEAN NOT NULL DEFAULT FALSE,
    codigos_procuracao TEXT[] NOT NULL DEFAULT '{}',
    descricao VARCHAR(255) NOT NULL,
    atualizado_em TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (id_sistema, id_servico)
);
CREATE INDEX idx_catalogo_servicos_integra_busca
    ON catalogo_servicos_integra(id_sistema, id_servico);

-- Catálogo semeado em desenvolvimento (editável enquanto o MVP não fecha).
-- Fontes: catalogo-de-servicos.md e servicos-x-procuracoes.md oficiais
-- (capturados em 2026-10); habilitados apenas serviços com versão
-- confirmada. Os demais aguardam P4.

INSERT INTO catalogo_servicos_integra
    (id, id_sistema, id_servico, verbo, versao_sistema, versao_confirmada,
     habilitado, apenas_consulta, requer_procuracao, codigos_procuracao, descricao)
VALUES
    (gen_random_uuid(), 'PROCURACOES', 'OBTERPROCURACAO41', 'Consultar', '1', TRUE,
     TRUE, FALSE, FALSE, '{}', 'Obter procurações eletrônicas do e-CAC (Integra-Procurações).'),
    (gen_random_uuid(), 'AUTENTICAPROCURADOR', 'ENVIOXMLASSINADO81', 'Apoiar', '1.0', TRUE,
     FALSE, FALSE, FALSE, '{}', 'Envio de XML assinado do Termo de Autorização (uso interno do fluxo AUTENTICAPROCURADOR).'),
    (gen_random_uuid(), 'SITFIS', 'RELATORIOSITFIS92', 'Emitir', NULL, FALSE,
     FALSE, FALSE, TRUE, '{00002}', 'Relatório de Situação Fiscal (Integra-SITFIS).'),
    (gen_random_uuid(), 'SITFIS', 'SOLICITARPROTOCOLO91', 'Apoiar', NULL, FALSE,
     FALSE, FALSE, TRUE, '{00002}', 'Solicitação de protocolo da Situação Fiscal (Integra-SITFIS).'),
    (gen_random_uuid(), 'PGDASD', 'CONSDECLARACAO13', 'Consultar', NULL, FALSE,
     FALSE, FALSE, TRUE, '{00146}', 'Consulta de declaração PGDAS-D (Integra-SN).'),
    (gen_random_uuid(), 'PGDASD', 'GERARDAS12', 'Emitir', NULL, FALSE,
     FALSE, FALSE, TRUE, '{00146}', 'Emissão de guia DAS (Integra-SN; sujeita ao ADR de emissão de guias).'),
    (gen_random_uuid(), 'CAIXAPOSTAL', 'INNOVAMSG63', 'Monitorar', NULL, FALSE,
     FALSE, FALSE, TRUE, '{00006}', 'Caixa Postal — novas mensagens (Integra-CaixaPostal).'),
    (gen_random_uuid(), 'CAIXAPOSTAL', 'MSGCONTRIBUINTE61', 'Consultar', NULL, FALSE,
     FALSE, FALSE, TRUE, '{00006}', 'Caixa Postal — lista de mensagens do contribuinte.'),
    (gen_random_uuid(), 'CAIXAPOSTAL', 'MSGDETALHAMENTO62', 'Consultar', NULL, FALSE,
     FALSE, FALSE, TRUE, '{00006}', 'Caixa Postal — detalhamento de mensagem.'),
    (gen_random_uuid(), 'PGMEI', 'GERARDASPDF21', 'Emitir', NULL, FALSE,
     FALSE, FALSE, FALSE, '{}', 'Emissão de DAS MEI em PDF (Integra-MEI).'),
    (gen_random_uuid(), 'PGMEI', 'GERARDASCODBARRA22', 'Emitir', NULL, FALSE,
     FALSE, FALSE, FALSE, '{}', 'Emissão de DAS MEI em código de barras (Integra-MEI).'),
    (gen_random_uuid(), 'DCTFWEB', 'GERARGUIA31', 'Emitir', NULL, FALSE,
     FALSE, FALSE, TRUE, '{00103}', 'Geração de guia DCTFWeb (Integra-DCTFWeb).'),
    (gen_random_uuid(), 'PARCSN', 'PEDIDOSPARC163', 'Consultar', NULL, FALSE,
     FALSE, TRUE, TRUE, '{00076,00188}', 'Parcelamentos — pedidos (Integra-Parcelamentos; apenas consulta).'),
    (gen_random_uuid(), 'PARCSN', 'OBTERPARC164', 'Consultar', NULL, FALSE,
     FALSE, TRUE, TRUE, '{00076,00188}', 'Parcelamentos — detalhe do parcelamento (apenas consulta).'),
    (gen_random_uuid(), 'PARCSN', 'PARCELASPARAGERAR162', 'Consultar', NULL, FALSE,
     FALSE, TRUE, TRUE, '{00076,00188}', 'Parcelamentos — parcelas para geração (apenas consulta).'),
    (gen_random_uuid(), 'PARCSN', 'DETPAGTOPARC165', 'Consultar', NULL, FALSE,
     FALSE, TRUE, TRUE, '{00076,00188}', 'Parcelamentos — detalhe de pagamento (apenas consulta).');
