# INSTRUCTIONS.md — Setup, Migrations, Troubleshooting e Contribuição

> Guia operacional da plataforma **Accurit** para desenvolvedores, DevOps e contribuidores. Stack: Rust 2024/Axum 0.8, Diesel 2.2+, PostgreSQL 16, Angular 21, Next.js 16 e Docker Compose.

---

## 1. Pré-requisitos

| Ferramenta | Versão mínima | Uso |
|---|---|---|
| Docker Engine | 24.x | Build e execução dos containers |
| Docker Compose | v2.20+ | Orquestração local |
| Git | 2.x | Controle de versão |
| Node.js | 20 LTS | Desenvolvimento dos frontends |
| npm | 10+ | Dependências dos frontends |
| Rust toolchain | 1.85+ (edition 2024) | Desenvolvimento local opcional do backend |
| PostgreSQL client | 16 (opcional) | Inspeção manual do banco |

Instalação (referências oficiais):

- Docker Engine/Desktop + Compose plugin: <https://docs.docker.com/get-docker/>.
- Rust via `rustup` (edition 2024): <https://rustup.rs/>.
- Node.js 20 LTS via `nvm` ou instalador oficial: <https://nodejs.org/>.

> **Não é necessário instalar `diesel_cli` no host.** O fluxo normal não exige Rust nem Angular CLI instalados localmente: tudo roda via Docker Compose.

---

## 2. Setup rápido via Docker Compose

```bash
git clone https://github.com/Siricode-BR/Accurit.git
cd Accurit
cp .env.example .env
# Edite .env com as credenciais do ambiente.
docker compose up --build
```

No startup, o container `accurit-backend`:

1. aguarda o PostgreSQL (`accurit-postgres`) ficar saudável, com retry/backoff;
2. executa as migrations embutidas no binário (`diesel_migrations::embed_migrations!`);
3. inicia o servidor Axum na porta interna `8080`.

Acessos locais:

| Serviço | URL |
|---|---|
| Web (site institucional) | `http://localhost/` |
| Admin | `http://localhost/admin/` |
| Login do cliente | `http://localhost/login` |
| Portal do cliente | `http://localhost/app/` |
| Health check | `http://localhost/api/v1/health` |

Comandos úteis:

```bash
docker compose up --build -d      # segundo plano
docker compose logs -f backend    # logs da API
docker compose logs -f proxy      # logs do Nginx
docker compose down               # encerra preservando volumes
docker compose down -v            # encerra e apaga o banco (destrutivo)
```

---

## 3. Variáveis de ambiente

Copie `.env.example` para `.env` e preencha os valores. Nunca versione `.env`, certificados, tokens ou credenciais reais.

| Variável | Obrigatória | Descrição |
|---|---:|---|
| `DATABASE_URL` | Sim | `postgres://usuario:senha@postgres:5432/accurit` |
| `POSTGRES_USER` / `POSTGRES_PASSWORD` / `POSTGRES_DB` | Sim | Credenciais do container PostgreSQL |
| `APP_ENV` | Sim | `development`, `staging` ou `production` |
| `JWT_SECRET` | Sim | Segredo de assinatura dos tokens de sessão |
| `CERTIFICATE_MASTER_KEY` | Sim | Chave AES-256-GCM de 32 bytes para certificados A1, credenciais e tokens; valor de exemplo deve falhar no startup |
| `SERPRO_TOKEN_URL` | Sim | `https://gateway.apiserpro.serpro.gov.br/token` |
| `SERPRO_SAPI_AUTH_URL` | Produção | `https://autenticacao.sapi.serpro.gov.br/authenticate` |
| `SERPRO_INTEGRA_TRIAL_URL` | Sim | Base Trial do Integra Contador |
| `SERPRO_DIVIDA_ATIVA_TRIAL_URL` | Sim | Base Trial da Consulta Dívida Ativa |
| `SERPRO_INTEGRA_PROD_URL` | Produção | `https://gateway.apiserpro.serpro.gov.br/integra-contador/v1` |
| `SERPRO_DIVIDA_ATIVA_PROD_URL` | Produção | Indefinida; vazia bloqueia com `SERPRO_PROD_NOT_CONFIGURED` |
| `SERPRO_TRIAL_BEARER_TOKEN` | Trial | Chave Bearer pública de demonstração fornecida pela SERPRO |
| `SERPRO_CONSUMER_KEY` / `SERPRO_CONSUMER_SECRET` | Não | Fallback global de desenvolvimento; produção usa `credenciais_serpro` por escritório |
| `SERPRO_OPERATOR_HEADER_ENABLED` | Não | `true` envia `x-cpf-usuario`; header não confirmado oficialmente |
| `DIVIDA_ATIVA_FALLBACK_CODIGO_CONTRIBUINTE` | Não | `false` por padrão; habilita fallback não confirmado |
| `SERPRO_HTTP_CONNECT_TIMEOUT_SECS` | Não | Timeout de conexão (padrão 5) |
| `SERPRO_HTTP_REQUEST_TIMEOUT_SECS` | Não | Timeout de requisição (padrão 30) |
| `SERPRO_PROCURACAO_CACHE_TTL_SECS` | Não | TTL do cache de procuração (padrão 300) |
| `RUST_LOG` | Não | Nível de log (`info`, `debug`, `warn`) |

