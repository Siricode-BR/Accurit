# ROADMAP.md — Roadmap de Entregas da Accurit

> Plataforma **Accurit** — SaaS B2B multi-tenant de gestão contábil, compliance e consultas fiscais SERPRO. Cada fase está organizada estritamente em três camadas: Banco de Dados, Backend Rust/DDD e Frontend.

**Legenda:** `[ ]` pendente · `[x]` concluído · `[~]` em andamento. A marcação deve refletir o estado real do repositório.

**Estado atual:** a Fase 0 (reconciliação documental e configuração) foi concluída. Nenhum item de implementação das fases seguintes foi iniciado.

---

## Fase 0 — Reconciliação documental e configuração

- [x] Corrigir contratos oficiais da Dívida Ativa e do Integra Contador em `README`, `SPEC` e ADRs.
- [x] Separar autenticação OAuth2 (Dívida Ativa) da autenticação SAPI (Integra Contador em produção).
- [x] Registrar D1–D5 e pendências de confirmação em `docs/ADR/0007-modelo-contratacao-serpro.md` e `docs/DECISOES-PENDENTES.md`.
- [x] Atualizar `.env.example`, `docker-compose.yml`, `nginx/nginx.conf`, `.gitattributes` e renomear `backend/Diesel.toml` para `backend/diesel.toml`.
- [x] Validar referências, encoding e ausência de segredos; emitir commits da fase.

---

## Fase 1 — Fundação, infraestrutura e base multi-tenant

### 1. Banco de Dados
- [ ] Configurar PostgreSQL 16 (`postgres:16-alpine`), volume persistente, healthcheck e usuário de aplicação com privilégios mínimos.
- [ ] Criar migrations iniciais para `escritorios`, `usuarios` e tabelas de suporte.
- [ ] Adicionar `escritorios.modo_trial_serpro BOOLEAN NOT NULL DEFAULT TRUE`.
- [ ] Criar índices e constraints de isolamento por `escritorio_id`.
- [ ] Criar migration de `auditoria_eventos` para ações administrativas.

### 2. Backend (Rust / DDD)
- [ ] Preparar as camadas `domain/`, `application/`, `infrastructure/` e `presentation/` com regras de dependência.
- [ ] Configurar Axum 0.8, Tokio, pool Diesel 2.2+ e `diesel_migrations::embed_migrations!`.
- [ ] Executar migrations automaticamente no startup, com retry/backoff e advisory lock para múltiplas réplicas.
- [ ] Expor `GET /api/v1/health` com status do banco.
- [ ] Configurar variáveis via `.env.example`, `tracing`/`request-id` e sem credenciais reais.
- [ ] Implementar middlewares de request-id, CORS restrito e rate limit por escritório.

### 3. Frontend (Angular / Next.js)
- [ ] Configurar Admin Angular 21 e Client Angular 21 com Signals, Standalone Components e organização por features.
- [ ] Configurar o app Next.js 16 (`web`) com App Router, SSG/SSR e build standalone.
- [ ] Preparar Nginx para `/`, `/admin/`, `/app/`, `/login` e `/api/`.
- [ ] Validar a subida completa com `docker compose up --build`.

**Aceite:** banco saudável, migrations executadas pelo backend sem `diesel_cli`, health check HTTP 200 e todas as rotas públicas respondendo pelo proxy.

---

## Fase 2 — Autenticação, escritórios e clientes

