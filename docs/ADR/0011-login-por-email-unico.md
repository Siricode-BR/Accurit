# ADR-0011 — Login do escritório por e-mail único

- **Status:** Aceito
- **Data:** 2026-11-07
- **Decisores:** Dono do produto, Arquitetura Accurit
- **Escopo:** Identificação do tenant no login da plataforma do escritório (`/api/v1/auth/client/login`)
- **Suplanta parcialmente:** [ADR-0009](0009-autenticacao-por-aplicacao.md) (login com slug) e [ADR-0010](0010-revisao-atores-papeis-e-provisionamento.md) (unicidade de e-mail por escritório)

## Contexto

O login da plataforma do escritório exigia `{ escritorio (slug), email, senha }` porque a
unicidade era `UNIQUE (escritorio_id, email)`: o mesmo e-mail podia existir em tenants diferentes.
Isso obrigava o usuário a lembrar o slug e expunha um dado interno na tela de login.

## Decisão

1. O e-mail é **único em toda a plataforma**, sem diferenciar maiúsculas de minúsculas:
   `CREATE UNIQUE INDEX uq_usuarios_email ON usuarios (lower(email))`.
2. O login recebe apenas `{ email, senha }`; o escritório vem de `usuarios.escritorio_id`.
   O campo legado `escritorio`, se enviado, é ignorado.
3. O provisionamento (`POST /api/v1/admin/escritorios` e `bootstrap`) verifica o e-mail globalmente
   **antes** de criar o escritório, para não deixar tenant órfão; conflito responde `409`.
4. O rate limit de login passa a usar a chave `IP|rota|e-mail`.
5. E-mail inexistente responde `401 AUTH_INVALID_CREDENTIALS` com o mesmo custo de hash e **sem**
   auditoria (não há tenant para atribuir o evento; evita inflar a tabela com tentativas anônimas).
6. Em desenvolvimento a mudança foi aplicada direto em `create_usuarios` (sem `ALTER`), conforme a
   política de migrations; bancos de dev existentes precisam ser recriados.

## Consequências

- Uma pessoa com acesso a dois escritórios precisa de e-mails distintos (ex.: `+alias`).
- Não é possível reservar o e-mail de um usuário de outro escritório: o conflito de provisionamento
  revela que o e-mail já existe na plataforma (aceito: a rota é restrita ao backoffice).
