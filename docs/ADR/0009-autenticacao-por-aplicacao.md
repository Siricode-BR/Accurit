# ADR-0009 — Autenticação própria por aplicação (Painel e Portal)

- **Status:** Aceito — parcialmente suplantado: a decisão A1 (papéis por audiência) e o exemplo de
  bootstrap do A5 foram revisados pelo [ADR-0010](0010-revisao-atores-papeis-e-provisionamento.md).
  A mecânica desta decisão permanece válida.
- **Data:** 2026-10-21
- **Decisores:** Arquitetura Accurit
- **Escopo:** Fase 2A — login, sessão, renovação e logout do Admin (`/admin/`) e do Client (`/app/`)
- **Relaciona-se com:** [ADR-0001](0001-escolha-stack-tecnologica.md), [ADR-0005](0005-alternador-modo-trial-serpro.md), [ADR-0006](0006-migracao-automatica-banco-dados.md), [ADR-0007](0007-modelo-contratacao-serpro.md)

## Contexto

A plataforma tem dois frontends autenticados com audiências distintas: o painel do escritório
(`admin/`, papéis `ADMIN`/`OPERADOR`) e o portal do cliente (`client/`, papel `CLIENTE`). Ambos
compartilham o mesmo backend Axum e o mesmo banco multi-tenant.

A Fase 2A precisava entregar autenticação própria e completa — sem delegar a um provedor externo —
com quatro exigências: sessões independentes por aplicação (um `CLIENTE` não pode usar o painel e um
token do painel não pode valer no portal), tokens de acesso de vida curta, renovação silenciosa sem
novo login, e proteção contra roubo de refresh token e força bruta.

A numeração prevista em `docs/DECISOES-PENDENTES.md` apontava "ADR-0008" para a decisão de
autenticação; o 0008 foi consumido pela Fase 1 (decisões de implementação), então esta decisão passou
a ser o ADR-0009.

## Decisões

### A1 — Separação por audiência no contrato HTTP

As rotas de autenticação são agrupadas por audiência e montadas sob `/api/v1/auth`:

```text
POST /api/v1/auth/admin/login     POST /api/v1/auth/client/login
POST /api/v1/auth/admin/refresh   POST /api/v1/auth/client/refresh
POST /api/v1/auth/admin/logout    POST /api/v1/auth/client/logout
GET  /api/v1/auth/admin/me        GET  /api/v1/auth/client/me
```

O JWT carrega a claim `aud` (`admin` ou `client`) e o middleware `ExigirAudiencia` rejeita token de
uma audiência em rota da outra. O papel exigido também é fixo por audiência: `ADMIN`/`OPERADOR` no
painel, `CLIENTE` no portal. O login do portal rejeita usuário sem papel `CLIENTE` e vice-versa.

**Motivo:** auditar e evoluir painel e portal de forma independente; impedir escalada de privilégio
por reuso de token entre aplicações; permitir TTLs de sessão distintos por perfil de uso.

### A2 — Access token JWT e refresh token opaco com rotação

- **Access token:** JWT HS256 com `sub`, `escritorio_id`, `role`, `aud`, `iat`, `exp` e `jti`; sem
  PII; validade padrão de 900 s (`AUTH_ACCESS_TTL_SECS`).
- **Refresh token:** valor aleatório de 256 bits, opaco para o cliente; o banco guarda apenas o
  hash SHA-256 (`refresh_hash CHAR(64) UNIQUE`).
- **Rotação:** cada renovação invalida o token apresentado e emite um novo, mantendo a
  `familia_id`. O reuso de um token já rotacionado revoga toda a família, audita
  `AUTH_REFRESH_REUSO` e exige novo login.
- TTLs distintos: painel 8 h (`AUTH_REFRESH_TTL_ADMIN_SECS=28800`), portal 7 dias
  (`AUTH_REFRESH_TTL_CLIENT_SECS=604800`).

**Motivo:** detecção de roubo de refresh por construção (rotação + família); minimizar impacto de
vazamento do access token com vida curta; permitir revogação real no servidor.

### A3 — Refresh em cookie `HttpOnly` com dupla proteção CSRF

O refresh viaja em cookie `HttpOnly; SameSite=Strict; Path=/api/v1/auth/{admin|client}`, com `Secure`
controlado por `AUTH_COOKIE_SECURE` (padrão `true`; `false` aceito apenas com `APP_ENV=development`).
`POST` em `refresh`/`logout` exige `Content-Type: application/json` e o header
`X-Requested-With: accurit`; sem isso responde `400 AUTH_CSRF_HEADER_MISSING`. O access token nunca é
persistido em `localStorage`; fica apenas em memória (`signal`) no frontend.

