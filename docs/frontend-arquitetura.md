# Arquitetura do Frontend (monorepo Turborepo)

> Status: atualizada para a estrutura `frontend/` pós-refatoração. O backend NÃO faz parte deste monorepo — vive em `backend/` (repositório/container próprio, Rust/Axum) e é consumido via HTTP.
>
> Nota de nomenclatura: os pacotes pedidos como `@monorepo/core` e `@monorepo/ui` são publicados neste workspace como **`@accurit/core`** e **`@accurit/ui`** (escopo `@accurit`); o nome do workspace Turborepo é `accurit-frontend`.

## 1. Estrutura

```
frontend/
├─ apps/
│  ├─ web/      # Next.js 16 (App Router)  → landing institucional + leads
│  ├─ admin/    # Angular 22 (Signals)     → backoffice SaaS (tenants)
│  └─ client/   # Angular 22 (Signals)     → portal do escritório contábil
├─ packages/
│  ├─ ts-config/     → @accurit/ts-config  (extends de tsconfig)
│  ├─ shared/        → @accurit/shared     (validação/higienização CPF/CNPJ, e-mail, formatters)
│  ├─ core/          → @accurit/core       (DTOs/contratos espelhando o OpenAPI do backend)
│  ├─ ui/            → @accurit/ui         (design system Angular + tokens.css)
│  └─ eslint-config/ → @repo/eslint-config
├─ turbo.json
└─ pnpm-workspace.yaml
```

### Decisões de arquitetura

- **Cada app é conteinerizado e servido de forma independente**; o reaproveitamento máximo fica em `/packages`.
- **Contratos de API são sincronizados manualmente** do OpenAPI do backend (sem dependência de build cruzado).
- **O backend (Rust/Axum) é consumido via HTTP** pelos apps Angular, por meio de endpoints REST sob `/api/v1/`.
- **O build order do Turborepo garante que packages sejam construídos antes dos apps** que os consomem (`dependsOn: ["^build"]`).

### Como os packages são consumidos

- **`@accurit/core` e `@accurit/shared`**: exportam **fonte TypeScript** (`main`/`exports` → `src/index.ts`). Não há etapa de build do pacote: o bundler do app (Angular/Next) transpila no consumo. São somente tipos e funções puras — zero dependência de Angular e nenhuma dependência de runtime além de `tslib`.
- **`@accurit/ui`**: única lib compilada — Angular **ng-packagr** → `dist/ui` (FESM2022 + `.d.ts`), com `tokens.css` exportado separadamente (`@accurit/ui/tokens.css`). O app consome a FESM publicada localmente via `node_modules` workspace; o código de tela importa os seletores (`acc-button`, `acc-badge`, `acc-spinner`, …) como `standalone components` diretamente no `imports` do componente — sem módulos e sem barrel global.
- **Resolução**: pnpm workspace cria symlinks em `node_modules/@accurit/*`; não há `paths` no `tsconfig` da app — a resolução segue o esquema de exportação de cada pacote. `turbo run build` ordena `^build` primeiro (`packages/shared → packages/core → packages/ui → apps/*`).

## 2. Camadas por app (variação corporativa de DDD front)

```
apps/<angular-app>/src/app/
├─ core/        # autenticação, guards, interceptor, config global (API_BASE_URL)
├─ data/        # services de domínio: HttpClient + estado em Signals
└─ features/    # telas (standalone components, lazy por rota)
```

- `data/service.ts` possui estado privado `#state = signal<...>()` e expõe apenas `computed` de leitura.
- `features/*` consomem os signals declarativamente no template (`@if`, `@for`, texto derivado), **sem** `async pipe` nem observables em camada de view.
- RxJS restrito a HttpClient/guards/interceptor e fluxos que precisam de streaming/single-flight (ex.: `SerproAmbienteService` memoiza a requisição com `ReplaySubject(1)`).

## 3. Contratos e validações

- **`@accurit/core`** = tipos *explícitos* de request/response. Nenhuma chamada HTTP tipa inline contra `any`; o service define `<ListagemClientesDto>` direto do core.
- **`@accurit/shared`** = funções puras. `ClienteService` usa `sanitizeDocument` para remover duplicação de CPF/CNPJ na busca e nas validações.
- **`@accurit/ui`** = design system; o app nunca reimplementa botão/badge/spinner localmente.