Exemplo seguro de `.env.example` (sem valores reais):

```dotenv
APP_ENV=development
DATABASE_URL=postgres://accurit:accurit@postgres:5432/accurit
POSTGRES_USER=accurit
POSTGRES_PASSWORD=change-me
POSTGRES_DB=accurit

JWT_SECRET=change-me
# Substitua por 32 bytes aleatórios; o valor de exemplo falha no startup.
CERTIFICATE_MASTER_KEY=change-me-32-bytes

SERPRO_TOKEN_URL=https://gateway.apiserpro.serpro.gov.br/token
SERPRO_SAPI_AUTH_URL=https://autenticacao.sapi.serpro.gov.br/authenticate
SERPRO_INTEGRA_TRIAL_URL=https://gateway.apiserpro.serpro.gov.br/integra-contador-trial/v1
SERPRO_DIVIDA_ATIVA_TRIAL_URL=https://gateway.apiserpro.serpro.gov.br/consulta-divida-ativa-trial/api/v1
SERPRO_INTEGRA_PROD_URL=https://gateway.apiserpro.serpro.gov.br/integra-contador/v1
SERPRO_DIVIDA_ATIVA_PROD_URL=
SERPRO_TRIAL_BEARER_TOKEN=
SERPRO_CONSUMER_KEY=
SERPRO_CONSUMER_SECRET=
SERPRO_OPERATOR_HEADER_ENABLED=true
DIVIDA_ATIVA_FALLBACK_CODIGO_CONTRIBUINTE=false
SERPRO_HTTP_CONNECT_TIMEOUT_SECS=5
SERPRO_HTTP_REQUEST_TIMEOUT_SECS=30
SERPRO_PROCURACAO_CACHE_TTL_SECS=300

RUST_LOG=info
```

A chave de trial é configuração de ambiente e **não** deve ser embutida no código, no bundle Angular ou no repositório. O mesmo vale para `Consumer Secret`, tokens e certificados A1.
`CERTIFICATE_MASTER_KEY` deve ter exatamente 32 bytes; o placeholder `change-me-32-bytes` deve ser
rejeitado no startup com mensagem clara.

---

## 4. Containers, portas e rotas

| Serviço | Container | Porta interna | Observação |
|---|---|---:|---|
| `postgres` | `accurit-postgres` | 5432 | Volume persistente e healthcheck |
| `backend` | `accurit-backend` | 8080 | Axum + migrations automáticas |
| `web` | `accurit-web` | 3000 | Next.js standalone |
| `admin` | `accurit-admin` | 80 | Angular + Nginx interno |
| `client` | `accurit-client` | 80 | Angular + Nginx interno |
| `proxy` | `accurit-proxy` | 80/443 | Ponto de entrada público |

Roteamento no proxy:

