---
origem_url: https://apicenter.estaleiro.serpro.gov.br/documentacao/api-integra-contador/pt/quick_start/
capturado_em: 2026-10-20
conversao: pendente-conferir-com-pdf
aviso: Cópia de trabalho. Em divergência, vale o documento oficial/PDF.
---

# Quick start — Integra Contador

Este documento descreve o fluxo de autenticação e a chamada de um serviço do Integra Contador.
O ambiente Trial do Integra Contador usa o Bearer público de demonstração; o ambiente de produção
usa autenticação na SAPI com certificado digital e-CNPJ (mTLS).

## 1. Obtenha Consumer Key e Consumer Secret

O `Consumer Key` e o `Consumer Secret` identificam o contrato do escritório com o SERPRO e são
obtidos na área do cliente. Mantenha esses valores protegidos.

## 2. Solicite o Bearer Token e o JWT Token

Para consumir as APIs em produção, é necessário obter um `access_token` (Bearer) e um `jwt_token`.
O token de acesso tem validade temporária; quando expirar, repita a autenticação.

**API protegida com Certificado Digital SSL**

Todo Bearer Token gerado solicita o certificado digital e-CNPJ padrão ICP-Brasil. Esse
certificado deve ser o mesmo utilizado na contratação do produto. A obrigatoriedade do e-CNPJ
segue o padrão de acesso aos sistemas da Receita Federal do Brasil (RFB).

### Como solicitar os tokens

Requisição HTTP `POST` para:

```http
POST https://autenticacao.sapi.serpro.gov.br/authenticate
Authorization: Basic base64(consumerKey:consumerSecret)
Role-Type: TERCEIROS
Content-Type: application/x-www-form-urlencoded

grant_type=client_credentials
```

Requisitos:

- certificado digital e-CNPJ padrão ICP-Brasil válido (arquivo `.p12`/`.pfx` e senha);
- `Authorization` com `Consumer Key` e `Consumer Secret` concatenados por `:` e convertidos em
  Base64;
- `Role-Type: TERCEIROS`;
- `Content-Type: application/x-www-form-urlencoded`.

Exemplo de chamada:

```bash
curl -i -X POST \
  -H "Authorization: Basic <BASE64_CONSUMER_KEY_CONSUMER_SECRET>" \
  -H "Role-Type: TERCEIROS" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d 'grant_type=client_credentials' \
  --cert-type P12 \
  --cert arquivo_certificado.p12:senha_certificado \
  https://autenticacao.sapi.serpro.gov.br/authenticate
```

Se ocorrer `415 Unsupported Media Type`, confirme o `Content-Type:
application/x-www-form-urlencoded`.

## 3. Receba os tokens

O retorno da autenticação contém `access_token`, `jwt_token`, `token_type` e `expires_in`:

```json
{
  "expires_in": 2008,
  "scope": "default",
  "token_type": "Bearer",
  "access_token": "<ACCESS_TOKEN>",
  "jwt_token": "<JWT_TOKEN>"
}
```

- `access_token` é enviado no header `Authorization: Bearer <access_token>`.
- `jwt_token` é enviado no header `jwt_token: <jwt_token>` nas chamadas de produção.
- Quando o token expirar, o gateway retorna `HTTP 401`; repita a autenticação na SAPI.

## 4. Faça uma requisição

Com os tokens em mãos, envie a requisição para o verbo desejado:

```http
POST https://gateway.apiserpro.serpro.gov.br/integra-contador/v1/Consultar
Authorization: Bearer <ACCESS_TOKEN>
jwt_token: <JWT_TOKEN>
Content-Type: application/json
Accept: application/json

{
  "contratante": { "numero": "00000000000000", "tipo": 2 },
  "autorPedidoDados": { "numero": "00000000000000", "tipo": 2 },
  "contribuinte": { "numero": "00000000000000", "tipo": 2 },
  "pedidoDados": {
    "idSistema": "PGDASD",
    "idServico": "CONSEXTRATO16",
    "versaoSistema": "1.0",
    "dados": "{ \"numeroDas\": \"99999999999999999\" }"
  }
}
```

A URL base de produção é `https://gateway.apiserpro.serpro.gov.br/integra-contador/v1` e o verbo
(`/Apoiar`, `/Consultar`, `/Declarar`, `/Emitir` ou `/Monitorar`) é concatenado a ela.

### Envelope de resposta

```json
{
  "contratante": { "numero": "string", "tipo": 2 },
  "autorPedidoDados": { "numero": "string", "tipo": 2 },
  "contribuinte": { "numero": "string", "tipo": 2 },
  "pedidoDados": {
    "idSistema": "string",
    "idServico": "string",
    "versaoSistema": "string",
    "dados": "string"
  },
  "status": 0,
  "dados": "string",
  "mensagens": [
    { "codigo": "string", "texto": "string" }
  ]
}
```

O campo `dados` é uma string contendo JSON específico do serviço, tanto na requisição quanto na
resposta.

## Observações

- No Trial, o Bearer público de demonstração é configurado em `SERPRO_TRIAL_BEARER_TOKEN` e não
  deve ser versionado nem embutido no frontend.
- A autenticação de produção do Integra Contador é distinta da autenticação da API de Consulta
  Dívida Ativa. Não reutilize tokens ou endpoints entre produtos.
- `versaoSistema` deve ser validada contra o catálogo oficial de cada serviço.
