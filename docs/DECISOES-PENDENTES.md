# Decisões Pendentes e Itens Não Confirmados

- **Atualizado em:** 2026-10-21
- **Escopo:** Fase 0 (reconciliação documental), Fase 1 (fundação técnica) e Fase 2A (autenticação por aplicação)
- **Regra:** nenhum item abaixo pode ser tratado como contrato oficial até confirmação. O código
  deve isolar o comportamento em porta/flag e falhar de forma explícita quando a configuração não
  existir.

## Resumo

| ID | Tema | Status | Impacto | Ação |
|---|---|---|---|---|
| D1 | Modelo de contratação SERPRO | **Decidido** (modelo A padrão; porta para B) | Alto | Ver [ADR-0007](ADR/0007-modelo-contratacao-serpro.md) |
| D2 | Fallback “Código de Contribuinte” | **Não confirmado** | Médio | Desabilitado por padrão; validar em trial |
| D3 | Header `x-cpf-usuario` | **Não confirmado** | Médio | Manter flag; teste live opt-in |
| D4 | Allowlist de serviços (parcelamentos/SICALC) | **Decidido** | Alto | Ver [ADR-0002](ADR/0002-escopo-consultivo-divida-ativa.md) |
| D5 | URL de produção da Dívida Ativa | **Não confirmado** | Alto | Bloquear com `SERPRO_PROD_NOT_CONFIGURED` |
| P1 | Conversão dos docs oficiais para Markdown | Pendente | Baixo | Conferir com PDF e remover aviso |
| P2 | Formato de “sem pendências” (200 vazio vs 404) | **Não confirmado** | Médio | Teste live opt-in |
| P3 | Formato multi-inscrição de `/devedor` | **Não confirmado** | Médio | DTO tolerante; teste live |
| P4 | `versaoSistema` dos demais serviços | **Não confirmado** | Alto | Habilitar serviço a serviço |
| P5 | Produção do Integra Contador (SAPI) | Parcialmente confirmado | Alto | Validar mTLS/e-CNPJ em homologação |
| P6 | Ativação do modelo B | Não decidido | Alto | Exige necessidade + revisão jurídica |

## Decisões de implementação da Fase 1

Estas decisões estão **fechadas** para a fundação e registradas em
[ADR-0008](ADR/0008-decisoes-implementacao-fase-1.md). Não são pendências externas; mudanças exigem
nova decisão arquitetural.

| ID | Decisão | Efeito |
|---|---|---|
| F1 | Diesel síncrono + pool `r2d2`, acesso via traits e `spawn_blocking` | Sem `diesel-async` na Fase 1; porta de repositório preserva evolução |
| F2 | `POSTGRES_USER` superusuário só para administração/testes; `POSTGRES_APP_USER` não-superuser com DDL em `public` | `DATABASE_URL` nunca usa superusuário; split migrator/runtime adiado |
| F3 | Linkagem dinâmica com `libpq` no Alpine (`-crt-static` desabilitado) | Runtime instala `libpq`; build estático rejeitado |
| F4 | Crate `accurit_backend` + binário `backend`, edition 2024 | `main.rs` fino; testes usam a biblioteca |
| F5 | Rate limit com chave abstrata; Fase 1 usa IP de `X-Real-IP` | Fase 2 troca para `escritorio_id` sem reescrever o middleware |
| F6 | Testes de integração no serviço `backend-test` (profile `tools`) | Banco descartável por teste; sem dependência de toolchain no host |

## Decisões de implementação da Fase 2A

Estas decisões estão **fechadas** para a autenticação por aplicação e registradas em
[ADR-0009](ADR/0009-autenticacao-por-aplicacao.md). Não são pendências externas; mudanças exigem nova
decisão arquitetural.

