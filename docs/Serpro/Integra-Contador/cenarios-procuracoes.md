---
origem_url: https://apicenter.estaleiro.serpro.gov.br/documentacao/api-integra-contador/pt/cenarios_trial/cenarios_procuracoes/
capturado_em: 2026-10-20
conversao: pendente-conferir-com-pdf
aviso: Cópia de trabalho. Em divergência, vale o documento oficial/PDF.
---

# Cenários Trial — Procurações

## Obter Procurações

Esta simulação retorna as procurações eletrônicas entre um outorgante titular e seu respectivo
procurador. Para cada procuração encontrada, são retornadas a quantidade de sistemas na
procuração, o nome do sistema e a data de expiração.

### Requisição

```http
POST https://gateway.apiserpro.serpro.gov.br/integra-contador-trial/v1/Consultar
accept: text/plain
Authorization: Bearer <TOKEN_TRIAL_PUBLICO>
Content-Type: application/json

{
  "contratante": { "numero": "99999999999999", "tipo": 2 },
  "autorPedidoDados": { "numero": "99999999999999", "tipo": 2 },
  "contribuinte": { "numero": "99999999999", "tipo": 1 },
  "pedidoDados": {
    "idSistema": "PROCURACOES",
    "idServico": "OBTERPROCURACAO41",
    "versaoSistema": "1",
    "dados": "{ \"outorgante\":\"99999999999999\", \"tipoOutorgante\": \"2\", \"outorgado\":\"99999999999\", \"tipoOutorgado\":\"1\" }"
  }
}
```

### Observações

- `idSistema` = `PROCURACOES`
- `idServico` = `OBTERPROCURACAO41`
- `versaoSistema` = `"1"` (valor observado no cenário oficial)
- O payload de `dados` é uma string JSON com `outorgante`, `tipoOutorgante`, `outorgado` e
  `tipoOutorgado`.
- Quando a operação é executada em nome de procurador, o header
  `autenticar_procurador_token` deve ser enviado nas requisições do Integra Contador.
- O serviço `OBTERPROCURACAO41` é do tipo `Consultar` e não exige pré-validação de si mesmo.
