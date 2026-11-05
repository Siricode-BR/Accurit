# web — Landing institucional (Next.js)

Site institucional e captação de leads do Accurit. App Router com **SSG** (páginas estáticas prerenderizadas) e um route handler dinâmico para leads.

## Páginas

| Rota | Renderização | Descrição |
| --- | --- | --- |
| `/` | SSG | Landing page moderna: hero, recursos, integração SERPRO, como funciona, CTA para o **Portal do Escritório Contábil** (`/app/`) |
| `/contato` | SSG + Client Component | Formulário de demonstração (`POST /api/leads`) |
| `/api/leads` | Server (route handler) | Valida com `isValidEmail`/`formatPhone` de `@accurit/shared` |

## Consumo dos pacotes compartilhados

`@accurit/core` e `@accurit/shared` são TypeScript fonte, transpilados pelo próprio build do Next via `transpilePackages` (`next.config.ts`). Server Components e route handlers podem usá-los sem tocar em APIs de browser — estratégia de renderização do App Router preservada.

O `output: "standalone"` só é ativado dentro do Docker (`NEXT_OUTPUT=standalone` no `apps/web/Dockerfile`) para o runner minimalista; localmente `pnpm --filter web start` funciona normalmente.

## Comandos

```sh
pnpm --filter web dev      # next dev (Turbopack)
pnpm --filter web build    # build de produção (SSG)
pnpm --filter web start    # next start (produção local)
pnpm --filter web lint
```

## Docker

`docker-compose.yml` (raiz): serviço `web`, build `context: ./frontend`, `dockerfile: apps/web/Dockerfile`, exposto internamente na `:3000` e roteado pelo Nginx em `/`.
