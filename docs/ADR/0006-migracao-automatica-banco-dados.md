# ADR-0006 — Migração Automática do Banco de Dados

- **Status:** Aceito
- **Data:** 2026-10-20
- **Decisores:** Arquitetura Accurit
- **Escopo:** Versionamento e aplicação de migrations no startup do backend
- **Relaciona-se com:** [ADR-0001](0001-escolha-stack-tecnologica.md), [ADR-0005](0005-alternador-modo-trial-serpro.md)

## Contexto

O ambiente de referência da Accurit é Docker Compose. O fluxo `docker compose up --build` deve subir a stack completa sem exigir que o desenvolvedor instale `diesel_cli`, execute comandos manuais ou mantenha o schema sincronizado por procedimento externo.

O uso manual de `diesel setup`/`diesel migration run` traz problemas:

- dependência de ferramenta e versão no host;
- risco de esquecimento ou execução parcial;
- divergência entre ambientes;
- dificuldade de onboarding;
- possibilidade de o backend iniciar com schema incompatível;
- automação de deploy mais complexa.

O Diesel oferece `diesel_migrations::embed_migrations!`, que incorpora as migrations SQL ao binário em tempo de compilação. O binário pode então aplicar as migrations pendentes no startup, antes de abrir o servidor HTTP.

## Decisão

### 1. Migrations embutidas no binário

As migrations permanecem em `backend/migrations/`, no formato aceito pelo Diesel:

```text
backend/migrations/
└── 20261020120000_create_escritorios/
    ├── up.sql
    └── down.sql
```

O crate backend declara:

```rust
use diesel_migrations::{embed_migrations, EmbeddedMigrations};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
```

O recurso `postgres` do `diesel_migrations` deve estar habilitado no `Cargo.toml`.

### 2. Execução automática no startup

O startup do backend segue esta ordem:

```text
1. Ler DATABASE_URL e demais configurações.
2. Conectar ao PostgreSQL com retry/backoff limitado.
3. Adquirir advisory lock de migração.
4. Executar migrations pendentes com MigrationHarness.
5. Liberar o advisory lock.
6. Criar o pool Diesel.
7. Iniciar o servidor Axum em 0.0.0.0:8080.
8. Expor GET /api/v1/health com o estado do banco.
```

Exemplo de referência:

```rust
use diesel::pg::PgConnection;
use diesel::Connection;
use diesel::RunQueryDsl;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use std::time::Duration;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

fn connect_with_retry(database_url: &str, max_attempts: u32) -> PgConnection {
    let mut attempt = 1;

    loop {
        match PgConnection::establish(database_url) {
            Ok(conn) => return conn,
            Err(err) if attempt >= max_attempts => {
                panic!("PostgreSQL indisponível após {attempt} tentativas: {err}");
            }
            Err(_) => {
                std::thread::sleep(Duration::from_secs(2_u64.pow(attempt.min(4))));
                attempt += 1;
            }
        }
    }
}

fn run_migrations(
    database_url: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = connect_with_retry(database_url, 15);

    diesel::sql_query("SELECT pg_advisory_lock(727001)").execute(&mut conn)?;
    let result = conn.run_pending_migrations(MIGRATIONS);
    diesel::sql_query("SELECT pg_advisory_unlock(727001)").execute(&mut conn)?;

    result.map_err(|e| format!("falha ao executar migrations: {e}"))?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = std::env::var("DATABASE_URL")?;

    run_migrations(&database_url)?;

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    let app = axum::Router::new(); // estado, rotas e middlewares reais
    axum::serve(listener, app).await?;
    Ok(())
}
```

O exemplo é ilustrativo; a implementação real deve:

- usar o pool Diesel após as migrations;
- garantir liberação do advisory lock mesmo em erro (RAII/guard ou bloco de cleanup);
- registrar logs estruturados de início, quantidade de migrations aplicadas e falha;
- encerrar o processo com código de erro se qualquer migration falhar.

### 3. Docker Compose e ordem de inicialização

