# INSTRUCTIONS.md — Setup, Migrations e Contribuição

## 1. Pré-requisitos

| Ferramenta | Versão mínima | Uso |
|---|---|---|
| Docker | 24.x | Build e execução dos containers |
| Docker Compose | v2.20+ | Orquestração local |
| Git | 2.x | Controle de versão |
| Node.js | 20 LTS | Desenvolvimento dos frontends |
| npm | 10+ | Dependências dos frontends |
| Rust toolchain | versão compatível com `Cargo.toml` | Apenas para desenvolvimento local opcional |

Não é necessário instalar `diesel_cli` no host. A execução normal do projeto não exige Rust nem Angular CLI instalados localmente, desde que os serviços sejam executados via Docker Compose.

## 2. Setup rápido via Docker Compose

```bash
git clone https://github.com/Siricode-BR/Accurit.git
cd Accurit
cp .env.example .env
```

Configure `.env` com os valores do ambiente. Não inclua credenciais reais no Git. Em seguida:

```bash
docker compose up --build
```

O backend Rust deve aguardar o PostgreSQL ficar disponível e executar as migrations embutidas antes de iniciar o servidor Axum. Não execute `diesel migration run`, `diesel setup` nem instale Diesel CLI no host.

Acessos locais, conforme o Compose/Nginx:

- Landing: `http://localhost/`
- Admin: `http://localhost/admin/`
- Área do cliente: `http://localhost/app/`
- Health check: `http://localhost/api/v1/health`

Para executar em background: `docker compose up --build -d`. Logs: `docker compose logs -f backend`; encerrar: `docker compose down`. Para preservar o banco, não use `-v` ao encerrar, salvo se quiser apagar os volumes.

## 3. Como criar migrations SQL

As migrations ficam em `backend/migrations/`. O binário Rust as inclui durante a compilação usando `diesel_migrations::embed_migrations!`.

Crie manualmente uma nova pasta versionada no padrão aceito pelo Diesel, com os dois arquivos:

```text
backend/migrations/
└── 20261020120000_create_exemplo/
    ├── up.sql
    └── down.sql
```

Exemplo:

`up.sql`
```sql
CREATE TABLE exemplo (
    id UUID PRIMARY KEY,
    escritorio_id UUID NOT NULL REFERENCES escritorios(id),
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

`down.sql`
```sql
DROP TABLE exemplo;
```

Ajuste o timestamp/nome ao padrão de migrations já usado pelo repositório. Escreva SQL compatível com PostgreSQL 16.

### Regras obrigatórias

- Adicionar sempre `up.sql` e `down.sql` quando a reversão for segura.
- Nunca editar uma migration que já foi aplicada em ambientes compartilhados; crie uma nova.
- Revisar constraints, índices e isolamento por `escritorio_id`.
- Evitar operações destrutivas sem plano de migração e backup.
- Após criar ou alterar uma migration, reconstruir o backend para embuti-la: `docker compose up --build`.
- Verificar os logs do backend para confirmar sucesso; se a migration falhar, o backend deve encerrar com erro claro em vez de iniciar parcialmente.
- **Não** rodar Diesel CLI no host ou comandos manuais de migration como parte do fluxo normal.

O código deve declarar `embed_migrations!` apontando para o diretório de migrations relativo ao crate. O startup usa `MigrationHarness::run_pending_migrations` (ou API equivalente da versão do Diesel), com conexão/pool apropriado e retry limitado enquanto o PostgreSQL inicializa. Em ambientes com múltiplas réplicas, a estratégia de deploy deve evitar execução concorrente de migrations.

## 4. Desenvolvimento local opcional

O desenvolvimento individual é opcional; Docker Compose é o caminho recomendado.

```bash
# Inicie banco e dependências
docker compose up -d postgres

# Backend (se a toolchain Rust e variáveis locais estiverem configuradas)
cd backend
cargo run
```

Para frontends, em seus diretórios correspondentes:

```bash
npm ci
npm run start
```

Os scripts podem variar por `package.json`. Configure as URLs de API e CORS para desenvolvimento local. Nunca exponha serviços internos sem autenticação.

## 5. Padrões de arquitetura

### 5.1 Backend Rust — DDD

Para cada nova funcionalidade:

1. `domain/<feature>/`: entidades, value objects, regras e traits; sem Axum, Diesel ou Reqwest.
2. `application/`: casos de uso que orquestram as abstrações do domínio.
3. `infrastructure/`: implementações Diesel, clientes HTTP SERPRO e criptografia.
4. `presentation/`: rotas Axum, handlers, DTOs e validação.

Handlers devem ser enxutos. Derive `escritorio_id` do contexto autenticado; não aceite o tenant como fonte de autoridade no corpo/query da requisição.

### 5.2 Angular — Clean Architecture por features

- Criar `features/<nome>/` com domínio/casos de uso quando necessários, `data/` para DTOs/adaptadores HTTP e `presentation/` para páginas/componentes standalone.
- Usar Signals para estado de UI e `computed` para valores derivados.
- Manter `HttpClient` nos adaptadores da camada `data` e interceptors transversais.
- Usar guards por papel e tratamento consistente de 401/403.
- Implementar filtros de clientes: Todos, CPF, CNPJ - MEI, CNPJ - Simples Nacional, CNPJ - Lucro Presumido e CNPJ - Lucro Real.
- O toggle Trial é apenas apresentação do estado; o backend é a autoridade sobre ambiente e permissões.

### 5.3 Integrações SERPRO

- Separar configuração de Trial e Produção por produto.
- Não incluir Consumer Secret, token, certificado ou senha no código, no frontend ou em logs.
- Antes de serviço que exige procuração, validar `PROCURACOES / OBTERPROCURACAO41`.
- Não presumir que autenticação ou envelopes de produtos SERPRO diferentes são intercambiáveis.
- Dívida Ativa é exclusivamente consultiva; não criar fluxo de parcelamento, negociação, cobrança ou emissão de DARF para esse módulo.
- Não repetir automaticamente operações potencialmente não idempotentes sem proteção.

## 6. Qualidade, segurança e contribuição

Commits seguem Conventional Commits, por exemplo:

```text
feat(divida-ativa): consulta por inscricao
feat(clientes): adiciona filtro por regime
fix(serpro): renova token expirado
docs: atualiza especificacao de integracao
```

Antes de abrir Pull Request:

- [ ] Migrations incluídas e revisadas.
- [ ] Testes de unidade e integração atualizados.
- [ ] Testes de isolamento de tenant para rotas novas.
- [ ] Nenhum segredo ou dado pessoal sensível em logs/repositório.
- [ ] `cargo fmt`, Clippy, testes Rust e lint/testes/build dos frontends aprovados.
- [ ] SPEC/ROADMAP/README atualizados quando houver mudança arquitetural ou contratual.
- [ ] Trial e Produção testados separadamente, sem fallback silencioso.
