# Arquitetura do Frontend (monorepo Turborepo)

> Status: atualizada para a estrutura `frontend/` pós-refatoração. O backend NÃO faz parte deste monorepo — vive em `backend/` (repositório/container próprio, Rust/Axum) e é consumido via HTTP.

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

Decisões: cada app é conteinerizado e servido de forma independente; o reaproveitamento máximo fica em `/packages`; contratos de API são **sincronizados manualmente** do OpenAPI do backend (sem dependência de build cruzada).

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
- **`@accurit/ui`** = única lib compilada (ng-packagr → FESM `dist/ui`). Turborepo garante `dependsOn: ["^build"]`; os demais packages exportam TS fonte ESM (transpile no consumidor; Next via `transpilePackages`).

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

## 7. Pendências conhecidas

1. `/api/leads` (route handler do Next) e `/api/` (backend) dividem o mesmo prefixo no edge — decidir ownership do lead routing (ou mover para o backend).
2. `og-image` e `favicon` próprios da Accurit na landing.
