# ADR-0019 — CNPJ como identificador do escritório (substitui o slug)

Status: aceito

## Contexto

O escritório de contabilidade (tenant) era identificado por um `slug` textual, usado em
provisionamento, login e referências internas. O `slug` não é um identificador fiscal,
exigia regras artificiais de unicidade e não conversava com o certificado A1 nem com o
modelo SERPRO (Contratante/Procurador identificados por CNPJ).

## Decisão

1. O CNPJ passa a ser o identificador público e fiscal do escritório. A coluna `cnpj`
   (`VARCHAR(14)`, somente dígitos, `UNIQUE`) substitui `slug` em `escritorios`.
2. O CNPJ é validado no domínio pelo value object `domain::tenancy::Cnpj` (14 dígitos,
   dígitos verificadores, máscara aceita e normalizada). O mesmo value object é usado
   por `bootstrap-tenant`, provisionamento de tenant e qualquer entrada administrativa.
3. O login continua **exclusivamente por e-mail**. O CNPJ não é credencial e não é
   solicitado na tela de login; o e-mail identifica o usuário e, por consequência, o
   escritório.
4. Todo o isolamento multi-tenant continua por `escritorio_id`; o CNPJ nunca vem do
   corpo da requisição em rotas autenticadas.
5. O formulário administrativo aceita a máscara `00.000.000/0000-00`, mas o backend
   persiste apenas os 14 dígitos. CNPJ inválido retorna `422 VALIDACAO`; CNPJ já
   cadastrado retorna `409 CONFLITO`.
6. Ao cadastrar um certificado A1, o CNPJ do titular do certificado deve corresponder ao
   CNPJ do escritório; divergência é rejeitada com mensagem amigável (nunca expondo o
   conteúdo do certificado).

## Consequências

- O banco de desenvolvimento precisa ser recriado (migration editada in place, sem
  `ALTER`): `20261021120000_create_escritorios`.
- `domain/tenancy/slug.rs` foi removido e o duplicado `domain/auth/cnpj.rs` foi
  eliminado; existe um único value object `Cnpj`.
- `bootstrap-tenant` passou a exigir `--escritorio-cnpj` e `--escritorio-nome`.
- A listagem administrativa expõe `id`, `cnpj`, `nome`, `modoTrialSerpro`, `versao` e
  `atualizadoEm`; a busca aceita nome ou CNPJ.

## Referências

- SPEC §2 (atores/tenancy) e §10.3 (contratos administrativos).
- ADR-0010 (atores e provisionamento) e ADR-0011 (login por e-mail único).