## 4. Roteamento no edge (Nginx)

| Rota | Upstream | Observação |
| --- | --- | --- |
| `/` | `web:3000` | Next.js SSG + `/api/leads` (route handler do próprio web) |
| `/app/` | `client:80` | SPA Angular com `--base-href /app/` |
| `/admin/` | `admin:80` | SPA Angular com `--base-href /admin/` |
| `/api/` | `backend:8080` | rate-limit 30 r/s; logins dedicados a `10 r/m` |
| `/login` | 301 → `/app/login` | compat de URL antiga |

Os base-hrefs chegam como build args (`BASE_HREF`) ao `ng build` dentro do Dockerfile; os nginx de runtime (por app) entregam `index.html` (SPA fallback) + `Cache-Control immutable` para assets com hash.

## 5. Containers

| Serviço | Dockerfile | Runtime |
| --- | --- | --- |
| `web` | `apps/web/Dockerfile` | node:24-alpine + `next build` standalone (`NEXT_OUTPUT=standalone`) |
| `admin` | `apps/admin/Dockerfile` | nginx stable-alpine, healthcheck `/healthz` |
| `client` | `apps/client/Dockerfile` | nginx stable-alpine, healthcheck `/healthz` |
| `proxy` | imagem nginx | monta `nginx/nginx.conf` da raiz |

Healthchecks do compose exigidos pelo `depends_on` do proxy: web (`:3000/`), admin/client (`/healthz`), backend (`/api/v1/health`).

## 6. Qualidade

- `pnpm typecheck` (raiz do monorepo) cobre packages + apps.
- Testes unitários: **Vitest** nos apps Angular (`@angular/build:unit-test`) — sem Karma.
- Lint/padronização: `@repo/eslint-config` flat configs (base/angular/library) + Prettier.

## 7. Integração SERPRO no `apps/client` (modelo Software House)

Esta seção descreve como o portal do escritório (`apps/client`) consome os pacotes para o fluxo
de **validação de procuração e-CAC** (`/Apoiar` → AUTENTICAPROCURADOR), adotado no modelo
Software House (ADR-0007).

### 7.1 Papel dos pacotes

| Pacote | Uso no fluxo |
| --- | --- |
| `@accurit/core` | DTOs do contrato com o backend: `AutenticarProcuradorRequestDto` (`{ clienteId }`) e `AutenticarProcuradorResponseDto` (`{ clienteId, status: 'AUTORIZADA', expiraEm: string \| null }`). Também `AmbienteSerpro`/`AmbienteSerproInfoDto` (já existentes). Espelham o JSON `camelCase` do backend — **o `jwt_token` nunca aparece em nenhum DTO**. |
| `@accurit/ui` | Componentes do design system usados na tela: `AccButtonComponent` (variantes `primary`/`outline`/`ghost`, `loading` input), `AccBadgeComponent` (`status`: `success \| warning \| danger \| info \| neutral`) e `AccSpinnerComponent` (`size`: `sm \| md \| lg`). |
| `@accurit/shared` | `formatDocument`/`isValidDocument` no rendering e validação da busca por CPF/CNPJ (inalterado). |

### 7.2 Fluxo da tela (`features/clientes/cliente-list.component.ts`)

1. A listagem de clientes (`ClienteService`) exibe em cada linha o botão **"Validar Procuração e-CAC"** (`acc-button` `primary`).
2. Durante a requisição, o botão é substituído por `<acc-spinner size="sm">` (estado local `procuracaoValidando: signal<ReadonlySet<string>>`).
3. Ao concluir, um `<acc-badge>` mostra o status: **Autorizada** (`status="success"`) ou **Pendente** (`status="warning"`).
4. Falhas HTTP (401/403/429/503…) são convertidas pelo `data/procuracao.service.ts` em status `PENDENTE` e uma mensagem de erro não-bloqueante é exibida como banner `role="alert"` (com botão `Fechar`) renderizado abaixo da tabela de clientes.
5. O status por cliente fica em `ProcuracaoService#statusPorCliente` (signal somente-leitura). Ausência de entrada = `PENDENTE`.