- O serviço `postgres` usa `postgres:16-alpine`, volume persistente e healthcheck com `pg_isready`.
- O serviço `backend` declara dependência de `postgres` com condição `service_healthy`.
- Não existe um serviço/container separado de migrations.
- A imagem do backend é construída com `backend/migrations/` disponível no contexto de build; o `embed_migrations!` incorpora os arquivos no binário.
- O volume do PostgreSQL não é recriado em `docker compose down`; `docker compose down -v` é destrutivo e exige intenção explícita.

### 4. Convenções de migration

- Nome da pasta: `YYYYMMDDHHMMSS_nome_descritivo`.
- Cada migration contém `up.sql` e, quando a reversão for segura, `down.sql`.
- SQL compatível com PostgreSQL 16.
- Toda tabela de negócio inclui `escritorio_id` e índices/constraints de isolamento.
- Migrations aplicadas em ambiente compartilhado são imutáveis: não editar; criar uma nova migration.
- Evitar operações destrutivas sem plano de backup e migração de dados.
- Revisar constraints, índices, locks e impacto em tabelas grandes.
- Preferir a estratégia expand/contract para mudanças incompatíveis em produção.
- Não usar `diesel_cli` como parte do fluxo normal; o fluxo suportado é reconstruir a imagem e reiniciar o backend.
- `backend/src/infrastructure/persistence/schema.rs` é versionado e gerado por uma tarefa de desenvolvimento com container
  descartável (`diesel print-schema`); o CI regenera o schema contra um banco limpo e falha em caso
  de drift. Runtime e deploy não executam `diesel_cli`.
- **Regra pré-MVP (Fases 1 a 6):** é proibido usar `ALTER` (`ALTER TABLE`, `ADD COLUMN`, `DROP COLUMN`,
  `RENAME`, `ALTER TYPE`, `ALTER INDEX` etc.) em qualquer `up.sql` ou `down.sql`. Enquanto o MVP não
  estiver concluído e aceito:
  - tabela nova exige nova migration com `CREATE TABLE`;
  - alteração em tabela existente exige editar o `CREATE TABLE` da migration que a criou e recriar o
    banco local com `docker compose down -v && docker compose up --build`;
  - `down.sql` contém apenas `DROP`.
  A imutabilidade de migrations aplicadas, o expand/contract e a criação de novas migrations para
  mudanças passam a valer somente após o encerramento do MVP. A regra é verificada automaticamente
  por `backend/tests/migrations_policy.rs`.
- O arquivo de configuração do Diesel fica em `backend/diesel.toml` (minúsculo).

### 5. Múltiplas réplicas e concorrência

- O advisory lock `pg_advisory_lock(727001)` serializa a execução entre réplicas.
- A primeira réplica aplica as migrations; as demais aguardam e então verificam que não há migrations pendentes.
- O lock deve ser liberado em caso de erro; se a conexão cair, o PostgreSQL libera o lock.
- Migrations longas aumentam o tempo de startup e podem afetar a disponibilidade; devem ser planejadas e, quando possível, executadas em etapas.
- O health check só deve reportar saudável após as migrations concluídas e o pool pronto.

### 6. Falha e rollback

- Se uma migration falhar, o backend **não inicia parcialmente**. O processo registra o erro e encerra com código diferente de zero.
- Não há rollback automático de migrations no startup. `down.sql` existe para reversão controlada, mas a recuperação padrão é corrigir a migration ou restaurar backup e aplicar uma nova migration.
- O deploy deve ser interrompido e o erro investigado antes de nova tentativa.
- A aplicação não deve tentar “ignorar” uma migration com erro.

## Consequências

### Positivas

- **Um passo, zero Diesel CLI:** `docker compose up --build` é suficiente para criar e atualizar o schema.
- **Determinismo:** o binário carrega exatamente as migrations daquela versão.
- **Sem drift de schema:** ambientes novos e existentes convergem para o mesmo estado.
- **Onboarding simples:** não é necessário instalar ferramentas adicionais no host.
- **Deploy reproduzível:** a imagem contém código e schema versionados juntos.
- **Proteção contra corrida:** o advisory lock evita execução concorrente em múltiplas réplicas.
- **Fail-fast:** schema incompatível impede o servidor de subir com estado incorreto.

### Negativas