| Rota | Destino |
|---|---|
| `/` e `/_next/` | `web:3000` |
| `/admin/` | `admin:80` |
| `/app/` | `client:80` |
| `/login` | `301` para `/app/login` |
| `/api/` | `backend:8080` |

Em produção, apenas o proxy publica portas. Backend, banco e frontends ficam em rede interna.

---

## 5. Como criar migrations SQL

As migrations ficam em `backend/migrations/` e são embutidas no binário Rust em tempo de compilação por `diesel_migrations::embed_migrations!`.

Crie manualmente uma pasta versionada no padrão aceito pelo Diesel, com `up.sql` e `down.sql`:

```text
backend/migrations/
└── 20261020120000_create_exemplo/
    ├── up.sql
    └── down.sql
```

`up.sql`:

```sql
CREATE TABLE exemplo (
    id UUID PRIMARY KEY,
    escritorio_id UUID NOT NULL REFERENCES escritorios(id),
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_exemplo_escritorio ON exemplo(escritorio_id);
```

`down.sql`:

```sql
DROP TABLE exemplo;
```

### Fluxo de trabalho

1. Crie a pasta com timestamp no padrão do repositório (`YYYYMMDDHHMMSS_nome`).
2. Escreva SQL compatível com PostgreSQL 16.
3. Reconstrua o backend para embutir a migration:

   ```bash
   docker compose up --build
   ```

4. Confirme nos logs que a migration foi aplicada:

   ```bash
   docker compose logs -f backend
   ```

### Regras obrigatórias

- Sempre incluir `up.sql` e `down.sql` quando a reversão for segura.
- Nunca editar uma migration já aplicada em ambiente compartilhado; crie uma nova.
- Revisar constraints, índices e isolamento por `escritorio_id`.
- Evitar operações destrutivas sem plano de migração e backup.
- Se a migration falhar, o backend deve encerrar com erro claro — nunca iniciar parcialmente.
- Em múltiplas réplicas, o startup usa advisory lock para evitar execução concorrente.
- **Não** rodar `diesel setup`, `diesel migration run` ou instalar `diesel_cli` como parte do fluxo normal.

O crate backend declara:

```rust
pub const MIGRATIONS: diesel_migrations::EmbeddedMigrations =
    diesel_migrations::embed_migrations!("migrations");
```

O startup usa `MigrationHarness::run_pending_migrations` (ou API equivalente da versão do Diesel) após aguardar o banco com retry limitado.

O arquivo de configuração do Diesel é `backend/diesel.toml` (minúsculo). `backend/schema.rs` é
versionado, gerado em tarefa de desenvolvimento com container descartável (`diesel print-schema`) e
verificado contra drift no CI; runtime e deploy não executam `diesel_cli`.

---

## 6. Desenvolvimento local opcional

Docker Compose é o caminho recomendado. Para desenvolver um serviço isoladamente:

```bash
# Banco e dependências
docker compose up -d postgres

# Backend local (Rust 1.85+ e .env configurados)
cd backend
cargo run
```

Frontends:

```bash
cd admin    # ou client / web
npm ci
npm run start
```

- Configure as URLs de API e CORS para o ambiente local.
- O Admin espera a API em `/api` via proxy de desenvolvimento ou variável de ambiente.
- Nunca exponha serviços internos sem autenticação.
- Scripts podem variar por `package.json`; consulte cada projeto.

---

## 7. Padrões de arquitetura e código

### 7.1 Backend Rust — DDD

Para cada funcionalidade:

1. `domain/<feature>/`: entidades, value objects, invariantes e traits; sem Axum, Diesel ou Reqwest.
2. `application/`: casos de uso que orquestram as abstrações do domínio.
3. `infrastructure/`: repositórios Diesel, clientes HTTP SERPRO e criptografia.
4. `presentation/`: rotas Axum, handlers, DTOs e validação.

Convenções:

