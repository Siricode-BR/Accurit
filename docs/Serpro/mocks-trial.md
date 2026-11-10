# MOCKs do SERPRO em Ambiente TRIAL

## Configuração

O ambiente de mock do SERPRO é ativado pela combinação de duas variáveis de ambiente:

- `SERPRO_TRIAL_BEARER_TOKEN`: Se ausente, o mock está ativo (`SERPRO_TRIAL_MOCK=true`). Se presente, o mock está desativado (`SERPRO_TRIAL_MOCK=false`).
- `SERPRO_TRIAL_MOCK`: Define explicitamente se o mock deve ser usado. Padrão: `true` quando `SERPRO_TRIAL_BEARER_TOKEN` não está definido.

**Exemplo .env:**

```
SERPRO_TRIAL_BEARER_TOKEN=  # Deixa em branco ou remove para ativar mock
SERPRO_TRIAL_MOCK=true
```

**Exemplo docker-compose.yml:**

```yaml
environment:
  SERPRO_TRIAL_MOCK: true
```

## Cenários Disponíveis

Os mocks retornam respostas determinísticas baseadas no último dígito do documento fornecido (CPF/CNPJ/Inscrição). Cada cenário tem um resultado esperado definido:

| Últ. dígito | Cenário Dívida Ativa | Cenário Integra Contador | Resultado |
| ------------- | --------------------- | ------------------------- | ----------- |
| 0, 1, 2 | Sem pendências | Sucesso (consulta) | SUCESSO |
| 3, 4 | Com pendências | Consulta sucesso | SUCESSO |
| 5, 6 | Não encontrado | Não encontrado | NAO_ENCONTRADO |
| 7, 8 | Acesso negado | Acesso negado (sem certificado) | ACESSO_NEGADO |
| 9 | Limite excedido | Limite excedido | LIMITE_EXCEDIDO |
| 9 (var.) | - | Indisponível | INDISPONIVEL |
| 9 (var.) | - | ENVIOXMLASSINADO81 | SUCESSO |
| 9 (var.) | - | OBTERPROCURACAO41 | SUCESSO |

## Cenários Detalhados

### Dívida Ativa

| Cenário | Descrição | Código Result | Quando ocorrer |
| --------- | ----------- | -------------- | ---------------- |
| **Sem pendências** | Devedor com inscrição regular, sem protestos ou penhoras | SUCESSO | Últimos dígitos 0, 1, 2 |
| **Com pendências** | Devedor com protestos ou penhoras ativos | SUCESSO | Últimos dígitos 3, 4 |
| **Não encontrado** | Inscrição/devedor não encontrado na base | NAO_ENCONTRADO | Últimos dígitos 5, 6 |
| **Acesso negado** | Procuração negada ou inválida | ACESSO_NEGADO | Últimos dígitos 7, 8 |
| **Limite excedido** | Limite de consultas mensais excedido | LIMITE_EXCEDIDO | Último dígito 9 |

### Integra Contador

| Cenário | Descrição | Código Result | Quando ocorrer |
| --------- | ----------- | -------------- | ---------------- |
| **Sucesso (consulta)** | Consulta realizada com sucesso | SUCESSO | Último dígito 0, 1 |
| **Não encontrado** | Procuração ou inscrição não encontrada | NAO_ENCONTRADO | Último dígito 2 |
| **Acesso negado** | Procuração negada - certificado A1 inválido ou inexistente | ACESSO_NEGADO | Último dígito 3 |
| **Limite excedido** | Limite de chamadas/mês excedido | LIMITE_EXCEDIDO | Último dígito 4 |
| **Indisponível** | Serviço temporariamente indisponível | INDISPONIVEL | Último dígito 5 |
| **ENVIOXMLASSINADO81** | XML assinado (simulado) enviado com sucesso | SUCESSO | Último dígito 6 |
| **OBTERPROCURACAO41** | Procuração encontrada e validada com sucesso | SUCESSO | Último dígito 7 |
| **Resposta inválida** | Resposta do serviço inválida | RESPOSTA_INVALIDA | Último dígito >= 8 (default) |

## Uso no Frontend

### Banner "Ambiente Trial — respostas simuladas"

Quando o ambiente for TRIAL e o mock estiver ativo, exibir um banner no cliente:

