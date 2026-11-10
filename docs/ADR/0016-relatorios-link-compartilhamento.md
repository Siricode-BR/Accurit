# ADR-0016 — Relatórios e link seguro de compartilhamento (Fase F)

Status: aceito

## Contexto

A Fase F introduz a geração de relatórios em PDF e XLSX (situação fiscal, histórico de
consultas e pendências por carteira) e a possibilidade de compartilhar esses relatórios com
o cliente final por meio de um link seguro e expirável, sem exigir autenticação no portal
do escritório (o cliente PF/PJ atendido é um dado do escritório, nunca um usuário, conforme
SPEC §1.3).

## Decisão

### 1. Armazenamento de relatórios

É criada a tabela `relatorios` com uma coluna `conteudo BYTEA` que armazena o conteúdo do
arquivo gerado (PDF ou XLSX) diretamente no banco de dados. Cada relatório está vinculado a
um escritório (`escritorio_id`), ao usuário que o gerou (`usuario_id`) e opcionalmente a um
cliente específico (`cliente_id`). O status (`gerando`, `concluido`, `falhou`) permite
diferenciar relatórios ainda em processamento dos já finalizados.

A geração é síncrona na requisição: o endpoint `POST /api/v1/relatorios` cria o registro, gera o
conteúdo e atualiza o registro com o status e o conteúdo antes de responder. Isso evita a
complexidade de um job runner assíncrono apenas para relatórios, dado o volume esperado
baixo nesta fase.

### 2. Link seguro de compartilhamento

É criada a tabela `links_compartilhamento_relatorios` com um token aleatório
(`UUID::now_v7()` convertido para string), tempo de expiração (`expira_em`) e timestamp de
último acesso (`acessado_em`). O endpoint `GET /api/v1/relatorios/compartilhado/{token}` é público
(sem autenticação): valida o token, verifica a expiração, registra o acesso e devolve o
conteúdo do relatório para download direto.

O token é gerado com `Uuid::now_v7()`, que fornece aleatoriedade suficiente para evitar
adivinhação. A expiração é configurável no momento da criação (padrão: 7 dias). Após a
expiração, o acesso é negado com erro 404 (não vazar existência de links expirados).

### 3. Isolamento de tenant

Todas as operações de relatório são filtradas por `escritorio_id`, derivado exclusivamente
do `RequestContext` autenticado (JWT + middleware de tenancy). O link público não exige
autenticação, mas o token só concede acesso ao relatório específico vinculado, sem expor
outros relatórios do escritório.

## Consequências

- Positivo: escritórios podem exportar relatórios em PDF/XLSX e compartilhá-los com clientes
  finais sem exigir login desses clientes no portal.
- Negativo: o conteúdo dos relatórios é armazenado no banco (BYTEA), o que pode aumentar o
  tamanho do banco ao longo do tempo; uma política de retenção/limpeza pode ser necessária
  em fases futuras.
- Negativo: o link público não exige autenticação, portanto quem possuir o token poderá
  baixar o relatório; a expiração é a principal medida de controle.

## Referências

- [SPEC §8.9](SPEC.md) — Relatórios e link seguro de compartilhamento (Fase F).
- [SPEC §9.20–9.21](SPEC.md) — Tabelas `relatorios` e `links_compartilhamento_relatorios`.
- [SPEC §1.3](SPEC.md) — Fora de escopo: portal autenticado para o cliente PF/PJ.