- Handlers enxutos: validação de entrada, chamada do caso de uso e resposta.
- `escritorio_id` vem do `RequestContext`; nunca do corpo/query.
- Erros de domínio tipados; sem `unwrap()` em caminho de produção.
- `serde` apenas na borda; domínio não depende de DTOs HTTP.
- Valores monetários usam `rust_decimal::Decimal`, nunca `f32`/`f64`.
- Toda chamada SERPRO inclui correlação; `x-cpf-usuario` é operacional e só é enviado quando
  `SERPRO_OPERATOR_HEADER_ENABLED=true`.

### 7.2 Angular — Clean Architecture por feature

- Criar `features/<nome>/` com `domain/`, `application/`, `data/` e `presentation/` quando necessário.
- Usar Signals para estado de UI e `computed` para valores derivados.
- Manter `HttpClient` nos adaptadores da camada `data` e interceptors transversais.
- Usar Standalone Components e guards por papel.
- Tratamento consistente de 401/403 e estados de loading/erro/vazio.
- O toggle Trial é apenas apresentação; o backend é a autoridade.

### 7.3 Integrações SERPRO

- Separar configuração de Trial e Produção por produto e por escritório.
- Não presumir que autenticação ou envelopes de produtos diferentes são intercambiáveis: Dívida
  Ativa usa OAuth2; Integra Contador em produção usa SAPI com mTLS e-CNPJ, `access_token` e
  `jwt_token`.
- Validar procuração via `PROCURACOES / OBTERPROCURACAO41` antes de serviço protegido **somente
  quando `autorPedidoDados` difere do contribuinte**.
- Higienizar CPF/CNPJ e moeda antes de serializar; usar `parse_moeda_ptbr_estrito` para respostas da
  API e `parse_moeda_entrada_usuario` para entrada de usuário.
- Assinar XML com `<Signature>`/`<Transform>` no namespace default (sem prefixo `ds:`) e validar a
  cadeia ICP-Brasil (RSA-SHA256, SHA-256 e EndCertOnly).
- Enviar `Authorization` e, quando o contrato exigir, `jwt_token`/`autenticar_procurador_token`/
  `x-jwt-assertion` — sempre injetados no backend. `x-cpf-usuario` é opcional e controlado por
  `SERPRO_OPERATOR_HEADER_ENABLED`.
- Validar o catálogo oficial antes de usar `idSistema`/`idServico` (ex.: `CONSDECLARACAO13`, `TRANSDECLARACAO11`, `INNOVAMSG63`); `RELATORIOSITFIS92` é **Emitir**.
- Conferir os códigos de procuração: PGDASD `00146`, DCTFWeb `00103`, SITFIS `00002`, Caixa Postal `00006`, DTE `00050`, PAGTOWEB `00004`, REGIMEAPURACAO `00060`, DASNSIMEI `00229`, EPROCESSO `00051`.
- Tratar `202` (`tempoEspera`), `204` (`eTag`) e `304` (`etag` + `expires`) com polling controlado.
- Tratar HTTP 504 como estado indeterminado: não reenviar imediatamente; consultar o status antes de repetir.
- Dívida Ativa é exclusivamente consultiva: não criar fluxo de parcelamento, negociação, cobrança ou DARF de dívida ativa.
- Não repetir automaticamente `Declarar`/`Emitir` sem idempotência.

---

## 8. Troubleshooting

### 8.1 Erros de integração SERPRO

