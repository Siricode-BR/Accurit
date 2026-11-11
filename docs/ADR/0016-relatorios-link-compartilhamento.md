# ADR-0016 — Relatórios e link seguro de compartilhamento (Fase F)

Status: aceito

> Este ADR unifica a decisão antes registrada em duplicidade como ADR-0016 e ADR-0017.
> A ADR-0017 foi consolidada aqui e removida; não há decisão independente nela.

## Contexto

A Fase F introduz a geração de relatórios em PDF e XLSX (situação fiscal, histórico de
consultas e pendências por carteira) e a possibilidade de compartilhar esses relatórios com
o cliente final por meio de um link seguro e expirável, sem exigir autenticação no portal
do escritório (o cliente PF/PJ atendido é um dado do escritório, nunca um usuário, conforme
SPEC §1.3).

O link público é o ponto mais sensível: um relatório contém dados fiscais do contribuinte e
não pode ser protegido apenas pela dificuldade de adivinhar a URL.

## Decisão

### 1. Armazenamento de relatórios

É criada a tabela `relatorios` com a coluna `conteudo BYTEA`, que armazena o arquivo gerado
(PDF ou XLSX) no banco. Cada relatório é vinculado a um escritório (`escritorio_id`), ao
usuário que o gerou (`usuario_id`) e opcionalmente a um cliente (`cliente_id`). O status
(`gerando`, `concluido`, `falhou`) diferencia relatórios em processamento dos finalizados.
Cada relatório tem `expira_em` (retenção do próprio artefato).

A geração é síncrona na requisição: `POST /api/v1/relatorios` cria o registro, gera o
conteúdo e atualiza o registro com status e conteúdo antes de responder. Isso evita a
complexidade de um job runner assíncrono apenas para relatórios, dado o volume esperado
baixo nesta fase.

### 2. Link seguro de compartilhamento

É criada a tabela `links_compartilhamento_relatorios` com:

- `token_hash VARCHAR(64) UNIQUE` — **hash SHA-256 de um token opaco** (dois UUID v7
  concatenados, 64 caracteres hexadecimais de alta entropia). O token em claro é devolvido
  **uma única vez**, na criação do link, e nunca é persistido nem logado;
- `expira_em` — expiração configurável na criação (padrão: 30 dias);
- `max_acessos` — limite de acessos por link (`1..=1000`, padrão 50);
- `acessos` / `acessado_em` — contador e instante do último acesso;
- `revogado_em` — revogação explícita pelo escritório.

Regras de validação (fail-closed): token inexistente, expirado, revogado ou com acessos
esgotados respondem `404` sem revelar qual condição falhou. O acesso é **somente leitura**:
o endpoint público devolve exclusivamente o conteúdo do relatório vinculado ao token.

A rota pública `GET /api/v1/relatorios/compartilhado/{token}` não exige autenticação, mas
passa pelo rate limit global de requisições e por headers de segurança; tentativas repetidas
de tokens inválidos não recebem resposta diferenciada. O incremento de `acessos` e
`acessado_em` registra a auditoria de acesso de cada link.

### 3. Revogação

`DELETE /api/v1/relatorios/{id}/links` revoga todos os links ativos de um relatório
(`revogado_em = NOW()`), invalidando imediatamente qualquer token em circulação. A revogação
é auditável pelo próprio registro (quem revogou via `usuario_id` do relatório e quando).

### 4. Isolamento de tenant

Todas as operações autenticadas de relatório e de criação/revogação de link são filtradas
por `escritorio_id`, derivado exclusivamente do `RequestContext` (JWT + middleware de
tenancy). O link público não exige autenticação, mas o token só concede acesso ao relatório
específico vinculado, sem expor outros relatórios do escritório.

## Consequências

- Positivo: escritórios exportam relatórios em PDF/XLSX e os compartilham com clientes finais
  sem exigir login desses clientes no portal.
- Positivo: vazamento do banco não expõe tokens utilizáveis (apenas hashes); expiração,
  limite de acessos e revogação são controles independentes.
- Negativo: o conteúdo dos relatórios é armazenado no banco (BYTEA), o que pode aumentar o
  tamanho do banco ao longo do tempo; a retenção (`expira_em`) precisa ser aplicada por rotina
  de limpeza em fase futura.
- Negativo: qualquer portador do token, dentro da janela de validade e do limite de acessos,
  pode baixar o relatório — o link deve ser tratado como credencial.

## Referências

- [SPEC §8.9](../SPEC.md) — Relatórios e link seguro de compartilhamento (Fase F).
- [SPEC §9.20–9.21](../SPEC.md) — Tabelas `relatorios` e `links_compartilhamento_relatorios`.
- [SPEC §1.3](../SPEC.md) — Fora de escopo: portal autenticado para o cliente PF/PJ.
