# Observabilidade

## Métricas Prometheus

`GET /api/v1/metrics` exige `Authorization: Bearer <METRICS_TOKEN>`; sem token configurado a
rota responde `403` (fail closed). Métricas expostas:

| Métrica | Tipo | Descrição |
| --- | --- | --- |
| `accurit_http_requests_total{classe}` | counter | Requisições por classe 2xx/3xx/4xx/5xx |
| `accurit_http_request_duration_ms_sum` | counter | Soma das durações HTTP |
| `accurit_http_request_duration_ms_count` | counter | Quantidade de observações |
| `accurit_serpro_calls_total` | counter | Chamadas SERPRO mediadas |
| `accurit_serpro_errors_total` | counter | Chamadas SERPRO com erro |
| `accurit_uptime_seconds` | gauge | Tempo desde o start |
| `accurit_info{versao,ambiente}` | gauge | Versão e ambiente |

`GET /api/v1/health/detalhado` devolve versão, ambiente, uptime, status do banco e contadores.

## Logs

- Todo request recebe `x-request-id` (aceito se válido, senão gerado) e o span estruturado
  `http_request` inclui método, path e `request_id`.
- Payloads sensíveis (CPF/CNPJ, tokens, certificados, credenciais) não são logados.
- O frontend nunca imprime `HttpErrorResponse` em produção; erros exibem mensagem amigável e
  `Código de suporte: <requestId>` em 5xx.

## Alertas de exemplo

```yaml
groups:
  - name: accurit
    rules:
      - alert: AccuritErroSerproAlto
        expr: rate(accurit_serpro_errors_total[5m]) > 0.05
        for: 10m
        labels: { severity: warning }
        annotations:
          summary: "Taxa de erro SERPRO acima de 5%"
      - alert: AccuritLatenciaAlta
        expr: rate(accurit_http_request_duration_ms_sum[5m]) / rate(accurit_http_request_duration_ms_count[5m]) > 1500
        for: 10m
        labels: { severity: warning }
        annotations:
          summary: "Latência HTTP média acima de 1,5s"
      - alert: AccuritErros5xx
        expr: increase(accurit_http_requests_total{classe="5xx"}[5m]) > 20
        for: 5m
        labels: { severity: critical }
        annotations:
          summary: "Aumento de respostas 5xx"
      - alert: AccuritCotaConsumo
        expr: increase(accurit_serpro_calls_total[1h]) > 0
        for: 1h
        labels: { severity: info }
        annotations:
          summary: "Consumo SERPRO ativo; conferir cotas por tenant no painel /consumo"
```

## Painel por tenant

O app admin (`/consumo`) mostra, por escritório/produto/ambiente: total de chamadas, taxa de
erro, duração média, ranking, série diária e exportação CSV. O app client (`/consumo`) mostra
o consumo do mês, limites do plano, avisos em 80%/100% e histórico recente.
