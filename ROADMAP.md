# ROADMAP.md — Roadmap de Entregas

> Plataforma **Accurit**. Cada fase está organizada estritamente em três camadas: Banco de Dados, Backend Rust/DDD e Frontend.

**Legenda:** `[ ]` pendente · `[x]` concluído. A marcação deve refletir o estado real do repositório.

## Fase 1 — Infraestrutura e base multi-tenant

### 1. Banco de Dados
- [ ] Configurar PostgreSQL 16, volume persistente, healthcheck e usuário com privilégios mínimos.
- [ ] Criar migrations iniciais para `escritorios`, `usuarios` e tabelas de suporte.
- [ ] Adicionar `escritorios.modo_trial_serpro BOOLEAN NOT NULL DEFAULT TRUE`.
- [ ] Criar índices e constraints de isolamento multi-tenant.

### 2. Backend (Rust / DDD)
- [ ] Preparar camadas `domain/`, `application/`, `infrastructure/` e `presentation/`.
- [ ] Configurar Axum, pool Diesel e `diesel_migrations::embed_migrations!`.
- [ ] Executar migrations pendentes automaticamente no startup, com retry limitado para aguardar PostgreSQL.
- [ ] Expor `GET /api/v1/health`.
- [ ] Configurar variáveis via `.env.example`, sem credenciais reais.

### 3. Frontend (Angular / Next.js)
- [ ] Configurar Admin Angular v18+ e Client Angular v18+ com Signals e organização por features.
- [ ] Configurar landing Next.js (App Router, build standalone).
- [ ] Preparar Nginx para `/`, `/admin/`, `/app/` e `/api/`.
- [ ] Validar subida de toda a stack com `docker compose up --build`.

**Aceite:** banco saudável, migrations executadas pelo backend e health check HTTP 200.

## Fase 2 — Autenticação, escritórios e clientes

### 1. Banco de Dados
- [ ] Criar `clientes` com `escritorio_id`, `tipo` (`PF`/`PJ`), `porte_regime`, `cpf_cnpj`, nome/razão social, contatos e timestamps.
- [ ] Restringir `tipo` a `PF` e `PJ`; restringir regimes a `MEI`, `SIMPLES_NACIONAL`, `LUCRO_PRESUMIDO`, `LUCRO_REAL`.
- [ ] Criar `UNIQUE (escritorio_id, cpf_cnpj)` e índices `(escritorio_id, tipo)` e `(escritorio_id, porte_regime)`.
- [ ] Criar tabelas de certificados A1, procurações, tokens e histórico de consultas conforme modelo.

### 2. Backend (Rust / DDD)
- [ ] Implementar login JWT com claims `escritorio_id` e `role`.
- [ ] Aplicar middleware de tenancy e enforcement nos repositórios Diesel.
- [ ] Implementar CRUD de clientes PF/PJ com validação de CPF/CNPJ.
- [ ] Implementar filtros da API: `Todos`, `CPF`, `CNPJ - MEI`, `CNPJ - Simples Nacional`, `CNPJ - Lucro Presumido`, `CNPJ - Lucro Real`.
- [ ] Implementar armazenamento seguro de certificado A1 e gestão de validade.
- [ ] Padronizar erros, logs redigidos, paginação e ordenação.

### 3. Frontend (Angular / Next.js)
- [ ] Implementar login e guards por papel.
- [ ] Criar telas de cadastro, edição, detalhes e listagem de clientes.
- [ ] Disponibilizar todos os filtros tributários obrigatórios e sincronizá-los com query params.
- [ ] Implementar loading, erro, vazio, validação e paginação.
- [ ] Criar páginas institucionais e formulários de lead na landing Next.js.

**Aceite:** cliente PF/PJ pode ser cadastrado e filtrado; usuário não acessa dados de outro escritório.

## Fase 3 — Configuração SERPRO e Modo Trial

### 1. Banco de Dados
- [ ] Garantir `modo_trial_serpro BOOLEAN NOT NULL DEFAULT TRUE` em `escritorios`.
- [ ] Criar trilha de auditoria para alteração do ambiente SERPRO.
- [ ] Persistir metadados do ambiente efetivamente utilizado em cada consulta.

