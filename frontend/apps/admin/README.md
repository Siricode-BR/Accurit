# admin — Backoffice SaaS (Angular)

Backoffice do administrador da plataforma: gestão de escritórios (tenants) e alternância do modo trial do SERPRO.

## Rotas

| Rota | Descrição |
| --- | --- |
| `/login` | Login administrativo (`publicGuard`) |
| `/tenants` | Listagem de escritórios com métricas (`authGuard`) |

Rotas lazy via `loadComponent`; base-href de produção é `/admin/` (build arg `BASE_HREF` no compose).

## Estrutura

```
src/app/
├─ core/
│  ├─ api.config.ts          # API_BASE_URL (InjectionToken)
│  └─ auth/
│     ├─ auth.service.ts     # login/logout com Signals (isLoading, errorMessage)
│     ├─ session.store.ts    # SessionStore persistido em localStorage
│     ├─ auth.interceptor.ts # Bearer token + redirect em 401
│     └─ auth.guards.ts      # authGuard / publicGuard
├─ data/
│  └─ tenant.service.ts      # estado dos tenants em Signals
└─ features/
   ├─ auth/login.component.ts
   └─ tenants/tenant-list.component.ts
```

## Padrão

Standalone + Signals ONLY na UI: os services (`data/`) guardam o estado em `#state = signal(...)` privado e expõem `computed` de leitura; componentes consomem os signals declarativamente no template (`@if`, `@for`), sem `async pipe` nem observables de UI. RxJS aparece apenas no HttpClient e no interceptor.

## Comandos

```sh
pnpm --filter admin dev       # ng serve (http://localhost:4200)
pnpm --filter admin build     # ng build --configuration production
pnpm --filter admin test      # Vitest
pnpm --filter admin typecheck
```

## Docker

Serviço `admin` no compose: build a partir do `./frontend` (`apps/admin/Dockerfile`), runtime nginx servindo o SPA estático com `/healthz` e roteado em `/admin/`.