### 1. Banco de Dados
- [ ] Criar `clientes` com `escritorio_id`, `tipo` (`PF`/`PJ`), `porte_regime`, `cpf_cnpj`, nome/razão social, contatos e timestamps.
- [ ] Restringir `tipo` a `PF`/`PJ` e regimes a `MEI`, `SIMPLES_NACIONAL`, `LUCRO_PRESUMIDO`, `LUCRO_REAL`.
- [ ] Criar `UNIQUE (escritorio_id, cpf_cnpj)` e índices `(escritorio_id, tipo)` e `(escritorio_id, porte_regime)`.
- [ ] Criar `certificados_a1`, `credenciais_serpro`, `procuracoes`, `tokens_serpro` e `consultas_fiscais` conforme o modelo.
- [ ] Adicionar `usuarios.cpf VARCHAR(11) NOT NULL` com CHECK de 11 dígitos e `UNIQUE (escritorio_id, cpf)`.
- [ ] Definir política de retenção e minimização de dados fiscais.

### 2. Backend (Rust / DDD)
- [ ] Implementar login JWT com claims `escritorio_id`, `usuario_id` e `role`.
- [ ] Aplicar middleware de tenancy e enforcement nos repositórios Diesel.
- [ ] Implementar CRUD de clientes PF/PJ com validação de dígitos verificadores de CPF/CNPJ.
- [ ] Implementar filtros da API: `Todos`, `CPF`, `CNPJ - MEI`, `CNPJ - Simples Nacional`, `CNPJ - Lucro Presumido`, `CNPJ - Lucro Real`.
- [ ] Implementar armazenamento cifrado (AES-256-GCM) e gestão de validade do certificado A1.
- [ ] Padronizar erros, paginação, ordenação, logs redigidos e auditoria.
- [ ] Cobrir IDOR e isolamento multi-tenant com testes.

### 3. Frontend (Angular / Next.js)
- [ ] Implementar login, guards por papel e tratamento consistente de 401/403.
- [ ] Criar telas de cadastro, edição, detalhes e listagem de clientes.
- [ ] Disponibilizar todos os filtros tributários e sincronizá-los com query params.
- [ ] Implementar estados de loading, erro, vazio, validação e paginação.
- [ ] Criar páginas institucionais e formulários de lead no app Next.js (`web`).

**Aceite:** cliente PF/PJ cadastrado, filtrado e editado com isolamento por escritório; usuário nunca acessa dados de outro tenant.

---

## Fase 3 — Configuração SERPRO e Modo Trial

### 1. Banco de Dados
- [ ] Garantir `modo_trial_serpro BOOLEAN NOT NULL DEFAULT TRUE` em `escritorios`.
- [ ] Criar trilha de auditoria para alteração do ambiente SERPRO (`quem`, `quando`, valor anterior/novo).
- [ ] Persistir o ambiente efetivamente utilizado em cada consulta (`consultas_fiscais.ambiente`).
- [ ] Criar constraints para `tokens_serpro` por produto/ambiente.

### 2. Backend (Rust / DDD)
- [ ] Criar configuração tipada e separada para Trial e Produção por produto.
- [ ] Configurar base Trial do Integra Contador: `https://gateway.apiserpro.serpro.gov.br/integra-contador-trial/v1`.
- [ ] Configurar base Trial de Consulta Dívida Ativa: `https://gateway.apiserpro.serpro.gov.br/consulta-divida-ativa-trial/api/v1`.
- [ ] Ler a chave Bearer de trial de `SERPRO_TRIAL_BEARER_TOKEN`, sem hardcode no código ou no bundle.
- [ ] Configurar produção do Integra Contador com SAPI, mTLS e e-CNPJ; configurar a produção da Dívida Ativa como bloqueada por `SERPRO_PROD_NOT_CONFIGURED`.
- [ ] Fazer o backend decidir o ambiente por escritório; proibir fallback silencioso entre Trial e Produção.
- [ ] Implementar `SerproCredentialProvider` e `credenciais_serpro` por escritório, com variáveis globais apenas como fallback de desenvolvimento.
- [ ] Implementar testes para estado ausente, troca de modo, credencial inválida e indisponibilidade.

