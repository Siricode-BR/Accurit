# ADR-0018 — Medição de uso SERPRO (FASE H — Produção)

Status: aceito

## Contexto

A plataforma precisa contar chamadas de saída ao SERPRO para fins de cobrança, distinguindo
ambientes `TRIAL` e `PRODUCAO`. A medição deve cobrir todos os caminhos de código (consultas
manuais, rotinas agendadas, lotes, retries, fallbacks, chamadas de procuração/autenticação) e
não pode armazenar payloads sensíveis.

Antes desta decisão, existiam chamadas diretas aos clientes HTTP (`ClienteIntegraContadorImpl`,
`ClienteDividaAtivaHttp`) sem qualquer contagem. Não havia mecanismo para distinguir TRIAL de
PRODUCAO para cobrança, nem limites de chamadas/mês por escritório.

## Decisão

### 1. Ponto único de medição

Toda chamada de saída ao SERPRO é contada no serviço de medição `ServicoMedicaoSerpro`, invocado
dos casos de uso da aplicação (`ExecutarIntegraContadorUseCase`, `ConsultarDevedorUseCase`,
`ConsultarInscricaoUseCase`) imediatamente após a chamada ao cliente HTTP. Isso garante que:

- Nenhum caminho de código escapa da medição.
- Retries contam cada tentativa enviada.
- Chamadas bloqueadas localmente (rate limit, produção não configurada, validação de parâmetro)
  são registradas à parte em `bloqueios_serpro` e não contam para cobrança.

### 2. Modelo de dados

Cinco tabelas novas (apenas `CREATE TABLE`, sem `ALTER` em migrations existentes):

| Tabela | Propósito |
|--------|-----------|
| `chamadas_serpro` | Registro bruto, append-only, sem payload sensível. Inclui `ambiente` (TRIAL/PRODUCAO) para cobrança separada. |
| `agregados_consumo_serpro` | Agregados por escritório/produto/ambiente/dia, atualizados atomicamente (upsert). |
| `planos_escritorio` | Definição de planos (chamadas/mês, preço). |
| `assinaturas_escritorio` | Assinatura ativa por escritório (plano, limite, bloqueio ao exceder). |
| `bloqueios_serpro` | Chamadas bloqueadas localmente (não contam para cobrança). |

### 3. Contexto de medição

O `ServicoMedicaoSerpro::registrar_chamada` recebe um `ComandoRegistrarChamada` com:

- `escritorio_id`, `usuario_id`, `cliente_id` — do contexto autenticado (RequestContext).
- `produto`, `verbo`, `servico`, `ambiente` — da chamada SERPRO.
- `origem` — `manual`, `rotina` ou `lote`.
- `resultado`, `status_http`, `duracao_ms`, `request_id` — da resposta HTTP.

### 4. Limites e cobrança

Antes de registrar cada chamada, o serviço verifica se o escritório tem assinatura ativa e está
dentro do limite de chamadas/mês. Ao exceder:

- `bloqueio_ao_exceder = true`: `LimiteChamadasExcedido`, chamada não registrada.
- `bloqueio_ao_exceder = false`: chamada registrada, escritório notificado (80% e 100%).

### 5. Consequências

Positivo:
- Cobrança separada por ambiente (trial vs produção).
- Limites configuráveis por escritório/plano.
- Auditoria completa de todas as chamadas SERPRO.
- Sem armazenamento de payloads sensíveis.

Negativo:
- Complexidade adicional nos casos de uso da aplicação (injeção de `ServicoMedicaoSerpro`).
- Dependência da precisão do `request_id` para correlação.

## Alternativas consideradas

| Alternativa | Motivo da rejeição |
|-------------|-------------------|
| Medição no cliente HTTP (infraestrutura) | Falta de contexto de negócio (`escritorio_id`, `usuario_id`) na camada de infraestrutura. |
| Medição por middleware Axum | Não teria acesso ao contexto de negócio nem ao `request_id` da chamada SERPRO. |
| Medição assíncrona (fila) | Aumento de latência e complexidade; síncrono é suficiente para volumes esperados. |

## Referências

- [`docs/GO-LIVE.md`](docs/GO-LIVE.md) — Checklist de go-live.
- [`SPEC.md` §10.5](SPEC.md) — Especificação da medição de uso.
