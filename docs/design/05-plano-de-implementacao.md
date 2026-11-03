# Plano de implementação do design system e nova UI

> **Fase 0 — plano para aprovação.** Nenhum item abaixo autoriza alteração de contrato de API. Execução começa somente após aprovação deste pacote de design.

## Estado atual verificado

- Monorepo contém `admin/`, `client/` e `web/`, mas não há `package.json` de workspace na raiz.
- Admin e Client são aplicações Angular standalone separadas, com Angular 21, Signals, Vitest e Tailwind v4 instalado/importado. Angular Material está instalado e usado diretamente; Angular CDK não consta como dependência direta.
- Web é Next.js App Router 16 com React 19 e Tailwind v4.
- Cada Angular tem seu próprio `package.json`, lockfile e `angular.json`; há budgets de produção já definidos. Nginx publica `/`, `/admin/`, `/app/` e `/api/`; builds e `BASE_HREF` precisam ser preservados.
- As páginas autenticadas ainda não compartilham um AppShell. Há login, clientes, Dívida Ativa, selo de ambiente, lista/provisionamento de escritórios e alternância de Trial; faltam várias telas do mapa.
- A feature Admin de dashboard existe sem rota. A página pública contém CTA que aponta para login Admin como “Acessar plataforma” e rodapé com nome de fase. Esses ajustes pertencem às fases futuras de UI, não a esta entrega documental.
- A API tem contrato legado incompatível em enumeração de tipo de cliente e serialização de certificados; detalhes e proposta de adaptação estão em [UI-CONTRATOS-DIVERGENTES.md](../UI-CONTRATOS-DIVERGENTES.md).

## Stack e estrutura proposta

**Decisão proposta:** Tailwind v4 + Angular CDK são o padrão de construção. Manter Angular Material apenas atrás de componentes wrapper em `libs/ui` enquanto não houver substituto aprovado. A escolha evita introduzir outro sistema visual, usa Tailwind já presente e aproveita CDK para overlays, focus management e acessibilidade. A remoção de Material será incremental, não uma reescrita.

```text
/
├── package.json                  # npm workspaces
├── admin/                        # Angular backoffice, continuar deploy /admin/
├── client/                       # Angular escritório, continuar deploy /app/
├── web/                          # Next institucional, continuar deploy /
└── libs/
    ├── ui/                       # tokens CSS + componentes Angular standalone
    └── core/                     # auth/session/interceptor/API comum parametrizada por audiência
```

- Criar workspace npm de raiz com `admin`, `client`, `web`, `libs/ui` e `libs/core` como workspaces sem migrar para Nx nem substituir Angular CLI.
- `libs/ui` fornece CSS compartilhado para os três apps; a entrada Angular exporta componentes standalone, wrappers de Material/CDK e padrões de acessibilidade.
- `libs/core` compartilha contratos e mecanismo de sessão por audiência (`admin`/`client`) sem misturar tokens nem relaxar guards.
- Next importa somente tokens/estilos compartilhados; componentes Angular não são dependência da aplicação Web.
- Manter builds Docker e contexto atual; adaptar cada Dockerfile para instalar/buildar o workspace na raiz e preservar `BASE_HREF` Angular e paths do Nginx.
- Criar uma página `/design-system` disponível somente em ambiente de desenvolvimento dentro da aplicação Client para demonstrar estados e componentes; não publicar rota de catálogo em produção.
- Componentes Angular novos seguem standalone, Signals, `ChangeDetectionStrategy.OnPush`, camada `data` para `HttpClient`, lazy routes e testes Vitest. Não mover regras de negócio para templates.
- Mappers ficam em `features/<feature>/data/`; até a API ser revisada, eles isolam formatos wire em tipos de domínio estáveis.
- Centralizar textos por feature, com pt-BR como locale padrão e estrutura pronta para tradução; datas usam `dd/MM/yyyy HH:mm` e valores monetários mantêm a string pt-BR fornecida pela API, sem conversão para `float`.

