# ADR-0014 — Emissão de guias (DAS/DARF) pelo Integra Contador

Status: aceito

## Contexto

A Fase D introduz a emissão de guias fiscais (DAS e DARF) pelo Integra Contador, utilizando o verbo `Emitir`. Até aqui, o Integra Contador era utilizado apenas para consultas (verbos `Consultar`, `Apoiar`, `Monitorar`). A emissão de guias é uma operação mutável que gera documentos fiscais com valor financeiro, portanto exige:

- Idempotência rigorosa (não emitir a mesma guia duas vezes);
- Auditoria completa (quem, quando, qual cliente, qual competência, qual valor);
- Armazenamento seguro do PDF gerado com retenção definida;
- Isolamento por `escritorio_id` (multi-tenant).

A Dívida Ativa permanece exclusivamente consultiva (ADR-0002 / D4): não existem endpoints para parcelar, negociar, cobrar ou emitir DARF de dívida ativa.

## Decisão

### 1. Autorização de emissão via Integra Contador

O verbo `Emitir` do Integra Contador é autorizado para emissão de guias DAS e DARF, desde que:

- O serviço esteja confirmado na tabela `catalogo_servicos_integra` (ADR-0013) com `versao_confirmada = true` e `habilitado = true`;
- A versão do sistema (`versaoSistema`) esteja confirmada (P4);
- O cliente (contribuinte) possua procuração válida quando exigida (P7);
- A emissão seja idempotente por (escritorio_id, cliente_id, tipo_guia, competencia).

### 2. Idempotência

Cada emissão gera uma chave de idempotência derivada de `(escritorio_id, cliente_id, tipo_guia, competencia)`. Tentativas repetidas com a mesma chave retornam a mesma guia previamente emitida, sem nova chamada ao SERPRO.

### 3. Armazenamento e retenção

O PDF da guia é armazenado de forma segura (armazenamento cifrado em repouso, acesso apenas por usuários autenticados do escritório proprietário). A retenção segue a política de 90 dias (SPEC §9.11), configurável por ambiente.

### 4. Auditoria

Toda emissão é auditada em `auditoria_eventos` com a ação `GUIA_EMITIDA`, registrando `usuario_id`, `escritorio_id`, `cliente_id`, `tipo_guia`, `competencia` e `valor_total`.

### 5. Dívida Ativa permanece consultiva

Não existem casos de uso nem endpoints para emissão de guias da Dívida Ativa. Qualquer tentativa de emissão de guia de Dívida Ativa retorna erro explícito.

## Consequências

- Positivo: escritórios podem emitir e baixar guias DAS/DARF com rastreabilidade completa.
- Negativo: aumenta a superfície de operações mutáveis; exige validação rigorosa de procuração e idempotência para evitar emissões duplicadas.
- Serviços adicionais do Integra Contador (além de `OBTERPROCURACAO41` e serviços de emissão confirmados) permanecem `AguardandoConfirmacao` até confirmação de `versaoSistema` (P4).

## Referências

- [ADR-0002](ADR/0002-escopo-consultivo-divida-ativa.md) — D4, escopo consultivo da Dívida Ativa.
- [ADR-0013](ADR/0013-catalogo-configuravel-servicos-integra.md) — Catálogo configurável.
- [SPEC §5.4](SPEC.md) — Integra Contador.
- [SPEC §9.11](SPEC.md) — Retenção e minimização de dados.