| Sintoma | Causa provável | Ação |
|---|---|---|
| HTTP 401 com mensagem de usuário não identificado | Credencial/token inválido; o header `x-cpf-usuario` **não** é documentado oficialmente | Conferir token/credencial e, se `SERPRO_OPERATOR_HEADER_ENABLED=true`, o CPF do operador; validar o comportamento real em teste live (D3) |
| HTTP 401 “invalid token” / token expirado | Token vencido, cache inconsistente ou credenciais incorretas | Invalidar o cache, renovar via `POST /token`, conferir `Consumer Key`/`Secret` |
| HTTP 401 de certificado / handshake mTLS | A1 expirado, cadeia ICP-Brasil inválida ou certificado trocado | Validar validade, fingerprint e cadeia; recadastrar o A1 do escritório |
| HTTP 403 de autorização | Procuração ausente, vencida, cancelada ou código errado | Consultar `OBTERPROCURACAO41` (somente se o autor difere do contribuinte) e conferir o código (`00146`, `00103`, `00002`, `00006` Caixa Postal, `00050` DTE, `00004`, `00060`, `00229`, `00051`) |
| `AcessoNegado-ICGERENCIADOR-003` | Header `x-jwt-assertion` ausente (fluxo de loja/e-commerce) | Renovar a autenticação na loja e enviar o token válido |
| `AcessoNegado-ICGERENCIADOR-019` | Autor do pedido diferente do contratante sem Termo de Autorização | Enviar o termo assinado via `ENVIOXMLASSINADO81` antes da operação |
| `AcessoNegado-ICGERENCIADOR-020` | `autenticar_procurador_token` ausente ou expirado | Renovar o token via `ENVIOXMLASSINADO81` e enviá-lo no header |
| `AcessoNegado-ICGERENCIADOR-022` | Autor do pedido sem procuração autorizada no eCAC | Solicitar ao contribuinte a outorga da procuração no portal eCAC |
| `AcessoNegado-ICGERENCIADOR-042` | `autenticar_procurador_token` com formato/tamanho inválido | Enviar o token de 36 bytes retornado pelo `ENVIOXMLASSINADO81` |
| `EntradaIncorreta-ICGERENCIADOR-052` | `idSistema`/`idServico` inexistente no catálogo | Corrigir os identificadores conforme o catálogo vigente |
| `Erro-{IdSistema}-058` (HTTP 504) | Timeout do gateway; estado indeterminado | Não reenviar imediatamente; consultar o status e reenviar somente se confirmado que não houve recebimento |
| HTTP 429 | Rate limit do gateway SERPRO | Respeitar `Retry-After`, aplicar backoff com jitter e revisar o limite por escritório |
| HTTP 5xx / timeout | Indisponibilidade transitória do SERPRO | Registrar correlação, aplicar retry somente em operações idempotentes e alertar |
| HTTP 400 de envelope | `idSistema`, `idServico`, `versaoSistema` ou `dados` inválidos | Conferir catálogo/contrato; `dados` deve ser string JSON, não objeto |
| Consulta vazia mesmo com CPF/CNPJ correto | Devedor sem inscrições ou comportamento de `200` vazio/`404` não confirmado | Verificar a fonte registrada em `consultas_fiscais`; o fallback por Código de Contribuinte é experimental e desabilitado por padrão (D2) |
| XML rejeitado por assinatura | Digest/canonicalização incorretos ou certificado inválido | Recalcular o digest, validar `<Transform>` no namespace default (sem `ds:`) e a cadeia ICP-Brasil |
| `202`/`204`/`304` inesperado | Processamento assíncrono, `eTag` ou token do Termo ainda válido | Aguardar `tempoEspera`/`eTag`, reutilizar o token do Termo e evitar polling desnecessário (cada consulta pode ser bilhetada) |
| Resultado inesperado após alternar o toggle | Ambiente Trial/Produção divergente entre frontend e backend | Confirmar `modo_trial_serpro` no banco e reler o ambiente no backend |

### 8.2 Erros de infraestrutura

| Sintoma | Causa provável | Ação |
|---|---|---|
| Backend reinicia em loop | PostgreSQL indisponível ou `DATABASE_URL` inválida | Verificar healthcheck do `accurit-postgres` e a URL |
| Migration falha no startup | SQL inválido, constraint ou migration já aplicada | Ler o log, corrigir `up.sql` e criar nova migration se já aplicada |
| `nginx` retorna 502 | Backend ainda não subiu ou porta incorreta | Aguardar health check; conferir `backend:8080` |
| 404 em `/admin/` ou `/app/` | Falta fallback de SPA no Nginx | Configurar `try_files $uri $uri/ /index.html` |
| Erro de CORS no frontend | Origem não permitida ou chamada direta à API | Usar o proxy `/api` e restringir `allowed_origins` |
| Porta 80/5432 ocupada | Outro serviço no host | Liberar a porta ou ajustar o mapeamento no Compose |
| Certificado não decifra | `CERTIFICATE_MASTER_KEY` alterada ou ausente | Restaurar a chave correta; recadastrar o A1 se necessário |
| Segredo aparece no bundle | Variável exposta ao build Angular/Next.js | Mover para o backend e usar apenas variáveis públicas quando indispensável |