## Sequência, dependências e estimativas

Estimativas preliminares em dias de engenharia, sujeitas a revisão depois da aprovação e análise do build dos workspaces. Não incluem espera por confirmação externa de contrato SERPRO nem APIs novas.

| Etapa | Escopo | Dependências | Estimativa |
|---|---|---|---:|
| 0 — Design e contratos divergentes | Cinco documentos de design e inventário de divergências | Leitura SPEC/ADRs | 2–3 dias |
| 1 — Fundação | npm workspaces; libs/ui e libs/core; tokens; tema/densidade; wrappers; página dev de catálogo; testes; Docker/BaseHref | Aprovação Fase 0 | 6–10 dias |
| 2 — Shells e navegação | AppShell Admin/Client, sidebar/topbar, selo, breadcrumbs, menu mobile, guards/mensagens, command palette, rotas base e links do Web | Etapa 1; manter contratos de auth | 6–9 dias |
| 3A — Login e início | Corrigir copy/rotas de login, Início do escritório, Visão geral Admin | Etapa 2; somente dados já disponíveis | 4–6 dias |
| 3B — Clientes | Lista/chips/busca/URL, formulário, ficha com abas e estados | Etapa 2; mapper de enum/paginação | 6–9 dias |
| 3C — Dívida Ativa | Consulta, inscrição, histórico, status, acessibilidade e preservação do aviso consultivo | Etapa 1; manter contrato SERPRO atual | 4–7 dias |
| 3D — Escritórios Admin | Lista, criação, detalhe, pré-voo, toggle, conflitos e feedback | Etapa 2; manter PATCH/409 existente | 5–8 dias |
| 3E — Certificados e gestão | Certificados A1; depois equipe, administradores, auditoria e saúde, com gates para APIs ausentes | Mappers e contratos de API para cada área | 8–14 dias + backend |
| 4 — QA e estabilização | Testes responsivos/a11y, regressões, Docker Compose, smoke e budgets | Etapas anteriores | 4–7 dias |

Estimativa total indicativa: **45–73 dias de engenharia**, com trabalho de backend/API adicional ainda não dimensionado para equipe, auditoria, detalhe de escritório e trilhas sem endpoint. Com duas pessoas em paralelo e revisão de design, expectativa aproximada de 5–8 semanas após aprovação; dependências externas podem alongar o calendário.

### Entrega por PR

- Um PR para fundação compartilhada; um para shells/navegação.
- Um PR por área de tela em 3A–3E, preservando testes atuais (`inicio`, login, `tenant-list`, `tenants.store`, selo de ambiente).
- Mudanças de contrato de API não entram nesses PRs. Necessidades novas viram proposta e aprovação de contrato antes de implementar frontend dependente.
- Cada PR inclui screenshots/estados, testes unitários, build do app afetado, evidência mobile e checklist de acessibilidade.

## Estratégia de migração

1. Antes de mover telas, adicionar os adaptadores de contrato descritos em [UI-CONTRATOS-DIVERGENTES.md](../UI-CONTRATOS-DIVERGENTES.md), com fixtures de respostas wire reais e testes de serialização.
2. Construir tokens sem alterar a aparência funcional de uma só vez; registrar baseline visual das telas existentes.
3. Criar wrappers de botão, campo, diálogo, tabela e estado para impedir novos usos diretos de Material. Migrar tela antiga quando ela receber mudança, não em codemod global.
4. Implementar os shells respeitando base paths reais do Nginx e mantendo o fluxo atual de sessão por audiência.
5. Migrar páginas na ordem de dependência. Cada página deve possuir loading, vazio, erro, sem permissão e offline antes de aceitar o PR.
6. Não ativar links para features sem API. Esconder item do menu ou mostrar estado de indisponibilidade explicitamente.
7. Validar preferência clara/escura, densidade, 320–1440+ px e reflow; não reduzir budgets ou removê-los para fazer build passar.

