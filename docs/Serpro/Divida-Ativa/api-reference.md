---
origem_pdf: consulta-divida-ativa - API CONSULTA DÍVIDA ATIVA.pdf
origem_url: https://apicenter.estaleiro.serpro.gov.br/documentacao/consulta-divida-at... (truncado no PDF)
capturado_em: 2026-10-20
conversao: manual-assistida
aviso: Cópia de trabalho. Em divergência, vale o PDF.
---

# API Reference — Chamada ao Serviço

**Consulta Divida Ativa** — versão `v1` — especificação **OAS3** (OpenAPI 3).

Documento capturado da página "Chamada ao Serviço" (Swagger UI). Os endpoints exigem
autorização (indicados com cadeado na interface).

## Servers

| Ambiente | URL |
|---|---|
| Produção | `https://gateway.apiserpro.serpro.gov.br/consulta-divida-ativa/api` |

> A URL trial correspondente é
> `https://gateway.apiserpro.serpro.gov.br/consulta-divida-ativa-trial/api` (ver
> [quick start](quick-start-consulta-divida-ativa.md)).

## Authorize

Autenticação via botão "Authorize" da interface (OAuth2 / Bearer Token). Veja o fluxo completo
de obtenção do token em [quick-start-consulta-divida-ativa.md](quick-start-consulta-divida-ativa.md).

## Endpoints (verbos)

### Tag: Divida Ativa DF

| Verbo | Path | Descrição |
|---|---|---|
| GET | `/v1/inscricao/{numeroInscricao}` | Consulta dados de inscrição |
| GET | `/v1/devedor/{numeroInscricaoDevedor}` | Consulta dados do devedor, seja pessoa física ou jurídica |

### Tag: Status

| Verbo | Path | Descrição |
|---|---|---|
| GET | `/status` | Retorna o status da API CPF |

> IMPORTANTE: O gateway interpreta as letras maiúsculas e minúsculas de forma diferente.
> Certifique-se de enviar os endpoints conforme esta referência, sem alterar para maiúsculo ou
> minúsculo qualquer parte do endpoint.

## Parâmetros de path

| Path | Parâmetro | Descrição |
|---|---|---|
| `/v1/inscricao/{numeroInscricao}` | `numeroInscricao` | Número da inscrição na Dívida Ativa (formato com dígitos verificadores, ex.: `9061700096419`). |
| `/v1/devedor/{numeroInscricaoDevedor}` | `numeroInscricaoDevedor` | CPF ou CNPJ do devedor (ex.: `09781911768`). |

> Os exemplos de valores acima vêm das chamadas de demonstração do quick start. Os detalhes de
> schemas de resposta completos não constam nesta captura (2 páginas; página 2 em branco) —
> consulte o swagger publicado em `origem_url` para o contrato integral.
