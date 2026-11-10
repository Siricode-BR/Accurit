# Mocks do SERPRO em Trial

O simulador é controlado por `SERPRO_TRIAL_MOCK` e **só** pode ser usado quando o
ambiente efetivo do escritório é `TRIAL`.

- Padrão: `true` quando `SERPRO_TRIAL_BEARER_TOKEN` está ausente; `false` quando o token
  está presente.
- `SERPRO_TRIAL_MOCK=false` desliga o simulador mesmo em Trial.
- Em `PRODUCAO` o simulador nunca é usado, independentemente da variável
  (`infrastructure::serpro::mock::mock_ativo` garante isso e há teste unitário).

As respostas são determinísticas e derivadas do **último dígito** do documento
(CPF/CNPJ do devedor, do contribuinte ou do número da inscrição).

## Dívida Ativa (`GET /devedor/{documento}` e `GET /inscricao/{numero}`)

| Último dígito | Resultado | HTTP/código observável no portal |
| --- | --- | --- |
| 0–1 | Com pendências (duas inscrições simuladas) | 200 `COM_PENDENCIAS` |
| 2–3 | Sem pendências | 200 `SEM_PENDENCIAS` |
| 4 | Não encontrado | 404 `SERPRO_NOT_FOUND` |
| 5 | Procuração/acesso negado | 403 `SERPRO_PROXY_DENIED` |
| 6 | Limite excedido | 429 `SERPRO_RATE_LIMITED` |
| 7 | Indisponível | 503 `SERPRO_UNAVAILABLE` |
| 8 | Indeterminado (sem retry) | 504 `SERPRO_INDETERMINATE` |
| 9 | Resposta inválida | 502 `SERPRO_BAD_RESPONSE` |

## Integra Contador / Procurações

| Último dígito | Resultado |
| --- | --- |
| 0–3, 5 | Execução concluída; `dados` contém `{"origem":"MOCK", ...}` |
| 4 | `403 ACESSO_NEGADO` (OBTERPROCURACAO41/autenticar) |
| 6 | `429 LIMITE_EXCEDIDO` |
| 7 | `503 INDISPONIVEL` |
| 8 | `504 INDETERMINADO` |
| 9 | `502 RESPOSTA_INVALIDA` |

O `ENVIOXMLASSINADO81` é simulado sem certificado A1 real e sem mTLS; devolve um
`jwt_procurador` fictício válido até a meia-noite seguinte. Nenhuma resposta contém
segredos, certificados ou URLs internas.

## Endpoint de cenários

`GET /api/v1/serpro/mock/cenarios` (audiência `client`) devolve:

```json
{
  "ativo": true,
  "cenarios": [
    {
      "id": "divida-com-pendencias",
      "servico": "DIVIDA_ATIVA",
      "titulo": "Dívida Ativa — com pendências",
      "descricao": "Devedor com inscrições ativas; documento terminado em 0 ou 1.",
      "documentoExemplo": "52998224725",
      "resultadoEsperado": "COM_PENDENCIAS"
    }
  ]
}
```

Fora de Trial ou com o mock desativado: `{"ativo": false, "cenarios": []}`.

## Medição e auditoria

As chamadas simuladas atravessam o mesmo ponto de cliente que a medição decora
(`ClienteDividaAtivaMock`/`ClienteIntegraContadorMock` em volta do cliente HTTP). Por
isso continuam sendo registradas em `chamadas_serpro`/`consultas_fiscais` com
`ambiente = TRIAL`, e a UI exibe o selo **Simulado**.