```
┌─────────────────────────────────────────┐
│ AVISO: Ambiente Trial — respostas simuladas │
│ As respostas dos serviços SERPRO são simuladas. │
│ Para produção, configure credenciais reais. │
└─────────────────────────────────────────┘
```

### Painel "Cenários de teste"

Nas telas de consulta (Dívida Ativa, Integra Contador e detalhes do cliente), exibir um painel/drawer com:

1. Seleção de cenário (ou aleatório)
2. Botão "Usar documento" que preenche o campo de documento com o documento de exemplo do cenário selecionado
3. Selo "Simulado" nos resultados

### Serviço `trial-mock.service.ts` (frontend)

Serviço centralizado que:

1. Verifica se o ambiente é TRIAL e o mock está ativo
2. Redireciona chamadas SERPRO para respostas mock
3. Faz cache das respostas para evitar chamadas repetidas
4. Em caso de falha silenciosa, passa para o handler real

```typescript
// trial-mock.service.ts
import { injectable, inject } from 'inversify';
import { HttpClient } from '@angular/common/http';
import { Observable, of, throwError } from 'rxjs';
import { catchError, map, tap } from 'rxjs/operators';

@injectable()
export class TrialMockService {
  private readonly http = inject(HttpClient);
  private readonly baseUrl = '/api/v1/serpro/mock';
  private readonly cache = new Map<string, any>();
  private readonly ambiente = 'TRIAL'; // definido pelo contexto

  listarCenarios(): Observable<{ id: string; servico: string; titulo: string }[]> {
    return this.http.get<{ ativo: boolean; cenarios: any }>(`${this.baseUrl}/cenarios`).pipe(
      map(res => res.cenarios?.map((c: any) => ({
        id: c.id,
        servico: c.servico,
        titulo: c.titulo,
      }))) || [],
      catchError(() => of([]))
    );
  }

  usarCenario(servico: string, documento: string): Observable<any> {
    // Busca o cenário correspondente e retorna a resposta mock
    const cacheKey = `${servico}:${documento}`;
    
    if (this.cache.has(cacheKey)) {
      return of(this.cache.get(cacheKey));
    }
    
    return this.http.post<any>(`${this.baseUrl}/executar`, {
      servico,
      documento,
    }).pipe(
      tap(res => this.cache.set(cacheKey, res)),
      catchError(() => this.mockFallback(servico, documento))
    );
  }

  private mockFallback(servico: string, documento: string) {
    // Em caso de falha do mock, tenta o handler real
    // Isso garante que o fluxo não quebre completamente
    return of(null);
  }
}
```

## Registro de Consultas e Auditoria

Mesmo quando usando mock, é importante registrar as consultas e auditoria normalmente com ambiente TRIAL e origem MOCK. Isso permite:

1. Estatísticas de uso durante o período de teste
2. Identificação de quais cenários são mais acionados
3. Comprovamento para habilitação da produção

**Registro de auditoria (exemplo):**

```json
{
  "escritorio_id": "...",
  "administrador_id": "...",
  "acao": "CONSULTA_SERPRO_MOCK",
  "entidade": "divida_ativa|integra_contador|procuracao",
  "entidade_id": "...",
  "valor_anterior": null,
  "valor_novo": {
    "servico": "DIVIDA_ATIVA",
    "cenario": "SUCESSO",
    "ultimo_digito": 2,
    "ambiente": "TRIAL",
    "origem": "MOCK"
  },
  "ip": "...",
}
```

## Limitações e Considerações

1. **Nunca use mock em ambiente PRODUÇÃO** - O mock só deve ser ativado quando `SERPRO_TRIAL_MOCK=true` e o ambiente for TRIAL.

2. **Mock não substitui integração real** - Os mocks são para desenvolvimento e testes apenas. A integração com a SERPRO real deve ser concluída antes do go-live.

3. **Documentação de cenários** - Cada novo cenário deve ser documentado aqui com seu código de resultado e condição de acionamento.

4. **Consistência com medição (G.1)** - As chamadas mock devem passar pelo mesmo ponto de cliente da medição de chamadas, de modo que também sejam contadas com ambiente TRIAL para fins de relatório e cobrança (quando aplicável).

5. **Tipos de documento** - Os mocks aceitam CPF (11 dígitos), CNPJ (14 dígitos) e inscrições estaduais. O cenário é determinado pelo último dígito numérico do documento.
