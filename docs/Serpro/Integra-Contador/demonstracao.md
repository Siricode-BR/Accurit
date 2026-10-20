---
origem_pdf: Demonstração - Integra Contador.pdf
origem_url: https://apicenter.estaleiro.serpro.gov.br/documentacao/api-integra-conta... (truncado no PDF)
capturado_em: 2026-10-17
conversao: manual-assistida
aviso: Cópia de trabalho. Em divergência, vale o PDF.
---

# Demonstração

**API Integra Contador** — `api-integra-contador-trial-sp.yaml`

O Integra Contador é uma plataforma de serviços, que fornece um conjunto de API's, para
escritórios de contabilidade e demais empresas do ramo contábil, otimizando a prestação de
serviços aos contribuintes.

## Chave do ambiente Trial (Bearer)

`<TOKEN_TRIAL_PUBLICO>` <!-- token de demonstração redigido; o valor está no PDF original -->

## Servers

`https://gateway.apiserpro.serpro.gov.br/integra-contador-trial/v1` — Trial

## Authorize

## Endpoints (verbos)

| Verbo | Método | Path | Descrição |
|---|---|---|---|
| Apoiar | POST | `/Apoiar` | Serviço do tipo Apoiar. Acionado por serviços auxiliares de suporte à solução. |
| Consultar | POST | `/Consultar` | Serviço do tipo Consultar. Executa um envio de pedido de dados do tipo consulta. |
| Declarar | POST | `/Declarar` | Serviço do tipo Declarar. Executa um envio de pedido de dados do tipo entrega ou transmissão de declaração. |
| Emitir | POST | `/Emitir` | Serviço do tipo Emitir. Executa um envio de pedido de dados do tipo emissão ou geração de documento de arrecadação. |
| Monitorar | POST | `/Monitorar` | Serviço do tipo Monitorar. Acionado por serviços auxiliares de monitoração de eventos. |

## Schemas

### `DadosEntrada`

| Campo | Tipo | Descrição |
|---|---|---|
| `contratante` | `Identificacao` | Identificação do contratante. |
| `autorPedidoDados` | `Identificacao` | Identificação do autor do pedido de dados. |
| `contribuinte` | `Identificacao` | Identificação do contribuinte. |
| `pedidoDados` | `PedidoDados` | Dados do pedido. |

> No Trial, o `tipo` de `contratante`/`autorPedidoDados`/`contribuinte` aparece com
> `Enum: Array [2]` (somente 2 = CNPJ) <!-- conferir no PDF, p.2 -->, enquanto `Identificacao.tipo`
> (usado por `DadosEntrada`) mantém `Enum: [1, 2]`.

### `Identificacao`

| Campo | Tipo | Descrição |
|---|---|---|
| `numero` | `string` | Número (CPF/CNPJ). |
| `tipo` | `TipoNi` (integer, int32) | Enum: `[1, 2]`. |

### `PedidoDados`

| Campo | Tipo | Descrição |
|---|---|---|
| `idSistema` | `string` | Código do sistema (Catálogo de Serviços). |
| `idServico` | `string` | Código do serviço (Catálogo de Serviços). |
| `versaoSistema` | `string` | Versão do sistema. |
| `dados` | `string` | Conteúdo do pedido. |

### `ProblemDetails`

| Campo | Tipo | Descrição |
|---|---|---|
| `type` | `string` | URI do tipo do problema. |
| `title` | `string` | Título. |
| `status` | `integer` (int32) | Código HTTP. |
| `detail` | `string` | Detalhe. |
| `instance` | `string` | Instância. |
| `< * >` | `object` | Campos de extensão (abertos). |

### `TipoNi`

`integer` (int32) — Enum: `[1, 2]`.

---

Documento datado de: **13 de fevereiro de 2026**
