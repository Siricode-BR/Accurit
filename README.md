# Accurit — Plataforma Contábil & Consulta Fiscal SERPRO

**Accurit** é uma plataforma B2B multi-tenant para escritórios de contabilidade, focada em gestão de clientes, auditoria e consultas fiscais automatizadas integradas aos serviços SERPRO (Integra Contador e Dívida Ativa). Desenvolvida em Rust (Axum, Diesel), Angular 18+ e Next.js.

> Plataforma B2B de Gestão Contábil, Compliance e Consultas Fiscais SERPRO.

> **Escopo de Dívida Ativa:** exclusivamente consultivo. A aplicação consulta e apresenta inscrições, situações e valores retornados pela API SERPRO. Não realiza parcelamento, negociação, emissão de cobrança ou emissão de DARF para Dívida Ativa.

## Funcionalidades

- **Consulta de Dívida Ativa por devedor:** CPF/CNPJ via `GET /v1/devedor/{cpfCnpj}`.
- **Consulta detalhada por inscrição:** via `GET /v1/inscricao/{numeroInscricao}`.
- **Integra Contador:** suporte aos verbos operacionais `Apoiar`, `Consultar`, `Declarar`, `Emitir` e `Monitorar`, conforme os serviços habilitados para o escritório.
- **Procurações e-CAC:** verificação de autorização antes de executar serviços que exigem procuração.
- **Gestão de clientes:** cadastro de pessoa física (CPF) e pessoa jurídica (CNPJ).
- **Filtros de clientes:** Todos, CPF, CNPJ - MEI, CNPJ - Simples Nacional, CNPJ - Lucro Presumido e CNPJ - Lucro Real.
- **Modo Trial SERPRO:** alternância no painel `/admin`, persistida por escritório.
- **Multi-tenancy:** isolamento dos dados por `escritorio_id`.
- **Migrations automáticas:** migrações PostgreSQL são embutidas no binário Rust e executadas no startup do backend.

## Stack

- Backend: Rust, Axum, Diesel ORM, `diesel_migrations`, Reqwest e PostgreSQL 16.
- Admin e área do cliente: Angular v18+ com Signals e arquitetura limpa organizada por features.
- Landing page: Next.js.
- Infraestrutura: Docker Compose e Nginx.

## Quickstart

### 1. Preparar configuração

```bash
git clone https://github.com/Siricode-BR/Accurit.git
cd Accurit
cp .env.example .env
```

Edite `.env` com os valores necessários ao ambiente. Configure credenciais SERPRO, segredos de aplicação e conexão com o banco conforme o `.env.example`. Não versione credenciais reais.

### 2. Subir a plataforma

```bash
docker compose up --build
```

O Compose inicia os serviços definidos no projeto. No startup, o backend aguarda o PostgreSQL estar disponível e executa automaticamente as migrations compiladas no binário Rust. **Não é necessário instalar Diesel CLI no host nem executar comandos manuais de migration.**

Acessos locais (conforme o `docker-compose.yml` e `nginx.conf`):

- Landing: `http://localhost/`
- Admin: `http://localhost/admin/`
- Área do cliente: `http://localhost/app/`
- Health check: `http://localhost/api/v1/health`

Para executar em segundo plano: `docker compose up --build -d`. Logs: `docker compose logs -f backend`.

## Configuração SERPRO

A seleção Trial/Produção é persistida por escritório em `escritorios.modo_trial_serpro`.

- **Trial:** usa os endpoints de demonstração configurados para Integra Contador e Consulta Dívida Ativa, com credenciais de trial apropriadas.
- **Produção:** usa os endpoints de produção e os requisitos de autenticação, mTLS e certificado A1 aplicáveis ao produto/serviço contratado.

A chave de trial indicada na documentação do projeto deve ser tratada como configuração de ambiente, não como segredo hardcoded no código. Nunca envie credenciais, certificados, tokens ou dados pessoais para logs ou para o repositório. Confirme URLs, escopos, headers e credenciais vigentes na documentação e no contrato SERPRO do ambiente.

## Documentação do repositório

- [`SPEC.md`](SPEC.md): arquitetura, contratos, modelo de dados e integrações.
- [`ROADMAP.md`](ROADMAP.md): entregas organizadas por camada.
- [`INSTRUCTIONS.md`](INSTRUCTIONS.md): setup, migrations e padrões de contribuição.

## Migrations

Adicione migrations SQL em `backend/migrations/<versão>_<nome>/up.sql` e `down.sql`. Elas são embutidas por `diesel_migrations::embed_migrations!` durante a compilação do backend e executadas no startup. Consulte `INSTRUCTIONS.md`.

## Segurança e limites operacionais

- Todas as consultas e mutações de domínio devem ser limitadas ao `escritorio_id` obtido do contexto autenticado.
- Certificados A1, senhas, Consumer Secrets e tokens devem ser protegidos por criptografia/segredos de ambiente.
- Aplicar timeouts, limites por escritório, retries apenas para falhas transitórias e respeito a `Retry-After`.
- O modo Trial não deve ser considerado fonte de dados fiscais reais nem ser usado como evidência de resultado de produção.
- Os serviços disponíveis, identificadores, códigos de procuração, envelopes e respostas podem variar conforme habilitação e versão do produto SERPRO. Validar os contratos em documentação oficial vigente antes de ativar produção.