### 2. Backend (Rust / DDD)
- [ ] Criar configuração tipada e separada para Trial e Produção por produto.
- [ ] Configurar base Trial do Integra Contador: `https://gateway.apiserpro.serpro.gov.br/integra-contador-trial/v1`.
- [ ] Configurar base Trial de Consulta Dívida Ativa: `https://gateway.apiserpro.serpro.gov.br/consulta-divida-ativa-trial/api/v1`.
- [ ] Ler token de trial de variável/secret (`SERPRO_TRIAL_BEARER_TOKEN`), sem hardcode no código.
- [ ] Configurar produção com credenciais e mTLS/certificado A1 conforme o contrato SERPRO.
- [ ] Fazer o backend decidir o ambiente por escritório; proibir fallback silencioso entre Trial e Produção.
- [ ] Implementar testes para estado ausente, troca de modo, credencial inválida e indisponibilidade.

### 3. Frontend (Angular / Next.js)
- [ ] Criar toggle “Modo Trial SERPRO” no Admin `/admin`.
- [ ] Gerenciar estado com Angular Signals; exibir ambiente ativo e confirmação antes de alternar para Produção.
- [ ] Restringir alteração a papel autorizado e atualizar a tela após resposta confirmada da API.
- [ ] Mostrar claramente quando resultados vêm do ambiente de demonstração.

**Aceite:** toggle persiste por escritório; chamadas novas usam somente o ambiente resolvido pelo backend.

## Fase 4 — Consulta Dívida Ativa (somente leitura)

### 1. Banco de Dados
- [ ] Criar histórico de consultas com tenant, cliente, endpoint/tipo, ambiente, status, timestamps e referência/protocolo disponível.
- [ ] Definir política de retenção e minimização dos dados fiscais retornados.
- [ ] Criar índices por escritório, cliente, data e status.

### 2. Backend (Rust / DDD)
- [ ] Implementar cliente HTTP para `GET /v1/devedor/{cpfCnpj}`.
- [ ] Implementar cliente HTTP para `GET /v1/inscricao/{numeroInscricao}`.
- [ ] Criar DTOs Serde para resumo por devedor e detalhe de inscrição conforme o contrato oficial.
- [ ] Mapear `numeroInscricao`, `situacaoDescricao`, `valorTotalConsolidadoMoeda`, `nomeDevedor`, `tipoDevedor`, `nomeUnidade` e detalhes da inscrição.
- [ ] Implementar autenticação da API de Dívida Ativa conforme a documentação específica do produto; não assumir que o fluxo do Integra Contador é idêntico.
- [ ] Implementar timeout, rate limiting, correlação, redaction e tratamento de 401/429/5xx.
- [ ] Expor endpoints internos para consulta por CPF/CNPJ e por número de inscrição.
- [ ] Garantir que não existam casos de uso/endpoints para parcelar, negociar, cobrar ou emitir DARF de Dívida Ativa.

### 3. Frontend (Angular / Next.js)
- [ ] Criar busca por CPF/CNPJ e tela de inscrições/pendências.
- [ ] Criar detalhe de inscrição com processo administrativo, receita principal, prescrição e valores retornados.
- [ ] Mostrar data, ambiente e estado da consulta.
- [ ] Exibir mensagens claras para ausência de pendências, erro de integração e dados de trial.
- [ ] Não exibir ações de negociação, parcelamento ou emissão de DARF para Dívida Ativa.

**Aceite:** consulta por devedor e por inscrição funciona no ambiente habilitado e a UI é exclusivamente consultiva.

## Fase 5 — Integra Contador, OAuth2 e procurações e-CAC

### 1. Banco de Dados
- [ ] Criar/ajustar `tokens_serpro` para token cifrado, escopo, expiração, ambiente e timestamps.
- [ ] Criar/ajustar registros de procuração com contribuinte, procurador, código de serviço, validade, status e data de verificação.
- [ ] Registrar chamadas operacionais com tipo de verbo, `idSistema`, `idServico`, ambiente, estado e referência de resposta, minimizando payloads.