### 3. Frontend (Angular / Next.js)
- [ ] Criar toggle “Modo Trial SERPRO” no Admin `/admin` com Angular Signals.
- [ ] Exibir ambiente ativo, aviso de demonstração e confirmação antes de alternar para Produção.
- [ ] Restringir a alteração a papel autorizado e atualizar a tela somente após resposta confirmada da API.
- [ ] Marcar visualmente resultados e consultas originados do ambiente Trial.

**Aceite:** toggle persiste por escritório, alteração é auditada e chamadas novas usam exclusivamente o ambiente resolvido pelo backend.

---

## Fase 4 — Consulta Dívida Ativa (somente leitura)

### 1. Banco de Dados
- [ ] Criar histórico de consultas com tenant, cliente, endpoint/tipo, ambiente, status, timestamps e protocolo/referência.
- [ ] Adicionar `fonte_consulta` (`CPF_CNPJ`/`CODIGO_CONTRIBUINTE`) para rastrear o fallback.
- [ ] Criar índices por `escritorio_id`, cliente, data e status.
- [ ] Definir rotina de expurgo conforme a política de retenção.

### 2. Backend (Rust / DDD)
- [ ] Implementar cliente OAuth2 (Client Credentials) e cache de token com renovação em HTTP 401.
- [ ] Implementar cliente HTTP para `GET {base}/devedor/{cpfCnpj}` (base termina em `/api/v1`).
- [ ] Implementar cliente HTTP para `GET {base}/inscricao/{numeroInscricao}`.
- [ ] Enviar `x-cpf-usuario` somente quando `SERPRO_OPERATOR_HEADER_ENABLED=true`, a partir de `usuarios.cpf`.
- [ ] Criar DTOs Serde para resumo por devedor (objeto plano ou lista, via `untagged`) e detalhe de inscrição conforme o contrato oficial.
- [ ] Mapear `numeroInscricao`, `situacaoDescricao`, `valorTotalConsolidadoMoeda`, `nomeDevedor`, `tipoDevedor`, `nomeUnidade`, `numeroPFNResponsavel`, `nomePFNResponsavel`, `numeroPFNInscricao`, `nomePFNInscricao` e `indicadorPrescricaoSV8` com renames explícitos.
- [ ] Higienizar CPF/CNPJ e valores monetários antes da serialização, com `parse_moeda_ptbr_estrito` para respostas da API.
- [ ] Manter o fallback por Código de Contribuinte desabilitado por padrão; implementar `ResolvedorChaveAlternativa` para habilitá-lo com `DIVIDA_ATIVA_FALLBACK_CODIGO_CONTRIBUINTE=true`.
- [ ] Implementar timeout, rate limiting, correlação, redaction e tratamento de 401/404/429/5xx.
- [ ] Bloquear a produção da Dívida Ativa com `SERPRO_PROD_NOT_CONFIGURED` até confirmação da URL.
- [ ] Expor endpoints internos de consulta por CPF/CNPJ e por número de inscrição.
- [ ] Garantir que não existam casos de uso/endpoints para parcelar, negociar, cobrar ou emitir DARF de dívida ativa.

### 3. Frontend (Angular / Next.js)
- [ ] Criar busca por CPF/CNPJ e tela de inscrições/pendências.
- [ ] Criar detalhe de inscrição com processo administrativo, receita principal, prescrição e valores retornados.
- [ ] Exibir data, ambiente e estado da consulta, incluindo a fonte do fallback.
- [ ] Exibir mensagens claras para ausência de pendências, erro de integração e dados de trial.
- [ ] Não exibir ações de negociação, parcelamento ou emissão de DARF.

**Aceite:** consulta por devedor e por inscrição funciona no ambiente habilitado, o fallback por Código de Contribuinte permanece desabilitado por padrão e, quando habilitado, é rastreável; a UI é exclusivamente consultiva.

---

## Fase 5 — Integra Contador, procurações e-CAC e assinatura XML