| ID | Decisão | Efeito |
|---|---|---|
| G1 | Rotas de auth por audiência (`/api/v1/auth/{admin,client}/*`) com claim `aud` e papel fixo por audiência | Token do painel não vale no portal; `CLIENTE` não acessa o painel e vice-versa |
| G2 | Access JWT HS256 de 900 s + refresh opaco 256 bits com hash SHA-256, rotação e família | Reuso de refresh revoga a família e audita `AUTH_REFRESH_REUSO`; logout revoga no servidor |
| G3 | Refresh em cookie `HttpOnly`/`SameSite=Strict` com `Path` por audiência; CSRF por `X-Requested-With: accurit` + `Content-Type` JSON | `AUTH_COOKIE_SECURE=true` por padrão; `false` só em `APP_ENV=development` |
| G4 | Argon2id, `401 AUTH_INVALID_CREDENTIALS` uniforme, lockout 5/900 s e rate limit 10 r/m (burst 5) no login | Sem enumeração de usuário; motivos auditados (`USUARIO_INEXISTENTE`, `USUARIO_INATIVO`, `USUARIO_BLOQUEADO`) |
| G5 | Primeiro usuário via `bootstrap-user` (subcomando do binário `backend`, senha em `BOOTSTRAP_USER_PASSWORD`) | Sem endpoint público de cadastro na fase |
| G6 | Numeração: autenticação por aplicação = ADR-0009 | O "ADR-0008" previsto originalmente foi consumido pelas decisões de implementação da Fase 1 |

## D1 — Modelo de contratação SERPRO

**Decisão:** adotar o modelo A (cada escritório contrata com o próprio e-CNPJ) como padrão, com a
porta `SerproCredentialProvider` para permitir o modelo B sem reescrever o adapter.

**Pendência:** nenhuma para o modelo A. O modelo B permanece desabilitado até necessidade concreta e
aprovação explícita.

**Referência:** [ADR-0007](ADR/0007-modelo-contratacao-serpro.md).

## D2 — Fallback “Código de Contribuinte”

**Status:** NÃO CONFIRMADO.

A documentação oficial consultada não descreve “Código de Contribuinte” como chave alternativa. O
parâmetro do path é `numeroInscricaoDevedor`; a amostra oficial usa CPF/CNPJ. Não há garantia de que
o endpoint aceite outro identificador.

**Decisão provisória:**

- `DIVIDA_ATIVA_FALLBACK_CODIGO_CONTRIBUINTE=false`.
- Porta `ResolvedorChaveAlternativa`; sem implementação configurada, não há fallback.
- Quando habilitado, no máximo uma tentativa, somente após resposta bem-sucedida e vazia.
- Registrar `fonte_consulta`.

**Como confirmar:** teste live opt-in (`RUN_LIVE_TRIAL=1`) com uma chave alternativa real fornecida
pela SERPRO; verificar se o path aceita o valor e qual o comportamento de erro.

## D3 — Header `x-cpf-usuario`

**Status:** NÃO CONFIRMADO.

O header não consta na documentação oficial consultada da Dívida Ativa nem do Integra Contador. Não
há confirmação de obrigatoriedade nem de HTTP 401 na ausência.

**Decisão provisória:**

- `SERPRO_OPERATOR_HEADER_ENABLED=true` (compatibilidade), envio centralizado em um único ponto.
- CPF obtido de `usuarios.cpf`, nunca do JWT ou do cliente.
- Teste live documenta exigência, formato e código de erro; se não for exigido, avaliar desabilitar
  o padrão.

## D4 — Allowlist de serviços

**Status:** DECIDIDO.

Bloquear todos os sistemas de parcelamento, inclusive `Consultar`, e `SICALC` `Emitir` (51, 53).
`PGMEI / DIVIDAATIVA24` fica `AguardandoDecisao`. A tabela completa está na
[ADR-0002](ADR/0002-escopo-consultivo-divida-ativa.md).

## D5 — URL de produção da Dívida Ativa

**Status:** NÃO CONFIRMADO.

A documentação oficial consultada cobre o Trial. A URL de produção não foi localizada.

**Decisão provisória:** `SERPRO_DIVIDA_ATIVA_PROD_URL` vazio e chamada bloqueada com
`SERPRO_PROD_NOT_CONFIGURED`. Não inferir a URL a partir do Integra Contador.

