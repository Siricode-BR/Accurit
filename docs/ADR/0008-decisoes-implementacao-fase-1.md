# ADR-0008 — Decisões de Implementação da Fase 1

- **Status:** Aceito
- **Data:** 2026-10-21
- **Decisores:** Arquitetura Accurit
- **Escopo:** Fundação do backend, banco, build e testes
- **Relaciona-se com:** [ADR-0001](0001-escolha-stack-tecnologica.md), [ADR-0006](0006-migracao-automatica-banco-dados.md)

## Contexto

A Fase 1 precisava entregar a stack executável com migrations automáticas, isolamento inicial por
`escritorio_id`, health check e testes reproduzíveis, sem antecipar autenticação, JWT ou integrações
SERPRO. As escolhas abaixo equilibram simplicidade operacional, segurança de privilégios e
capacidade de evolução para a Fase 2.

## Decisões

### F1 — Diesel síncrono com `spawn_blocking`

O backend usa Diesel síncrono com pool `r2d2`. Handlers e casos de uso acessam o banco por traits na
camada `application`; a infraestrutura executa o trabalho bloqueante em
`tokio::task::spawn_blocking`. Não há `diesel-async` nem pool assíncrono nesta fase.

**Motivo:** o ecossistema Diesel e as migrations são síncronos; `spawn_blocking` mantém o runtime
Tokio responsivo sem introduzir uma segunda abstração de banco. A porta de repositório permite migrar
para `diesel-async` no futuro sem alterar domínio ou apresentação.

**Consequências:** cada operação bloqueante precisa respeitar o limite do pool; testes e handlers
assíncronos não devem bloquear o reactor. O health check consulta o pool de forma controlada.

### F2 — Superusuário administrativo e usuário de aplicação

O container PostgreSQL cria:

- `POSTGRES_USER`: superusuário do container, usado apenas para administração e para os serviços do
  profile `tools` (`schema-tool` e `backend-test`);
- `POSTGRES_APP_USER`: role `LOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE`, com `CONNECT` no banco e
  `CREATE`/`USAGE` no schema `public`, usada pela `DATABASE_URL` do backend e pelas migrations.

O split entre migrator e runtime (roles distintas) fica adiado; a separação relevante nesta fase é
entre o superusuário administrativo e a role de aplicação sem privilégios de cluster.

**Motivo:** migrations exigem DDL em `public`, mas a aplicação não deve conectar como superusuário.
O init script do Postgres garante a role antes do primeiro startup do backend.

**Consequências:** `DATABASE_URL` nunca usa `POSTGRES_USER`; `TEST_DATABASE_ADMIN_URL` é exclusiva do
profile `tools`; alterações futuras de privilégios exigem recriar o volume ou aplicar o ajuste
manualmente em ambientes existentes.

### F3 — Linkagem dinâmica com `libpq` no Alpine

O builder usa `RUSTFLAGS="-C target-feature=-crt-static"` e `postgresql-dev`; o runtime instala
`libpq` e as bibliotecas SSL. A imagem final não é estática.

**Motivo:** a linkagem estática de `libpq` em musl falhou com símbolos como `pg_vsnprintf`. A
linkagem dinâmica é suportada pelo Alpine, reduz o binário e mantém o runtime enxuto.

**Consequências:** o runtime precisa das bibliotecas compartilhadas; a validação inclui `ldd` e o
health check real do container.

### F4 — Crate `accurit_backend` com binário `backend`

O `Cargo.toml` declara `[lib] name = "accurit_backend"` e `[[bin]] name = "backend"`, ambos em
edition 2024. `main.rs` é um bootstrap fino que consome a biblioteca.

**Motivo:** testes de integração e módulos internos usam a biblioteca; o nome curto do processo
facilita logs, Docker e operação.

**Consequências:** novos módulos devem ser expostos por `lib.rs`; o binário não deve concentrar
regra de negócio.

### F5 — Chave abstrata de rate limit

A abstração `RateLimitKey` define a chave do rate limit. Na Fase 1, a implementação usa o IP de
`X-Real-IP` (preenchido pelo proxy) com fallback para uma chave desconhecida. O health check é
isento.

**Motivo:** ainda não existe autenticação nem `escritorio_id` confiável. A abstração evita reescrever
o middleware quando a Fase 2 introduzir a chave por tenant.

**Consequências:** o limite é 30 requisições/s com burst 60; `429` inclui `Retry-After: 1`; a chave
por IP é provisória e não substitui limites por escritório nas fases seguintes.

### F6 — Testes de integração em container

Os testes de integração rodam no serviço `backend-test` (profile `tools`). Cada teste cria um banco
descartável usando `TEST_DATABASE_ADMIN_URL`, aplica as migrations embutidas e remove o banco ao
final. O serviço usa as redes `edge` (para baixar crates) e `data` (para o PostgreSQL).

**Motivo:** o ambiente de referência é Docker Compose; rodar os testes no mesmo runtime do backend
elimina divergências de toolchain e permite validar migrations, constraints e HTTP contra
PostgreSQL real.

**Consequências:** o primeiro run baixa o índice de crates; o profile `tools` não sobe no
`docker compose up` padrão; os testes exigem o superusuário apenas para criar/remover bancos.

## Alternativas consideradas

- **`diesel-async` desde a Fase 1:** rejeitado por complexidade e por desvio do ecossistema estável
  de migrations; a porta de repositório preserva a opção futura.
- **Role única de aplicação com superusuário:** rejeitada por excesso de privilégio e por dificultar
  a separação futura entre migrator e runtime.
- **Imagem estática:** rejeitada após falha de linkagem do `libpq` em musl; a dinâmica é suficiente
  para o runtime Alpine.
- **Testes apenas no host:** rejeitados porque não reproduzem o build e as dependências do container.

## Critérios de verificação

- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` e `cargo test --locked` passam.
- `docker compose --profile tools run --rm backend-test` passa com 29 testes de unidade e as suítes
  de migrations e HTTP.
- `docker compose --profile tools run --rm schema-tool` gera `schema.rs` formatado sem drift.
- `docker compose up --build` sobe todos os serviços saudáveis e aplica as três migrations.
- `scripts/smoke.ps1` e `scripts/smoke.sh` passam pelo proxy.
- A imagem de runtime contém `libpq` dinâmica e não conecta como superusuário.

## Referências

- [ADR-0006 — Migração automática do banco de dados](0006-migracao-automatica-banco-dados.md)
- [SPEC — Startup e migrations](../../SPEC.md)
- [INSTRUCTIONS — Testes e qualidade](../../INSTRUCTIONS.md)
- [ROADMAP — Fase 1](../../ROADMAP.md)