## Riscos, bloqueios e mitigação

| Risco/bloqueio | Impacto | Mitigação |
|---|---|---|
| API de equipe, ADMINs, auditoria, detalhe de escritório e saúde analítica ausente | Telas não podem carregar dados reais | Manter item desabilitado/fora do menu até contrato e autorização existirem |
| Enum do tipo de cliente não conversa com os valores usados pela UI | Criação/edição pode falhar no parse do request; resposta não combina com tipo TS | Mapper testado; proposta de correção backend separada |
| Certificados serializados em snake_case e ainda sem feature Angular | Integração tipada falha se assumir camelCase | DTO wire explícito + mapper para modelo de UI |
| Angular Material e estilos locais duplicados | Inconsistência visual e migração extensa | Wrappers de `libs/ui`, migração incremental e snapshot visual |
| Workspaces alteram resolução de dependência e Docker context | Build local/produção pode quebrar | Manter scripts por app, validar clean install, Compose e smoke antes de promover |
| Contrato de produção SERPRO ainda bloqueado | UI não pode prometer ou habilitar produção | Checklist mostra bloqueio; backend permanece autoridade; sem CTA de ativação |
| Contrato/procurações XML ainda incompletos | Estado visual pode implicar autorização falsa | “Não verificável” bloqueia; nenhuma inferência local |
| Tema escuro pode reduzir contraste | Falha WCAG AA | Tokens pareados e teste automatizado de contraste + auditoria manual |
| Ações por papel divergem entre guards e handlers | Usuário vê CTA que API nega | Matriz baseada em handler, teste por papel e resolução explícita antes da tela |
| Auditoria guarda dados fiscais demais | Risco LGPD | Eventos mínimos, mascaramento e política de retenção aprovada antes de lançar |

## Gates de qualidade por fase

- `admin`: `npm test -- --watch=false`, `npm run build`, `npm run lint`.
- `client`: `npm test -- --watch=false`, `npm run build`, `npm run lint`.
- `web`: `npm run lint`, `npm run build`.
- Backend e APIs existentes: testes atuais sem alterar contratos; integração completa quando banco de teste estiver disponível.
- Stack: `docker compose up --build` e `scripts/smoke.*` preservam rotas `/`, `/admin/`, `/app/`, `/api/`.
- Acessibilidade: Lighthouse >= 95 nas páginas principais como meta, auditoria WCAG 2.2 AA e teclado/leitor de tela em fluxos de login, cliente, consulta e troca de ambiente.
- Regressões: selo Trial, troca pessimista, confirmação de produção, conflito `409`, filtros de clientes, consulta e detalhe Dívida Ativa, autenticação por audiência.
- Segurança visual: varredura de bundle/logs para tokens/segredos e revisão de PII; nunca persistir access token no storage.

## Critérios de aceite propostos

- Design e matriz de papéis aprovados; nenhuma tela consultiva sugere negociação ou emissão de DARF de dívida ativa.
- Tokens CSS são compartilhados entre Web/Admin/Client; nenhum novo valor visual duplicado em estilo de feature.
- Navegação principal a até dois cliques e caminhos corretos para as duas audiências.
- Badge de ambiente aparece em telas do escritório e contexto do tenant no Admin.
- Toggle é pessimista, exige confirmação reforçada para Produção, tem pré-voo visível e trata `409` recarregando.
- Estados de loading/vazio/erro/sem permissão/offline são cobertos; erros oferecem `requestId` copiável.
- Acessibilidade WCAG 2.2 AA e fluxo por teclado; Lighthouse >= 95 como meta, não substituindo auditoria manual.
- Builds, testes, Docker Compose, smoke e budgets permanecem verdes em Admin, Client e Web.
- Todos os desacordos de API e owners estão listados em [UI-CONTRATOS-DIVERGENTES.md](../UI-CONTRATOS-DIVERGENTES.md).