### 1. Banco de Dados
- [ ] Ajustar `tokens_serpro` para token cifrado, escopo, expiração, ambiente, `tipo_token` (`ACCESS`/`JWT`/`AUTENTICAR_PROCURADOR`), `autor_pedido_dados` e timestamps.
- [ ] Ajustar `procuracoes` com ambiente, contribuinte, procurador, código de serviço, validade, status e data de verificação.
- [ ] Registrar chamadas operacionais com verbo, `idSistema`, `idServico`, ambiente, estado e referência de resposta.
- [ ] Criar tabela/chave de idempotência para `Declarar` e `Emitir`.
- [ ] Auditar alterações de configuração e execuções potencialmente mutáveis.

### 2. Backend (Rust / DDD)
- [ ] Implementar OAuth2 Client Credentials da Dívida Ativa em `POST https://gateway.apiserpro.serpro.gov.br/token` com Basic Base64 e requisitos mTLS do contrato.
- [ ] Implementar autenticação do Integra Contador em produção via SAPI (`https://autenticacao.sapi.serpro.gov.br/authenticate`), com mTLS e-CNPJ, `Role-Type: TERCEIROS`, `Content-Type: application/x-www-form-urlencoded`, `access_token` e `jwt_token`.
- [ ] Cachear `access_token`/`jwt_token` com expiração calculada de `expires_in`; invalidar e renovar em 401 com retry limitado.
- [ ] Implementar obtenção/renovação do `autenticar_procurador_token` (via `ENVIOXMLASSINADO81`), com validade até a meia-noite seguinte.
- [ ] Implementar DTOs `Identificacao`, `PedidoDados`, `IntegraContadorRequest` e envelope de resposta, preservando `pedidoDados.dados` como string JSON.
- [ ] Implementar adaptador POST para `/Apoiar`, `/Consultar`, `/Declarar`, `/Emitir` e `/Monitorar`.
- [ ] Validar `idSistema`, `idServico`, `versaoSistema` e payload contra o catálogo/contrato oficial de cada serviço.
- [ ] Suportar os serviços de referência: `SICALC CONSULTAAPOIORECEITAS52`, `PROCURACOES OBTERPROCURACAO41`, `SITFIS RELATORIOSITFIS92` (**Emitir**), `PGDASD CONSDECLARACAO13`, `PGDASD TRANSDECLARACAO11`, `DCTFWEB GERARGUIA31`, `CAIXAPOSTAL INNOVAMSG63` e `AUTENTICAPROCURADOR ENVIOXMLASSINADO81`.
- [ ] Enviar `x-cpf-usuario` somente quando `SERPRO_OPERATOR_HEADER_ENABLED=true`.
- [ ] Implementar checagem `PROCURACOES / OBTERPROCURACAO41` antes de operações protegidas, **somente quando `autorPedidoDados` difere do contribuinte**.
- [ ] Modelar `RegraProcuracao` (`NaoSeAplica`, `AnyOf`, `Condicional`) e validar os códigos PGDASD `00146`, DCTFWeb `00103`, SITFIS `00002`, Caixa Postal `00006`, DTE `00050`, PAGTOWEB `00004`, REGIMEAPURACAO `00060`, DASNSIMEI `00229` e EPROCESSO `00051`, confirmando-os no catálogo vigente.
- [ ] Negar execução quando a procuração estiver ausente, vencida, cancelada ou não verificável; tratar `403 / AcessoNegado-ICGERENCIADOR-022` como backstop.
- [ ] Implementar assinatura XMLDSig com `<Signature>` e `<Transform>` no namespace default (sem prefixo `ds:`), validação de digest e cadeia ICP-Brasil.
- [ ] Cifrar o certificado A1 em repouso e decifrá-lo apenas em memória, com `zeroize` após o uso.
- [ ] Adotar idempotência e auditoria para operações mutáveis; não repetir `Declarar`/`Emitir` automaticamente sem proteção.
- [ ] Cobrir falhas de token, 401, 403, 429, timeout, envelope inválido, XML inválido e erros de negócio.
- [ ] Tratar HTTP 504 como estado indeterminado: não reenviar automaticamente; consultar o status antes de repetir.
- [ ] Tratar `202` (`tempoEspera`), `204` (`eTag`) e `304` (`etag: autenticar_procurador_token:<token>` + `expires`) com polling controlado e sem retry automático para `Declarar`/`Emitir`.
- [ ] Não bilhetar `204`, `304`, `400`, `401`, `404`, `429`, `500`, `503`, `504`, `/Apoiar` e `/Monitorar`.