### 7.3 Gerenciamento seguro dos tokens de autorização

Regra de ouro: **o `autenticar_procurador_token` (jwt_token) nunca entra no navegador**.

- O backend (Rust) assina o Termo com o A1 do escritório, chama o `/Apoiar`, recebe o token e grava **cifrado** (AES-256-GCM, `nonce||ciphertext`) em `tokens_serpro`, vinculado a `(escritorio_id, cliente_id, ambiente)` — um token vigente por cliente (upsert pela constraint `uq_tokens_serpro`).
- O endpoint `POST /api/v1/serpro/integra-contador/procuracoes/autenticar` (audiência `Client`, `Authorization: Bearer <token da sessão do Accurit>` via `authInterceptor`) responde apenas `{ clienteId, status, expiraEm }`. O frontend não consegue obter o jwt_token pelo contrato.
- O frontend guarda **apenas o status de UI**, em memória (signal), por cliente e por sessão:
  - nada em `localStorage`, `sessionStorage`, IndexedDB ou cookies;
  - nada de refresh automático do token pelo browser; a validação é sempre deliberada (botão do usuário) ou consequência de interação explícita;
  - ao recarregar a página, o status volta a `PENDENTE` (fail-closed para a UI): o usuário revalida com um clique; o backend não reexpõe tokens para "restaurar" estado.
- O `authInterceptor` anexa o Bearer da sessão do escritório (token do Accurit) e trata 401 → `limpar sessão + /login`; ele **não conhece nem toca** no `autenticar_procurador_token`.
- Logs: nenhum valor de token é logado no frontend; o backend loga apenas metadados (`ambiente`, `estado`, `status_code`, `request_id`) — ver ADR-0007 e ADR-0004.

### 7.4 Binômio de consumo `@accurit/core` + `@accurit/ui`

```ts
// DTOs (pacote core, fonte TS consumida direto pelo bundler)
import { AutenticarProcuradorRequestDto, AutenticarProcuradorResponseDto } from '@accurit/core';

// UI (pacote ui, FESM `dist/ui` — ng-packagr)
import { AccButtonComponent, AccBadgeComponent, AccSpinnerComponent } from '@accurit/ui';
```

```ts
// Comunicação com o backend (HttpClient autenticado pelo interceptor da sessão)
@Injectable({ providedIn: 'root' })
export class ProcuracaoService {
  // POST /api/v1/serpro/integra-contador/procuracoes/autenticar
  // body: { clienteId }  →  resp: { clienteId, status: 'AUTORIZADA', expiraEm }
}
```

Pontos de atenção para quem evoluir o fluxo:

- **Não** adicione o `jwt_token` a nenhuma DTO/resposta — a separação frontend/backend é deliberada (REGRA SEG, ADR-0007).
- **Não** persista o status de procuração no cache do browser; quando o backend passar a expor o status real (ex.: `GET /procuracoes/status?cliente_id=`), o service deve reidratar o signal no load, mantendo fail-closed.
- Componentes de tela só importam `standalone components` de `@accurit/ui`; nada de seletores "parallel" reimplementados localmente.
- A validação exige Certificado A1 válido no backend; erro de certificado/config do escritório retorna 503 e a UI mostra o banner de erro (não confunde com "procuração não outorgada", HTTP 403).

## 8. Pendências conhecidas

1. `/api/leads` (route handler do Next) e `/api/` (backend) dividem o mesmo prefixo no edge — decidir ownership do lead routing (ou mover para o backend).
2. `og-image` e `favicon` próprios da Accurit na landing.
3. Reidratação do status de procuração na carga da lista de clientes (endpoint dedicado no backend) para evitar que o badge reinicie em `Pendente` a cada reload da sessão.
4. Migração dos demais fluxos SERPRO (envelope `contratante`/`autorPedidoDados` e uso de `autenticar_procurador_token` nas chamadas subsequentes) para o modelo Software House de ponta a ponta — ver "Pendências e follow-ups" no ADR-0007.
