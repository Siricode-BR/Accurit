# Accurit — Plataforma B2B Contábil & Consulta Fiscal SERPRO

[![Repositório](https://img.shields.io/badge/GitHub-Siricode--BR%2FAccurit-181717?logo=github&logoColor=white)](https://github.com/Siricode-BR/Accurit)
[![Rust](https://img.shields.io/badge/Rust-2024-000000?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Axum](https://img.shields.io/badge/Axum-0.8-2f6f8f)](https://github.com/tokio-rs/axum)
[![Angular](https://img.shields.io/badge/Angular-21-DD0031?logo=angular&logoColor=white)](https://angular.dev/)
[![Next.js](https://img.shields.io/badge/Next.js-16-App%20Router-000000?logo=nextdotjs&logoColor=white)](https://nextjs.org/)
[![PostgreSQL](https://img.shields.io/badge/PostgreSQL-16-4169E1?logo=postgresql&logoColor=white)](https://www.postgresql.org/)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

**Accurit** é uma plataforma B2B multi-tenant para escritórios de contabilidade, focada em gestão de clientes, auditoria e consultas fiscais automatizadas integradas aos serviços SERPRO (Integra Contador e Dívida Ativa da União). Desenvolvida em Rust (Axum, Diesel), Angular 21 e Next.js.

> **Slogan:** SaaS multi-tenant de gestão contábil e consulta fiscal automatizada via APIs SERPRO (Integra Contador & Dívida Ativa da União).

---

## Visão geral

A Accurit centraliza a operação fiscal do escritório contábil em um único ambiente: cadastro e segmentação de clientes PF/PJ, verificação de procurações e-CAC, consulta de pendências na Dívida Ativa da União e execução dos cinco verbos operacionais do Integra Contador (`Apoiar`, `Consultar`, `Declarar`, `Emitir` e `Monitorar`).

O produto foi desenhado para eliminar erro operacional e retrabalho: os payloads são higienizados antes do envio, os dados são isolados por escritório (`escritorio_id`), as credenciais SERPRO são protegidas por criptografia e o ambiente Trial/Produção é decidido exclusivamente pelo backend.

### Proposta de valor

- **Precisão fiscal:** normalização de CPF/CNPJ, valores monetários e payloads antes de cada chamada às APIs SERPRO.
- **Automação SERPRO:** Dívida Ativa com OAuth2 Client Credentials e Integra Contador com SAPI
  (`access_token` + `jwt_token`), cache com renovação em HTTP 401, mTLS e certificado por escritório.
- **Zero erro operacional:** validação de procuração e-CAC antes de serviços que exigem representação, assinatura digital de XML e trilha de auditoria.
- **Compliance by design:** isolamento multi-tenant, minimização de dados, logs redigidos e retenção controlada.
- **Operação simples:** migrations PostgreSQL executadas automaticamente no startup do container, sem `diesel_cli` no host.

## Funcionalidades

- **Dívida Ativa da União (somente leitura):** consulta por CPF/CNPJ (`GET {base}/devedor/{cpfCnpj}`) e detalhamento por inscrição (`GET {base}/inscricao/{numeroInscricao}`). A base termina em `/api/v1`; não duplicar o segmento. Não há parcelamento, negociação, liquidação, cobrança ou emissão de DARF.
- **Integra Contador:** suporte aos verbos `Apoiar`, `Consultar`, `Declarar`, `Emitir` e `Monitorar`, com envelope JSON padronizado e catálogo de sistemas/serviços habilitados.
- **Procurações e-CAC:** verificação via `PROCURACOES / OBTERPROCURACAO41` antes de serviços protegidos, **somente quando `autorPedidoDados` difere do contribuinte**, com códigos de referência por sistema (PGDASD, DCTFWeb, SITFIS, Caixa Postal, DTE e outros).
- **Assinatura eletrônica:** suporte a XMLDSig com `<Signature>` e `<Transform>` no namespace default (sem prefixo `ds:`), validação de digest e cadeia ICP-Brasil do certificado A1.
- **Modo Trial SERPRO:** toggle reativo no painel `/admin`, persistido por escritório em `escritorios.modo_trial_serpro`.
- **Gestão de clientes:** cadastro PF/PJ, validação de CPF/CNPJ e filtros por regime tributário.
- **Multi-tenancy:** todo dado de negócio é isolado por `escritorio_id`, derivado do contexto autenticado.
- **Migrations automáticas:** SQL versionado embutido no binário Rust com `diesel_migrations::embed_migrations!` e aplicado no startup.
- **Auditoria:** registro de consultas, alternância de ambiente e ações administrativas sem armazenar payload fiscal excessivo.

## Arquitetura da solução

```text
                                   Internet (HTTPS)
                                          │
                                          ▼
                        ┌──────────────────────────────────┐
                        │      Nginx Proxy (:80/:443)      │
                        │  TLS · gzip · rate limit · SPA   │
                        └───┬──────────┬──────────┬────────┘
                            │          │          │
              ┌─────────────┘          │          └─────────────┐
              │ /                      │ /admin                 │ /app · /login
              ▼                        ▼                        ▼
     ┌─────────────────┐      ┌─────────────────┐      ┌─────────────────┐
     │ Next.js Web App │      │ Angular Admin   │      │ Angular Client  │
     │ :3000 (SSG/SSR) │      │ :80 (SPA/CSR)   │      │ :80 (SPA/CSR)   │
     └────────┬────────┘      └────────┬────────┘      └────────┬────────┘
              │                        │                        │
              └────────────────────────┼────────────────────────┘
                                       │ /api/*
                                       ▼
                         ┌───────────────────────────────┐
                         │  Rust · Axum · Tokio · Diesel │
                         │  DDD · Clean Architecture     │
                         │  :8080                        │
                         └───────┬───────────────┬───────┘
                                 │               │
                                  ▼               ▼ (mTLS + e-CNPJ/A1)
                       ┌──────────────────┐   ┌──────────────────────────┐
                       │ PostgreSQL 16    │   │ SERPRO Gateway           │
                       │ :5432            │   │ Integra Contador         │
                       │                  │   │ Dívida Ativa da União    │
                       └──────────────────┘   └──────────────────────────┘
```

### Stack tecnológica

| Camada | Tecnologia | Responsabilidade |
|---|---|---|
| Backend | Rust 2024, Axum 0.8, Tokio, Diesel 2.2+ | API REST, regras de negócio, integrações SERPRO |
| Banco | PostgreSQL 16 (`postgres:16-alpine`) | Persistência multi-tenant e migrations |
| Web | Next.js (App Router, SSG/SSR) atrás do proxy Nginx | Site institucional, SEO e captação de leads |
| Admin | Angular 21 (Signals, Standalone) | Painel do escritório: clientes, configurações e Trial |
| Cliente | Angular 21 (Signals, Standalone) | Portal do cliente e login |
| Proxy | Nginx (`nginx:alpine`) | Roteamento, TLS, compressão e cache de estáticos |
| Orquestração | Docker Compose | Build e execução integrada da stack |

### Mapeamento de rotas do Nginx

| Rota pública | Upstream | Container | Estratégia |
|---|---|---|---|
| `/` | `web:3000` | `accurit-web` | Proxy Next.js (SSG/SSR) e assets `/_next/` |
| `/admin/` | `admin:80` | `accurit-admin` | SPA Angular com fallback para `index.html` |
| `/app/` | `client:80` | `accurit-client` | SPA Angular com fallback para `index.html` |
| `/login` | `client:80` (redirect) | `accurit-client` | `301` para `/app/login` |
| `/api/` | `backend:8080` | `accurit-backend` | `proxy_pass` sem cache; repassa `Authorization`, remove `x-cpf-usuario` do cliente e valida `X-Request-Id` |
| `/api/v1/health` | `backend:8080` | `accurit-backend` | Health check usado pelo Compose |

## Quickstart

Pré-requisitos: Docker Engine 24+ e Docker Compose v2.20+.

```bash
git clone https://github.com/Siricode-BR/Accurit.git
cd Accurit
./scripts/init-env.sh          # PowerShell: .\scripts\init-env.ps1
docker compose up --build
```

O `scripts/init-env` cria o `.env` a partir do `.env.example` com senhas hexadecimais,
`JWT_SECRET` (base64 de 48 bytes) e `CERTIFICATE_MASTER_KEY` (base64 de 32 bytes) gerados, sem
imprimir os segredos e sem sobrescrever um `.env` existente. Alternativamente, gere manualmente com
`openssl rand -base64 48` (`JWT_SECRET`) e `openssl rand -base64 32` (`CERTIFICATE_MASTER_KEY`).

> **Placeholders não sobem.** O backend recusa `JWT_SECRET=change-me` e
> `CERTIFICATE_MASTER_KEY=change-me-32-bytes` no startup, com mensagem clara e sem revelar valores.

> **Um passo, zero Diesel CLI.** No startup, o backend aguarda o PostgreSQL ficar saudável e executa automaticamente todas as migrations embutidas no binário. **Não é necessário instalar `diesel_cli` no host nem rodar comandos manuais de migration.**

Para executar em segundo plano:

```bash
docker compose up --build -d
docker compose logs -f backend
```

Para encerrar preservando os volumes do banco:

```bash
docker compose down
```

### Acessos locais

| Serviço | URL |
|---|---|
| Web (site institucional) | `http://localhost/` |
| Painel administrativo | `http://localhost/admin/` |
| Login do cliente | `http://localhost/login` |
| Portal do cliente | `http://localhost/app/` |
| Health check da API | `http://localhost/api/v1/health` |

## Configuração SERPRO

A seleção de ambiente é persistida por escritório em `escritorios.modo_trial_serpro` e resolvida no backend a cada requisição.

| Produto | Ambiente Trial | Ambiente de produção |
|---|---|---|
| Integra Contador | `https://gateway.apiserpro.serpro.gov.br/integra-contador-trial/v1` | `https://gateway.apiserpro.serpro.gov.br/integra-contador/v1` |
| Consulta Dívida Ativa | `https://gateway.apiserpro.serpro.gov.br/consulta-divida-ativa-trial/api/v1` | **Indefinida**; bloqueada por `SERPRO_PROD_NOT_CONFIGURED` até confirmação SERPRO |
| Token Dívida Ativa | `https://gateway.apiserpro.serpro.gov.br/token` | Mesmo endpoint, com credenciais de produção |
| Autenticação Integra Contador | Bearer público de demonstração | SAPI: `https://autenticacao.sapi.serpro.gov.br/authenticate` (mTLS e-CNPJ) |

- **Trial:** usa as URLs de homologação e a chave Bearer pública de demonstração configurada em `SERPRO_TRIAL_BEARER_TOKEN`. O valor deve ser copiado da documentação oficial da SERPRO para o `.env` local e nunca versionado.
- **Produção:** o Integra Contador usa SAPI com mTLS e-CNPJ, retornando `access_token` e `jwt_token`; a Dívida Ativa usa OAuth2 com mTLS e certificado A1 do escritório, mas permanece bloqueada até a SERPRO confirmar a URL.
- Credenciais são por escritório (`credenciais_serpro`); `SERPRO_CONSUMER_KEY`/`SERPRO_CONSUMER_SECRET` globais são apenas fallback de desenvolvimento.

> A chave de trial é configuração de ambiente, não segredo hardcoded. Nunca envie credenciais, certificados, tokens ou dados pessoais para logs ou para o repositório.

## Estrutura do repositório

```text
Accurit/
├── docs/                           # referências oficiais SERPRO (cópias de trabalho)
│   ├── ADR/                        # decisões arquiteturais
│   └── Serpro/
│       ├── Divida-Ativa/
│       ├── Integra-Contador/
│       └── Preços/
├── backend/                        # Rust 2024 · Axum · Diesel · DDD
│   ├── Cargo.toml
│   ├── Cargo.lock
│   ├── diesel.toml
│   ├── Dockerfile
│   ├── migrations/
│   │   └── 20261020120000_create_escritorios/
│   │       ├── up.sql
│   │       └── down.sql
│   └── src/
│       ├── main.rs                 # bootstrap, migrations e servidor Axum
│       ├── lib.rs
│       ├── domain/
│       │   ├── auth/
│       │   ├── tenancy/
│       │   ├── clientes/
│       │   ├── certificados/
│       │   ├── procuracoes/
│       │   ├── divida_ativa/
│       │   ├── integra_contador/
│       │   └── serpro/
│       ├── application/            # casos de uso e orquestração
│       ├── infrastructure/
│       │   ├── persistence/        # repositórios Diesel
│       │   ├── serpro/             # clientes HTTP, OAuth2 e mTLS
│       │   └── crypto/             # AES-256-GCM para A1/tokens
│       └── presentation/
│           ├── routes/
│           ├── handlers/
│           ├── dto/
│           └── middleware/         # auth, tenancy, request-id, rate limit
├── admin/                          # Angular 21 · painel do escritório
│   ├── Dockerfile
│   ├── package.json
│   └── src/app/
│       ├── core/
│       ├── features/
│       │   ├── auth/
│       │   ├── clientes/
│       │   ├── configuracoes/
│       │   └── trial/
│       └── shared/
├── client/                         # Angular 21 · portal do cliente
│   ├── Dockerfile
│   ├── package.json
│   └── src/app/
│       ├── core/
│       ├── features/
│       └── shared/
├── web/                            # Next.js · App Router
│   ├── Dockerfile
│   ├── package.json
│   └── src/app/
├── nginx/
│   └── nginx.conf                  # roteamento /, /admin, /app, /login, /api
├── docker-compose.yml
├── .env.example
├── README.md
├── SPEC.md
├── ROADMAP.md
├── INSTRUCTIONS.md
└── LICENSE
```

## Documentação

- [`SPEC.md`](SPEC.md): arquitetura, contratos SERPRO, DTOs, modelo de dados e segurança.
- [`ROADMAP.md`](ROADMAP.md): entregas organizadas por fase e camada.
- [`INSTRUCTIONS.md`](INSTRUCTIONS.md): setup, migrations, troubleshooting e contribuição.
- [`docs/Serpro/`](docs/Serpro): cópias de trabalho da documentação oficial SERPRO (Integra Contador, Dívida Ativa e preços).

## Segurança e limites operacionais

- Todas as consultas e mutações de domínio são limitadas ao `escritorio_id` obtido do contexto autenticado.
- O header `x-cpf-usuario` é operacional e **não consta na documentação oficial**; é enviado apenas
  quando `SERPRO_OPERATOR_HEADER_ENABLED=true`, a partir de `usuarios.cpf`, e nunca é aceito do
  cliente. O comportamento real do SERPRO é validado por teste live (D3).
- Certificados A1, senhas, `Consumer Secrets`, `jwt_token` e tokens são protegidos por criptografia AES-256-GCM e segredos de ambiente.
- CPF/CNPJ e valores monetários são higienizados antes da serialização do envelope JSON.
- O módulo Dívida Ativa é exclusivamente consultivo: não há rotas, casos de uso ou telas de parcelamento, negociação, cobrança ou DARF.
- O modo Trial não é fonte de dados fiscais reais e não pode ser usado como evidência de produção.
- Retries automáticos são permitidos apenas para falhas transitórias e operações idempotentes, respeitando `Retry-After`.
- Logs são redigidos; payloads fiscais são minimizados e mantidos apenas pelo período necessário.

## Licença

Distribuído sob a licença MIT. Consulte [`LICENSE`](LICENSE).
