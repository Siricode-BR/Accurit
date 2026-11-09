# ADR-0015 — Gestão de carteira: função granular e restrição por carteira de clientes (Fase E)

Status: aceito

## Contexto

A FASE E introduz a gestão de carteira de clientes no portal do escritório: vários
sócios/contatos por cliente (com representante principal), filiais/unidades, tags,
responsável interno e observações, além de importação em massa (CSV/XLSX) e de
permissões granulares. Duas decisões de arquitetura precisam ser registradas:

1. Como se define a função efetiva de um usuário no escritório (`GESTOR`,
   `ANALISTA_FISCAL`, `OPERADOR_LEITURA`).
2. Como se restringe o escopo de clientes visíveis e editáveis por usuário
   (carteira explícita vs. carteira completa do escritório).

## Decisão

### 1. Função efetiva do usuário

É criada a tabela `funcoes_usuarios(usuario_id PK, escritorio_id, funcao CHECK
('GESTOR','ANALISTA_FISCAL','OPERADOR_LEITURA'), ...)`. A linha é opcional:

- Se houver linha para o usuário no escritório, essa é a função efetiva.
- Caso contrário, a função deriva do `papel` (`usuarios.papel`):
  - `CLIENTE` → `GESTOR`;
  - `OPERADOR` → `ANALISTA_FISCAL`;
  - outros papéis (ex.: `Admin` do backoffice) → `ANALISTA_FISCAL` (menor privilégio).

A coluna `usuarios.papel` permanece inalterada. A função é lida do banco a cada
requisição (não vem do JWT), de modo que alterações de função/carteira refletem
imediatamente sem reemitir tokens.

### 2. Restrição de carteira por usuário

É criada a tabela `carteira_usuarios(usuario_id, cliente_id UNIQUE por usuário, ...)`:

- Usuário **sem** vínculos em `carteira_usuarios`: enxerga toda a carteira do
  escritório (`clientes_permitidos = NULL`).
- Usuário **com** vínculos: enxerga apenas os clientes vinculados
  (`clientes_permitidos = [ids]`).

Qualquer operação (leitura ou escrita) sobre um cliente fora da carteira responde
`404` — não se vaza a existência do cliente (mesmo padrão de IDOR já adotado
para consultas/situação fiscal). Somente funções com permissão de escrita
(`GESTOR`/`ANALISTA_FISCAL`) criam, editam ou removem itens de carteira
(sócios/contatos, filiais, tags, responsável e observações).

A gestão de função e carteira de outros usuários
(`PUT /usuarios/{id}/acesso`, `PUT /usuarios/{id}/carteira`) exige papel
`CLIENTE` e função `GESTOR`.

### 3. Escrita nas tabelas de carteira

Todas as 7 migrations da FASE E são apenas `CREATE TABLE` (sem `ALTER` em
tabelas existentes, conforme a regra de desenvolvimento do projeto). Cada tabela
tem `escritorio_id` e todos os filtros de repositório incluem
`escritorio_id = ?`, garantindo isolamento multi-tenant.

## Consequências

- Positivo: permissões granulares compatíveis com o papel do titular e com a
  carteira explícita; alterações de acesso são efetivas imediatamente; não há
  vazamento de existência de clientes fora da carteira.
- Negativo: a função deve ser consultada por requisição no banco (custo de uma
  leitura extra por requisição de escrita); endpoints de leitura de cliente
  específico precisam checar `pode_ver(cliente_id)` além do isolamento de
  `escritorio_id`.

## Referências

- [SPEC §8.8](SPEC.md) — Gestão de Carteira (Fase E).
- [SPEC §9.14–9.19](SPEC.md) — Tabelas `socio_contatos`, `cliente_filiais`,
  `cliente_tags`, `cliente_tags_vinculos`, `carteira_clientes`,
  `funcoes_usuarios`, `carteira_usuarios`.
- [SPEC §10.1](SPEC.md) — Contrato da API (Fase E).