**Como confirmar:** contrato/credencial de produção da SERPRO para o produto Dívida Ativa.

## P1 — Conversão dos docs oficiais

`docs/Serpro/Integra-Contador/quick-start.md` e
`docs/Serpro/Integra-Contador/cenarios-procuracoes.md` foram convertidos e estão marcados com
`conversao: pendente-conferir-com-pdf`. Conferir o conteúdo contra o PDF oficial, ajustar e remover o
aviso.

## P2 — “Sem pendências”: 200 vazio ou 404

**Status:** NÃO CONFIRMADO.

Não há amostra oficial de devedor sem inscrições. O DTO e o caso de uso devem distinguir:

- `200` com corpo vazio/lista vazia;
- `404` com mensagem de não encontrado;
- erro de integração.

**Como confirmar:** teste live com CPF/CNPJ sem pendências conhecido. Até lá, não converter `404` em
“sem pendências” automaticamente sem validação.

## P3 — Formato multi-inscrição de `/devedor`

**Status:** NÃO CONFIRMADO.

A amostra oficial é um objeto plano. Não há confirmação de que o endpoint retorne lista quando o
devedor tem múltiplas inscrições. O DTO usa `#[serde(untagged)]` para aceitar objeto ou lista, mas o
mapeamento de domínio precisa ser validado com uma amostra real multi-inscrição.

## P4 — `versaoSistema` dos demais serviços

**Status:** NÃO CONFIRMADO.

Confirma-se `OBTERPROCURACAO41 = "1"` e `ENVIOXMLASSINADO81 = "1.0"`. Os demais serviços devem ser
habilitados um a um, com `versaoSistema` extraída do catálogo/PDF e coberta por fixture. Não usar um
valor global.

## P5 — Produção do Integra Contador (SAPI)

**Status:** PARCIALMENTE CONFIRMADO.

O fluxo SAPI (endpoint, `Role-Type: TERCEIROS`, `Content-Type`, mTLS e-CNPJ, `access_token` +
`jwt_token`) está documentado. Ainda falta validar em homologação:

- formato exato da cadeia e do certificado aceito;
- necessidade de `x-jwt-assertion` em algum fluxo;
- duração/renovação do `jwt_token`;
- comportamento de `401` e `403` por produto.

## P6 — Ativação do modelo B

**Status:** NÃO DECIDIDO.

O modelo B (contratante central, autor diferente) exige Termo de Autorização, procuração e
`autenticar_procurador_token`. Não será habilitado sem necessidade concreta, revisão jurídica e
aprovação de arquitetura.

## Itens de infraestrutura já resolvidos

- `backend/Diesel.toml` renomeado para `backend/diesel.toml` no workspace; versionado com o
  scaffold do backend na Fase 1.
- `.gitattributes` com `* text=auto eol=lf` e normalização de CRLF versionados.
- `docs/Serpro/Preços` confirmado no git com “Preços” UTF-8.
- `CERTIFICATE_MASTER_KEY=change-me-32-bytes` deve falhar no startup.
- Três migrations da Fase 1 criadas e validadas (`escritorios`, `usuarios`, `auditoria_eventos`).
- `POSTGRES_APP_USER` criado pelo init do Postgres como role não-superuser.
- `schema.rs` gerado por `schema-tool` e formatado com `rustfmt`, sem `diesel_cli` no runtime.
- `backend-test` executa a suíte no container com banco descartável por teste.
- Smoke `.ps1`/`.sh` cobre as rotas públicas, health, redirect de login, envelope 404 e request-id.
- Quatro migrations validadas (`escritorios`, `usuarios`, `auditoria_eventos`, `sessoes`) com a
  política "sem `ALTER`" (`backend/tests/migrations_policy.rs` exige ≥ 8 migrations verificadas).
- Suíte `backend/tests/http_auth.rs` cobre login por audiência, cookie seguro, CSRF, rotação e reuso
  de refresh, logout, lockout e bloqueio cruzado entre painel e portal.
