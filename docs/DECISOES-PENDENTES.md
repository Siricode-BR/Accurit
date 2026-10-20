# Decisões Pendentes e Itens Não Confirmados

- **Atualizado em:** 2026-10-20
- **Escopo:** Fase 0 — reconciliação documental
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

- `backend/Diesel.toml` renomeado para `backend/diesel.toml`.
- `.gitattributes` com `* text=auto eol=lf` e normalização de CRLF.
- `docs/Serpro/Preços` confirmado no disco com “Preços” UTF-8; nome no git a confirmar após `git add`.
- `CERTIFICATE_MASTER_KEY=change-me-32-bytes` deve falhar no startup.
