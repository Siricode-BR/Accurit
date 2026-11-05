# client — Portal do Escritório Contábil (Angular)

Plataforma que o escritório de contabilidade usa no dia a dia: cadastro de clientes e, em seguida, consultas SERPRO (Integra Contador / Dívida Ativa).

## Rotas

| Rota | Descrição |
| --- | --- |
| `/login` | Login do escritório (`publicGuard`) |
| `/clientes` | Gestão de clientes PF/PJ com validação de CPF/CNPJ (`authGuard`) |

Rotas lazy via `loadComponent`; base-href de produção é `/app/` (build arg `BASE_HREF` no compose — esta é a rota acessada pelo botão "Portal do Escritório" da landing).

## Estrutura

```
src/app/
├─ core/
│  ├─ api.config.ts           # API_BASE_URL (InjectionToken)
│  └─ auth/
│     ├─ auth.service.ts      # login/logout com Signals
│     ├─ session.store.ts     # SessionStore persistido em localStorage
│     ├─ auth.interceptor.ts  # Bearer token + redirect em 401
│     └─ auth.guards.ts       # authGuard / publicGuard
├─ data/
│  ├─ cliente.service.ts      # clientes + filtros (estado em Signals)
│  └─ serpro.ambiente.service.ts  # ambiente SERPRO do escritório (RxJS single-flight)
└─ features/
   ├─ auth/login.component.ts
   └─ clientes/cliente-list.component.ts
```

## Design system

Componentes `acc-*` vindos de `@accurit/ui` (botão, card, badge, spinner, máscara de documento) e tokens CSS (`@accurit/ui/tokens.css`) — nenhum estilo de marca é redefinido no app.

## Comandos

```sh
pnpm --filter client dev       # ng serve (http://localhost:4200)
pnpm --filter client build     # ng build --configuration production
pnpm --filter client test      # Vitest
pnpm --filter client typecheck
```

> Os dois apps Angular usam a porta 4200 por padrão — rode um por vez ou ajuste `ng serve --port` (ou `angular.json` / variável de porta no compose).

## Docker

Serviço `client` no compose: build a partir do `./frontend` (`apps/client/Dockerfile`), runtime nginx com `/healthz`, roteado em `/app/`.
