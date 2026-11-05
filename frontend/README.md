# Accurit Frontend

Monorepo do frontend, gerenciado por **Turborepo + pnpm**. O backend é um serviço externo (repositório/container próprio) e é consumido apenas via HTTP por trás do proxy Nginx.

## Estrutura

```
frontend/
├─ apps/                        # aplicações servidas de forma independente
│  ├─ web/                      # Next.js (App Router, SSG) — landing institucional + captação de leads em /contato
│  ├─ admin/                    # Angular 22 (Signals, Standalone) — backoffice SaaS (tenants)
│  └─ client/                   # Angular 22 (Signals, Standalone) — portal do escritório contábil
├─ packages/                    # código compartilhado entre os apps
│  ├─ ts-config/                # configs base de TypeScript (base / angular / nextjs)
│  ├─ shared/                   # validações e formatadores (CPF/CNPJ, e-mail, telefone, docs)
│  ├─ core/                     # DTOs/contratos de API e tipos de domínio (espelham o OpenAPI do backend)
│  ├─ ui/                       # design system Angular (botão, card, badge, spinner, máscara de documento)
│  └─ eslint-config/            # configs ESLint compartilhadas
├─ turbo.json                   # pipeline de tasks do Turborepo
└─ pnpm-workspace.yaml
```

## Tasks (Turborepo)

Na raiz `frontend/`:

```sh
pnpm install          # instala tudo (workspaces)
pnpm build            # build de todos os apps + @accurit/ui
pnpm typecheck        # tsc --noEmit em todo o workspace
pnpm --filter web build        # build de um app (e deps: ^build)
pnpm --filter client dev       # dev server do portal (ng serve)
pnpm --filter admin dev        # dev server do backoffice
pnpm --filter web start        # next start (produção local)
```

O Dockerfile de cada app constrói a partir de `context: ./frontend` com as mesmas tasks (ver `docker-compose.yml` na raiz do repositório).

## Pacotes compartilhados

| Pacote | Exporta | Consumido por |
| --- | --- | --- |
| `@accurit/ts-config` | `extends` nos `tsconfig*.json` | todos |
| `@accurit/shared` | `cpfCnpj` validators, `isValidDocument`, `formatDocument`, `sanitizeDocument`, e-mail, telefone, moeda | web, admin, client, ui |
| `@accurit/core` | DTOs (`LoginRequestDto`, `ClienteDto`, `TenantDto`, ...) e tipos de domínio | web, admin, client, ui |
| `@accurit/ui` | `AccButton/Card/Badge/Spinner`, `accDocMask` directive, `tokens.css` | admin, client |

Todos são consumidos via `workspace:*` e exportam **TypeScript fonte ESM** (`exports` aponta para `src/index.ts`), compilados pelo bundler de cada app — exceto `@accurit/ui`, que é uma Angular library (ng-packagr → FESM). No Next.js os pacotes de TS fonte entram pelo `transpilePackages`. Somente `@accurit/ui` precisa de `build` antes de ser consumido — o Turborepo resolve essa ordem automaticamente (`dependsOn: ["^build"]`).

> `@accurit/core` é espelhado manualmente do OpenAPI do backend. Não há dependência de build entre frontend e backend.

## Convenções Angular (admin/client)

- **Standalone components** obrigatórios — sem NgModules.
- **Signals-first**: estado em `signal`/`computed`; a UI nunca usa `observable | async`. RxJS só onde é inevitável (HttpClient, guards/interceptors, fluxos com streaming — como o `SerproAmbienteService` single-flight).
- Camadação por app: `core/` (auth/API config), `data/` (services de domínio com estado em Signals), `features/` (componentes de tela).
- O estado do HttpClient de cada service é privado (`#state`) e exposto só como `computed` de leitura.

## Roteamento no edge

O proxy (`nginx/nginx.conf`, raiz do repositório) mapeia:

| Rota pública | Serviço | App |
| --- | --- | --- |
| `/` | `web:3000` | Next.js |
| `/app/` | `client:80` | Portal do escritório (`--base-href /app/`) |
| `/admin/` | `admin:80` | Backoffice SaaS (`--base-href /admin/`) |
| `/api/` | `backend:8080` | Backend Rust |

Os base-hrefs são passados como build args no `docker-compose.yml` (`BASE_HREF`), por isso links internos dos apps Angular funcionam sob o path público sem configuração adicional no dev local.

## Documentação

- Arquitetura detalhada: [`docs/frontend-arquitetura.md`](../docs/frontend-arquitetura.md) (raiz do repositório)
- Design system: [`docs/design/03-design-system.md`](../docs/design/03-design-system.md)
