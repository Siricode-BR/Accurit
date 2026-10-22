# ADR-0010 — Revisão do modelo de atores, papéis e provisionamento

- **Status:** Aceito
- **Data:** 2026-10-21
- **Decisores:** Dono do produto, Arquitetura Accurit
- **Escopo:** Semântica dos papéis, ligação papel↔audiência da Fase 2A, provisionamento de tenants e CPF de usuários
- **Suplanta parcialmente:** [ADR-0009](0009-autenticacao-por-aplicacao.md) (decisão A1 e A5 quanto a quem autentica em cada audiência)
- **Relaciona-se com:** [ADR-0007](0007-modelo-contratacao-serpro.md), [ADR-0008](0008-decisoes-implementacao-fase-1.md)

## Contexto

A Fase 2A implementou autenticação por aplicação (ADR-0009) modelando o papel `CLIENTE` como
"cliente PF/PJ final do escritório", com portal `/app` dedicado a ele e painel `/admin` para
`ADMIN`/`OPERADOR` do escritório. A revisão do modelo de negócio definiu que essa atribuição está
errada:

- O **CLIENTE** da Accurit é o **escritório de contabilidade que assinou o serviço** — ele é o
  titular do tenant e usa a plataforma do produto.
- O **OPERADOR** é um usuário **dentro da plataforma do CLIENTE** (membro da equipe do escritório).
- O **ADMIN** é o **gestor dos tenants** — operação interna do SaaS, administrando escritórios por
  dentro; ele não pertence a nenhum escritório.
- O cliente PF/PJ atendido pelo escritório **nunca é usuário da plataforma**: é dado em `clientes`.

A implementação atual está com a ligação papel↔audiência invertida
(`Audiencia::aceita`), e `usuarios.escritorio_id NOT NULL` conflita com um gestor global de tenants.

## Decisões

- **D1 — Papéis por audiência (revisão de A1/A5 do ADR-0009):** a audiência `admin` autentica
  **apenas** o gestor de tenants (papel `ADMIN`); a audiência `client` autentica
  `CLIENTE` e `OPERADOR`. A mecânica da Fase 2A fica intacta (JWT aud, rotação de refresh com
  família, cookie `HttpOnly`, CSRF, lockout, rate limit, testes).
- **D2 — ADMIN em tabela separada:** o gestor de tenants **não** fica em `usuarios` e não tem
  `escritorio_id`. Novas tabelas `administradores` (credenciais + lockout) e `sessoes_admin`
  (família de refresh, hash SHA-256, expiração e revogação), sem `ALTER` nas migrations existentes
  (política pré-MVP edita o `CREATE TABLE`).
- **D3 — Token de ADMIN sem tenant:** o JWT do gestor de tenants não carrega `escritorio_id`.
  Quando um administrador age sobre um tenant (provisionar, alternar Trial, etc.), o
  `escritorio_id` alvo vem da requisição — nunca do token, do corpo implícito ou de outra fonte
  autodeclarada. `RequestContext` passa a representar o `escritorio_id` como opcional.
- **D4 — Provisionamento por tela no backoffice:** a criação de escritórios e do usuário titular
  `CLIENTE` será feita por tela no `/admin` (backoffice), item da Fase 2B. O CLI mantém os
  subcomandos `bootstrap-admin` (primeiro gestor de tenants) e `bootstrap-tenant` (fallback que
  cria escritório + titular) — o `bootstrap-user` antigo é substituído.
- **D5 — Toggle Trial alterado pelo ADMIN:** o modo Trial SERPRO por escritório (Fase 3) é
  alternado no backoffice `/admin`, com confirmação e auditoria; o escritório somente visualiza o
  ambiente ativo. O texto do ROADMAP da Fase 3 continua correto — passa a usar a semântica nova.
- **D6 — CPF obrigatório para todos os papéis:** cada usuário (ADMIN, CLIENTE e OPERADOR) é uma
  pessoa física e tem CPF com 11 dígitos válidos; o escritório tem CNPJ
  (`escritorios.cnpj`). Fecha o "requisito de CPF para CLIENTE" da Fase 2 do ROADMAP.
- **D7 — Auditoria em nível SaaS:** `auditoria_eventos.escritorio_id` passa a aceitar `NULL`
  exclusivamente para eventos que não pertencem a nenhum tenant (sessões do gestor de tenants).
  Eventos administrativos sobre um tenant continudam gravados com o `escritorio_id` alvo.

## Consequências

- Backend: `Audiencia::aceita` revisada; login/refresh/logout/me de `/api/v1/auth/admin/*`
  consultam a tabela separada; `sessoes` passa a servir somente a plataforma do escritório
  (audiência `CLIENT`); `usuarios.papel` restringe-se a `('CLIENTE', 'OPERADOR')`.
- Frontend: `/admin` vira backoffice de gestão de tenants (Fase 2B: telas de provisionamento);
  `/app` é a plataforma do escritório; o formulário de login do `/admin` deixa de pedir o slug do
  escritório e o access token é anexado a qualquer requisição de `/api/v1/admin/*`.
- Testes: `http_auth.rs` cobre o bloqueio cruzado com o novo mapeamento (token de ADMIN não vale no
  portal, token de escritório não vale no backoffice); migrations sobem para seis (escritorios,
  usuarios, auditoria_eventos, sessoes, administradores, sessoes_admin).
- O aceite formal da Fase 2A fica condicionado ao ajuste descrito aqui.

## Alternativas consideradas

- **RLS/escritório "sede"** para o ADMIN em `usuarios`: mantém uma tabela só, mas inventa uma
  entidade de negócio falsa e polui queries de isolamento.
- **`escritorio_id` NULLABLE em `usuarios`:** obriga todos os consultas multi-tenant a tratar
  `NULL` e enfraquece as FKs por escritório.
- **Manter `CLIENTE` autenticando no portal do cliente final:** contradiz o produto — o cliente do
  escritório não usa a plataforma na Fase 2.

## Referências

- `SPEC.md` §1.4 (atores), §2.2, §9 (modelo de dados), §10.1 (endpoints).
- `INSTRUCTIONS.md` §3 (bootstrap), §5 (migrations).
- `docs/DECISOES-PENDENTES.md` (decisões A-ATR-1 a A-ATR-5, G1 revisado).
- Artifact de spec do epic: `atores-e-papeis`.
