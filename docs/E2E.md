# Testes E2E (Playwright)

A suíte cobre os fluxos críticos do portal e do backoffice contra um backend real,
PostgreSQL real e os mocks determinísticos de Trial. Nenhuma chamada à SERPRO real é
feita.

## Cobertura

| Projeto | Fluxo |
| --- | --- |
| `admin` | login do gestor, painel `/consumo`, aba de planos, listagem de administradores e rejeição de token de audiência `client` em rota de admin |
| `client` | login do escritório, cadastro de cliente PF, validação de procuração no Trial, consulta de Dívida Ativa por devedor, geração de relatório PDF, link público sem sessão e painel de consumo/limite |

Os testes vivem em `frontend/e2e`:

- `admin.spec.ts`
- `client.spec.ts`
- `static-server.mjs` — serve o build estático e faz proxy de `/api` para `127.0.0.1:8080`.

## Execução local (um comando)

Pré-requisitos:

- PostgreSQL 16+ acessível em `127.0.0.1:5432` com um usuário administrador para criar o banco de E2E;
- Rust/Cargo, Node 24 e pnpm;
- navegador do Playwright: `cd frontend && pnpm exec playwright install chromium`.

Na raiz:

```powershell
./scripts/e2e.ps1 -Reset
```

O script:

1. recria o banco `accurit_e2e`;
2. compila o backend e roda as migrations;
3. sobe o backend com `SERPRO_TRIAL_MOCK=true`;
4. cria o administrador `admin@e2e.test`, o escritório de teste e o plano `E2E`;
5. builda admin/client e sobe os servidores estáticos em `4201`/`4202`;
6. executa `pnpm exec playwright test` e encerra os processos ao final.

Logs ficam em `%TEMP%\accurit-e2e`. As portas `8080`, `4201` e `4202` precisam estar livres.

### Execução manual

Se preferir controlar cada processo:

```powershell
# 1. banco e backend (o backend aplica migrations no startup)
$env:DATABASE_URL='postgres://accurit:accurit@127.0.0.1:5432/accurit_e2e'
$env:SERPRO_TRIAL_MOCK='true'
$env:JWT_SECRET='e2e-secret-with-at-least-32-bytes-long'
$env:CERTIFICATE_MASTER_KEY='BwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwc='
cargo run --manifest-path backend/Cargo.toml

# 2. em outro terminal: build e servidores estáticos (a partir de frontend/)
cd frontend
pnpm --filter admin build
pnpm --filter client build
$env:PORT='4201'; $env:DIST='apps/admin/dist/admin/browser'; $env:BACKEND='http://127.0.0.1:8080'; node e2e/static-server.mjs
$env:PORT='4202'; $env:DIST='apps/client/dist/client/browser'; $env:BACKEND='http://127.0.0.1:8080'; node e2e/static-server.mjs

# 3. testes (em frontend/)
pnpm exec playwright test
```

O seed manual (admin, tenant, plano/assinatura) está descrito no cabeçalho de
`scripts/e2e.ps1`.

## Configuração

| Variável | Padrão | Uso |
| --- | --- | --- |
| `E2E_ADMIN_URL` | `http://127.0.0.1:4201` | backoffice |
| `E2E_CLIENT_URL` | `http://127.0.0.1:4202` | portal do escritório |
| `E2E_ADMIN_EMAIL` / `E2E_ADMIN_PASSWORD` | `admin@e2e.test` / `e2e-admin-123` | login admin |
| `E2E_ADMIN_NOME` | `Gestor E2E` | nome exibido em administradores |
| `E2E_CLIENT_EMAIL` / `E2E_CLIENT_PASSWORD` | `titular@e2e.test` / `e2e-user-123` | login do escritório |
| `E2E_PLANO` | `E2E` | nome do plano exibido no admin |
| `E2E_CHAMADAS_LIMITE` | `1[,.]000` | expressão do limite de chamadas no painel client |

## Limitações

- A suíte usa CPF gerado por execução para o cliente e CPF fixo `12345679891` na consulta
  de Dívida Ativa (cenário `COM_PENDENCIAS` do mock).
- O link público é validado em um contexto de navegador sem cookies.
- Ainda não há job de E2E no GitHub Actions porque o CI atual não sobe a stack completa
  com nginx e seed. O script local é a referência executável.