### 2. Backend (Rust / DDD)
- [ ] Implementar OAuth2 Client Credentials para `POST https://gateway.apiserpro.serpro.gov.br/token`, com Basic Base64 de `Consumer Key:Consumer Secret`, respeitando formato oficial de body e requisitos mTLS do contrato.
- [ ] Cachear access token com expiração calculada de `expires_in`; invalidar e renovar em 401 com retry limitado.
- [ ] Implementar DTOs `Identificacao`, `PedidoDados` e `IntegraContadorRequest`, preservando `pedidoDados.dados` como string JSON.
- [ ] Implementar adaptador POST para `/Apoiar`, `/Consultar`, `/Declarar`, `/Emitir` e `/Monitorar`.
- [ ] Validar `idSistema`, `idServico`, `versaoSistema` e payload com catálogo/contrato oficial de cada serviço.
- [ ] Implementar checagem `PROCURACOES / OBTERPROCURACAO41` antes de qualquer operação que exija procuração.
- [ ] Validar códigos de referência: PGDASD `00146`, DCTFWeb `00103`, SITFIS `00002`, Caixa Postal `00006`, confirmando-os no catálogo vigente.
- [ ] Negar execução quando procuração estiver ausente, vencida, cancelada ou não verificável.
- [ ] Adotar idempotência e auditoria para operações potencialmente mutáveis; não repetir automaticamente `Declarar`/`Emitir` sem proteção.
- [ ] Cobrir falhas de token, 401, 403, 429, timeout, envelope inválido e erros de negócio.

### 3. Frontend (Angular / Next.js)
- [ ] Criar fluxo de consultas por sistema/serviço habilitado.
- [ ] Exibir estado da autorização/procuração e motivo de bloqueio sem expor segredos.
- [ ] Apresentar status de processamento e resultados de `Monitorar` quando aplicável.
- [ ] Exigir confirmação explícita para ações que transmitam declarações ou emitam documentos.
- [ ] Distinguir falha de autorização, falha de transporte e rejeição de negócio.

**Aceite:** os cinco verbos são encaminhados pelo envelope comum; serviços protegidos são bloqueados antes da chamada operacional quando não houver procuração válida.

## Fase 6 — Qualidade, segurança e produção

### 1. Banco de Dados
- [ ] Testar migrations `up.sql`/`down.sql` em banco limpo e em upgrade.
- [ ] Revisar índices, constraints, retenção e estratégia de backup/restore.
- [ ] Verificar integridade e unicidade por escritório.

### 2. Backend (Rust / DDD)
- [ ] Testes unitários de domínio e testes de integração Axum/Diesel.
- [ ] Testes de isolamento multi-tenant e IDOR em todas as rotas relevantes.
- [ ] Testes com mocks de SERPRO para token, envelopes, procuração, Trial e respostas de Dívida Ativa.
- [ ] `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` e auditoria de dependências.
- [ ] Segurança de segredos, criptografia de certificado/token, CORS restrito, rate limits e logs redigidos.
- [ ] Validar comportamento quando migration falha e quando há múltiplas réplicas.
- [ ] Configurar observabilidade, request-id e métricas sem PII.

### 3. Frontend (Angular / Next.js)
- [ ] Testes unitários de Signals, filtros e estado do toggle Trial.
- [ ] Testes E2E para login, CRUD de clientes, consulta por devedor/inscrição e verificação de procuração.
- [ ] Verificar acessibilidade, estados de loading/erro/vazio e compatibilidade responsiva.
- [ ] Build de produção Angular e Next.js; checagens de lint e dependências.
- [ ] Confirmar que segredos SERPRO não aparecem no bundle frontend.

**Aceite:** pipeline verde, isolamento multi-tenant validado, migrations automáticas confiáveis e checklist de segurança aprovado.

## Pós-MVP
- [ ] Notificações de validade de certificado e retorno de consultas.
- [ ] Faturamento/recorrência por volume, se aplicável ao modelo comercial.
- [ ] PWA e API para parceiros com autenticação e limites por escritório.