- **Rebuild obrigatório:** adicionar ou alterar uma migration exige reconstruir a imagem do backend.
- **Startup mais lento:** conexão, lock e migrations aumentam o tempo até o servidor aceitar tráfego.
- **Falha bloqueia o serviço:** uma migration inválida impede o backend de iniciar; não há degradação parcial.
- **Sem rollback automático:** a recuperação exige correção, backup ou procedimento manual.
- **Privilégios de DDL:** o usuário do banco precisa de permissões para criar/alterar objetos durante a migration.
- **Migrations longas:** podem causar indisponibilidade e conflito com o tempo de healthcheck.
- **Conflitos de merge:** duas branches podem criar migrations com o mesmo timestamp ou ordem ambígua; a revisão precisa ser cuidadosa.
- **Binário maior:** os arquivos SQL são embutidos no executável.

### Mitigações

- Revisar migrations em code review e testar em banco limpo no CI.
- Usar timestamps únicos e sequência clara; resolver conflitos antes do merge.
- Planejar migrations longas com expand/contract e janelas de manutenção.
- Aumentar `start_period`/`timeout` do healthcheck quando necessário.
- Manter backup e procedimento de restauração documentados.
- Monitorar duração do startup e falhas de migration.
- Garantir que o advisory lock seja liberado em todos os caminhos.
- Usar um usuário de aplicação com privilégios mínimos, mas com DDL necessário ao schema; separar credenciais administrativas quando o contrato de segurança exigir.

## Conformidade e Regras de Negócio

1. **Automação obrigatória:** o backend aplica migrations no startup; não existe etapa manual obrigatória de `diesel migration run`.
2. **Sem `diesel_cli` no fluxo normal:** o host e o CI não precisam instalar `diesel_cli`; a documentação não pode instruir `diesel setup` como pré-requisito.
3. **Migrations embutidas:** `embed_migrations!("migrations")` é a fonte das migrations do binário; a pasta `backend/migrations/` é versionada.
4. **Ordem de startup:** conectar com retry, adquirir lock, executar migrations, liberar lock, criar pool, iniciar Axum.
5. **Fail-fast:** falha de migration impede o servidor de iniciar; o processo encerra com erro claro.
6. **Concorrência:** múltiplas réplicas usam advisory lock para evitar execução simultânea.
7. **Regra pré-MVP:** até o fim do MVP (Fases 1 a 6 aceitas), nenhuma migration usa `ALTER`; mudanças
   em tabela existente editam o `CREATE TABLE` da migration de origem e exigem recriar o banco local.
   Após o MVP, valem a imutabilidade de migrations aplicadas e o expand/contract.
8. **Reversão:** `down.sql` existe em toda migration e contém apenas `DROP` durante o pré-MVP; rollback automático não faz parte do startup.
9. **Isolamento:** migrations de tabelas de negócio incluem `escritorio_id`, constraints e índices coerentes com o modelo multi-tenant.
10. **Saúde:** `/api/v1/health` só reporta saudável após migrations e pool prontos.
11. **Deploy:** a imagem é reconstruída a cada mudança de migration; o Compose depende de PostgreSQL saudável.
12. **Segurança:** credenciais de banco vêm de ambiente/secret manager; nenhuma credencial é embutida na imagem ou no repositório.

## Critérios de verificação

- `docker compose up --build` sobe o PostgreSQL, aplica migrations e inicia o backend sem `diesel_cli`.
- Um banco vazio recebe todas as migrations na ordem correta.
- Uma migration inválida impede o backend de iniciar e gera erro claro.
- Duas réplicas simultâneas não aplicam a mesma migration em paralelo.
- `/api/v1/health` responde somente após o schema estar pronto.
- Novas migrations são embutidas na imagem após rebuild.
- O fluxo documentado não exige `diesel setup`, `diesel migration run` ou instalação de `diesel_cli`.
- Testes de CI criam um banco limpo e executam as migrations embutidas.

## Referências

- [INSTRUCTIONS — Migrations](../../INSTRUCTIONS.md)
- [README — Quickstart](../../README.md)
- [SPEC — Startup e migrations](../../SPEC.md)
- [ROADMAP — Fase 1](../../ROADMAP.md)
- [ADR-0001 — Escolha da stack tecnológica](0001-escolha-stack-tecnologica.md)
- [ADR-0005 — Alternador do modo Trial SERPRO](0005-alternador-modo-trial-serpro.md)