**Motivo:** `HttpOnly` remove o refresh do alcance de XSS; `SameSite=Strict` + header obrigatório
cobrem CSRF sem token extra; a distinção de contexto (`X-Requested-With`) também separa chamadas de
API de navegações.

### A4 — Antiforça-bruta e resposta uniforme

- Hash de senha com Argon2id (`AUTH_ARGON2_MEMORY_KIB=19456`, `AUTH_ARGON2_ITERATIONS=2`,
  `AUTH_ARGON2_PARALLELISM=1` por padrão).
- Falha de login sempre responde `401 AUTH_INVALID_CREDENTIALS` (hash fictício para usuário
  inexistente; sem enumeração de e-mail/escritório).
- Lockout após 5 falhas consecutivas por 900 s (`AUTH_MAX_FAILED_ATTEMPTS`, `AUTH_LOCKOUT_SECS`),
  com auditoria dos motivos `USUARIO_INEXISTENTE`, `USUARIO_INATIVO` e `USUARIO_BLOQUEADO`.
- Rate limit de login no backend e no proxy: 10 r/m com burst 5, chave `IP|rota|slug`.
- `ativo = FALSE` invalida refresh imediatamente; a auditoria registra `AUTH_LOGIN_SUCESSO`,
  `AUTH_LOGIN_FALHA`, `AUTH_LOGOUT` e `AUTH_REFRESH_REUSO` sem senha nem token.

### A5 — Bootstrap do primeiro usuário por CLI, sem cadastro público

Não existe endpoint público de cadastro. O primeiro usuário (e escritório) é criado pelo subcomando
`bootstrap-user` do binário do backend, com senha lida de `BOOTSTRAP_USER_PASSWORD` (≥ 8 caracteres):

```bash
docker compose run --rm -e BOOTSTRAP_USER_PASSWORD='...' backend \
  bootstrap-user --escritorio-slug demo --escritorio-nome "Escritório Demo" \
  --email admin@exemplo.com --cpf 00011122233 --papel ADMIN
```

**Motivo:** superfície de ataque mínima na fase; o cadastro autoatendido de escritórios/clientes
fica para uma fase com requisitos próprios.

### A6 — Persistência em `sessoes` sem `ALTER`

A sessão é materializada na tabela `sessoes` (migration `20261021130000_create_sessoes`), com
`audiencia`, `familia_id`, `refresh_hash`, `expira_em`, `revogada_em` e índices por
`(escritorio_id, usuario_id)` e `familia_id`. Segue a política pré-MVP: nova tabela via
`CREATE TABLE`, sem `ALTER` em migrations existentes ([ADR-0006](0006-migracao-automatica-banco-dados.md)).

### A7 — Sessão no frontend

Cada app guarda apenas o slug do escritório em `localStorage` (para pré-preencher o login) e o access
token em memória. No boot, um `provideAppInitializer` chama `refresh`; o interceptor anexa o Bearer e,
em `401`, renova uma única vez com requisições concorrentes deduplicadas (single-flight) e repete a
requisição. Falha de renovação redireciona para `/admin/login` ou `/app/login` com `returnUrl` — apenas
caminho interno, validado contra open redirect. Guards separam visitante de autenticado em cada app.

## Consequências

- Painel e portal têm ciclos de sessão independentes; um token vazado no painel não autentica no
  portal (e vice-versa), inclusive no backend.
- O logout revoga a sessão no servidor (não é só limpeza local) e limpa o cookie.
- A renovação silenciosa exige que o proxy encaminhe cookies e cabeçalhos sem alteração; o path do
  cookie é restrito à subárvore de auth de cada audiência.
- O modo Trial alternado continua sendo lido do banco no backend; o `role`/`aud` do token não concede
  privilégios administrativos implícitos.
- Métricas de lockout e rate limit são auditoria de segurança; ajustes exigem mudar `AUTH_*` e
  reiniciar o backend.

## Referências

- `SPEC.md` §2.2 (frontends), §9.10 (`sessoes`), §10.1 (endpoints de autenticação).
- `INSTRUCTIONS.md` §3 (variáveis `AUTH_*` e bootstrap), §5 (migrations), §9 (testes).
- `docs/DECISOES-PENDENTES.md` (divergência de numeração e decisões G1–G6).
- `backend/tests/http_auth.rs`, `backend/src/presentation/auth/`, `backend/src/application/auth/`.