### 3. Frontend (Angular / Next.js)
- [ ] Criar fluxo de consultas por sistema/serviço habilitado.
- [ ] Exibir estado da autorização/procuração e motivo de bloqueio sem expor segredos.
- [ ] Apresentar status de processamento e resultados de `Monitorar` quando aplicável.
- [ ] Exigir confirmação explícita para ações que transmitam declarações ou emitam documentos.
- [ ] Distinguir falha de autorização, falha de transporte e rejeição de negócio.
- [ ] Exibir feedback de assinatura/validação do certificado sem expor dados sensíveis.

**Aceite:** os cinco verbos são encaminhados pelo envelope comum, `x-cpf-usuario` é enviado quando habilitado, serviços protegidos são bloqueados antes da chamada quando não há procuração válida (e o autor difere do contribuinte) e o XML assinado passa na validação ICP-Brasil.

---

## Fase 6 — Qualidade, CI/CD e produção

### 1. Banco de Dados
- [ ] Testar migrations `up.sql`/`down.sql` em banco limpo e em upgrade.
- [ ] Revisar índices, constraints, retenção e estratégia de backup/restore.
- [ ] Verificar integridade, unicidade e isolamento por escritório.
- [ ] Validar execução concorrente de migrations com múltiplas réplicas.

### 2. Backend (Rust / DDD)
- [ ] Testes unitários de domínio e testes de integração Axum/Diesel.
- [ ] Testes de isolamento multi-tenant e IDOR em todas as rotas relevantes.
- [ ] Testes com mocks SERPRO para token OAuth2, SAPI, envelopes, procuração, Trial, Dívida Ativa, `202`/`204`/`304` e assinatura XML.
- [ ] `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` e `cargo audit`.
- [ ] Hardening de mTLS, ICP-Brasil, rotação de certificados, segredos, CORS restrito, rate limits e logs redigidos.
- [ ] Validar comportamento quando uma migration falha e quando há múltiplas réplicas.
- [ ] Configurar observabilidade, request-id, métricas e alertas sem PII.
- [ ] Pipeline CI/CD com build de imagem, testes e análise de dependências.

### 3. Frontend (Angular / Next.js)
- [ ] Testes unitários de Signals, filtros, guards e estado do toggle Trial.
- [ ] Testes E2E para login, CRUD de clientes, consulta por devedor/inscrição e verificação de procuração.
- [ ] Verificar acessibilidade, estados de loading/erro/vazio e compatibilidade responsiva.
- [ ] Builds de produção Angular e Next.js com checagens de lint e dependências.
- [ ] Confirmar que segredos SERPRO, certificados e tokens não aparecem no bundle frontend.
- [ ] Configurar pipeline CI para lint, testes e build das três aplicações.

**Aceite:** pipeline verde, isolamento multi-tenant validado, migrations automáticas confiáveis, mTLS/ICP-Brasil testados e checklist de segurança aprovado.

---

## Pós-MVP

- [ ] Notificações de validade de certificado A1 e de retorno de consultas.
- [ ] Relatórios gerenciais e exportação auditável por escritório.
- [ ] Faturamento/recorrência por volume, se aplicável ao modelo comercial.
- [ ] PWA e API para parceiros com autenticação e limites por escritório.
- [ ] Painel de observabilidade por tenant (latência, erros SERPRO e consumo de rate limit).