---

## 9. Testes e qualidade

```bash
# Backend
cd backend
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo audit

# Frontends
cd admin    # repetir em client e web
npm ci
npm run lint
npm test
npm run build
```

Checklist mínimo:

- Testes de isolamento multi-tenant e IDOR em rotas novas.
- Mocks de SERPRO para token OAuth2, SAPI, envelopes, procuração, Trial, Dívida Ativa e `202`/`204`/`304`.
- Testes de sanitização (CPF/CNPJ/moeda) e fallback por Código de Contribuinte desabilitado por padrão.
- Testes de assinatura XML (namespace default, sem `ds:`) e validação ICP-Brasil.
- Nenhum segredo, token, certificado ou dado pessoal em logs/repositório.
- Trial e Produção testados separadamente, sem fallback silencioso.

---

## 10. Contribuição

### 10.1 Commits — Conventional Commits

```text
feat(divida-ativa): consulta por inscricao
feat(integra-contador): suporte ao verbo monitorar
feat(clientes): adiciona filtro por regime
fix(serpro): renova token expirado
fix(seguranca): higieniza cpf antes do envelope
docs: update references to Accurit
docs: detalha fallback por codigo de contribuinte
test(procuracoes): cobre procuracao vencida
chore(ci): adiciona cargo audit ao pipeline
```

Regras:

- Assunto no imperativo, curto e específico; escopo entre parênteses.
- Um commit por mudança lógica; não misturar refatoração e feature.
- Breaking changes: `feat!: ...` ou rodapé `BREAKING CHANGE:`.
- Referenciar issues quando aplicável (`Refs #123`).

### 10.2 Branches

```text
feat/<tema>      # nova funcionalidade
fix/<tema>       # correção
docs/<tema>      # documentação
chore/<tema>     # infraestrutura, build, CI
```

### 10.3 Antes de abrir Pull Request

- [ ] Migrations incluídas, revisadas e testadas (`up`/`down`).
- [ ] Testes de unidade e integração atualizados.
- [ ] Testes de isolamento de tenant e IDOR para rotas novas.
- [ ] Nenhum segredo, certificado, token ou dado pessoal em logs/repositório.
- [ ] `cargo fmt`, Clippy, testes Rust, lint/testes/build dos frontends aprovados.
- [ ] `SPEC.md`, `ROADMAP.md`, `README.md` e `INSTRUCTIONS.md` atualizados quando houver mudança arquitetural, contratual ou de setup.
- [ ] Trial e Produção testados separadamente, sem fallback silencioso.
- [ ] Mudanças de segurança (A1, mTLS, `x-cpf-usuario`, XMLDSig) revisadas por um segundo contribuidor.

### 10.4 Convenções de código

- Rust: `rustfmt` obrigatório, Clippy sem warnings, erros tipados e sem `unwrap()` em produção.
- Angular: Standalone Components, Signals, `computed` para derivados e `HttpClient` restrito à camada `data`.
- Next.js: App Router, componentes de servidor por padrão e segredos apenas no servidor.
- SQL: nomes em `snake_case`, migrations versionadas e índices iniciando por `escritorio_id`.
- Documentação: manter os quatro arquivos consistentes entre si; toda nova regra de negócio deve ter reflexo em `SPEC.md`.

---

## 11. Suporte e referências

- [README.md](README.md) — visão geral, arquitetura e quickstart.
- [SPEC.md](SPEC.md) — contratos, DTOs, modelo de dados e segurança.
- [ROADMAP.md](ROADMAP.md) — fases e critérios de aceite.
- [Documentação oficial SERPRO](https://www.serpro.gov.br/) — catálogo, contratos e credenciais vigentes.
- [Diesel Migrations](https://diesel.rs/guides/migration_guide) — referência de migrations.
- [Axum](https://github.com/tokio-rs/axum) — documentação do framework.
