# ADR-0005 — Alternador do Modo Trial SERPRO

- **Status:** Aceito
- **Data:** 2026-10-20
- **Decisores:** Arquitetura Accurit
- **Escopo:** Configuração de ambiente SERPRO por escritório e toggle no painel Admin
- **Relaciona-se com:** [ADR-0003](0003-envelope-verbos-integra-contador.md), [ADR-0004](0004-seguranca-cabecalho-assinatura-xml.md)

## Contexto

A Accurit precisa operar contra dois ambientes da SERPRO:

| Produto | Trial | Produção |
|---|---|---|
| Integra Contador | `https://gateway.apiserpro.serpro.gov.br/integra-contador-trial/v1` | `https://gateway.apiserpro.serpro.gov.br/integra-contador/v1` |
| Consulta Dívida Ativa | `https://gateway.apiserpro.serpro.gov.br/consulta-divida-ativa-trial/api/v1` | **Indefinida**; bloqueada com `SERPRO_PROD_NOT_CONFIGURED` até confirmação SERPRO (D5) |
| Token Dívida Ativa | `https://gateway.apiserpro.serpro.gov.br/token` | Mesmo endpoint, com credenciais de produção |
| Autenticação Integra Contador | Bearer público de demonstração | SAPI: `https://autenticacao.sapi.serpro.gov.br/authenticate` (mTLS e-CNPJ) |

O ambiente Trial usa a chave Bearer pública de demonstração fornecida pela SERPRO (`SERPRO_TRIAL_BEARER_TOKEN`). Produção do Integra Contador usa SAPI com mTLS e-CNPJ, retornando `access_token` e `jwt_token`; produção da Dívida Ativa usa OAuth2 com mTLS e certificado A1 do escritório, mas permanece bloqueada até confirmação da URL oficial.

O escritório precisa alternar entre os ambientes em tempo de execução, sem redeploy e sem misturar dados. O painel Admin deve oferecer um toggle reativo (“Modo Trial SERPRO”), mas a decisão efetiva de ambiente não pode depender do frontend: o backend é a autoridade.

Riscos a evitar:

- chamadas de produção acidentais durante demonstrações;
- uso de token/credencial de Trial em produção ou vice-versa;
- troca de ambiente no meio de uma requisição;
- cache de token ou de procuração contaminado entre ambientes;
- UI mostrando um estado diferente do persistido;
- alteração do toggle por usuário sem permissão;
- vazamento de token de demonstração no bundle Angular.

## Decisão

### 1. Persistência por escritório

O estado é persistido na coluna `modo_trial_serpro` da tabela `escritorios`, criada diretamente no
`CREATE TABLE` inicial (sem `ALTER`, conforme a regra pré-MVP da [ADR-0006](0006-migracao-automatica-banco-dados.md)):

```sql
CREATE TABLE escritorios (
    id UUID PRIMARY KEY,
    slug VARCHAR(40) NOT NULL UNIQUE CHECK (slug ~ '^[a-z0-9-]{3,40}$'),
    nome TEXT NOT NULL,
    cnpj VARCHAR(14) UNIQUE CHECK (cnpj IS NULL OR cnpj ~ '^[0-9]{14}$'),
    modo_trial_serpro BOOLEAN NOT NULL DEFAULT TRUE,
    versao BIGINT NOT NULL DEFAULT 1,
    atualizado_em TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

- `TRUE` significa Trial/homologação.
- `FALSE` significa Produção.
- O valor padrão é `TRUE` para que novos escritórios comecem em ambiente de demonstração.
- O estado é por tenant (`escritorio_id`), não global.
- A leitura e a escrita exigem isolamento por tenant.

### 2. Backend como autoridade

O backend resolve o ambiente no **início de cada requisição** e mantém esse valor como snapshot imutável até o fim da operação:

```text
Request
  -> autenticação e RequestContext
  -> leitura de escritorios.modo_trial_serpro
  -> ambiente_efetivo = TRIAL | PRODUCAO
  -> seleção de base URL, credencial, token e certificado
  -> execução da operação com o snapshot
  -> persistência de consultas_fiscais.ambiente
  -> resposta
```

Regras:

- Uma requisição nunca troca de ambiente no meio do processamento.
- Se o toggle mudar durante uma requisição em andamento, a requisição continua no ambiente em que começou; a próxima requisição usa o novo valor.
- O ambiente efetivo é registrado em `consultas_fiscais.ambiente` e em logs de auditoria.
- O frontend não envia o ambiente na chamada de negócio; ele apenas lê e altera a configuração.
- Não há fallback silencioso de Produção para Trial nem de Trial para Produção. Falha de configuração de Produção resulta em erro explícito.
- Token, procuração e cache são particionados por ambiente; um token de Trial nunca é usado em Produção.

### 3. API administrativa

| Método | Rota | Descrição |
|---|---|---|
| `GET` | `/api/v1/admin/serpro/configuracao` | Retorna o estado persistido e o ambiente efetivo |
| `PATCH` | `/api/v1/admin/serpro/trial` | Alterna o modo Trial do escritório |

Exemplo de request:

```http
PATCH /api/v1/admin/serpro/trial
Authorization: Bearer <jwt>
Content-Type: application/json

{ "modo_trial_serpro": false }
```

Exemplo de response:

```json
{
  "modo_trial_serpro": false,
  "ambiente_efetivo": "PRODUCAO",
  "atualizado_em": "2026-10-20T14:30:00Z"
}
```

Regras:

- A rota exige papel `ADMIN`; `OPERADOR` e `CLIENTE` recebem `403`.
- A alteração é auditada em `auditoria_eventos` com `quem`, `quando`, valor anterior, valor novo e IP.
- A resposta só é retornada após a persistência ser confirmada.
- Usar controle de concorrência otimista (versão ou `atualizado_em`). Conflito retorna `409` e a UI recarrega o estado.
- O endpoint não retorna tokens, credenciais ou certificados.
- A troca para Produção pode exigir confirmação explícita adicional na UI; a API valida que o tenant tem configuração de Produção antes de aceitar a mudança, retornando erro claro se faltar certificado/credencial.

Implementação (Fase 3):

- Os exemplos acima usam `modo_trial_serpro`; o contrato entregue usa **camelCase no JSON**
  (convenção de §10.1, ex. `accessToken`): `GET /api/v1/admin/escritorios` lista os escritórios
  com `modoTrialSerpro`/`versao`/`atualizadoEm` (paginado) e
  `PATCH /api/v1/admin/escritorios/{id}/serpro/trial` recebe `{ modoTrialSerpro, versao }` —
  o escritório alvo vem no path do PATCH (rota revisada por D7 de DECISOES-PENDENTES).
- A existência do escritório é verificada **antes** da validação de produção: alvo inexistente
  responde `404` mesmo que a produção não esteja configurada.
- Pré-requisitos de produção: credencial `INTEGRA_CONTADOR`/`PRODUCAO` e certificado A1 em
  validade; Dívida Ativa em produção responde sempre `SERPRO_PROD_NOT_CONFIGURED`.
- Após a troca persistida, os caches voláteis (token de procuração/produto) do escritório são
  invalidados antes de responder (§6, 7). O ambiente da requisição é resolvido uma única vez
  (snapshot no `RequestContext`) — trocas não afetam requisições em andamento.
- A plataforma do escritório consome `GET /api/v1/serpro/ambiente`
  (apenas `{ ambienteEfetivo, atualizadoEm }`, revisado por D8) para o selo permanente na
  página inicial e para os componentes das fases seguintes.
- Versão de referência da tela: `admin/src/app/features/tenants` (toggle pessimista por linha,
  modal de confirmação para produção, recarrega em `409`).

### 4. Toggle reativo no Admin com Angular Signals

O estado de tela é modelado com Signals, mas a fonte de verdade é o backend:

```ts
import { computed, signal } from '@angular/core';

export class SerproConfigStore {
  readonly modoTrial = signal<boolean>(true);
  readonly pending = signal(false);
  readonly erro = signal<string | null>(null);

  readonly ambiente = computed(() =>
    this.modoTrial() ? 'TRIAL' : 'PRODUCAO'
  );

  readonly rotulo = computed(() =>
    this.modoTrial() ? 'Modo Trial SERPRO' : 'Produção SERPRO'
  );
}
```

Comportamento:

- O componente carrega o estado via `GET /api/v1/admin/serpro/configuracao` e inicializa o Signal.
- O toggle é **pessimista**: fica desabilitado durante o `PATCH`; o Signal só é atualizado após resposta de sucesso.
- Em erro, o valor permanece o anterior e a UI exibe mensagem segura.
- A troca para Produção exige confirmação explícita (modal/diálogo) com aviso de que dados reais serão consultados.
- A UI exibe um badge permanente do ambiente ativo (`TRIAL` ou `PRODUCAO`) e marca resultados de Trial.
- A UI não decide o ambiente de uma chamada de negócio; ela apenas reflete o estado do servidor.
- Nenhum token, credencial ou certificado é armazenado no frontend.
- `computed()` é usado para valores derivados; não duplicar estado em variáveis mutáveis paralelas.

### 5. Seleção de endpoint e credencial

| Modo | Integra Contador | Dívida Ativa | Credencial |
|---|---|---|---|
| Trial | Base Trial | Base Trial | `SERPRO_TRIAL_BEARER_TOKEN` de ambiente |
| Produção | `https://gateway.apiserpro.serpro.gov.br/integra-contador/v1` | **Bloqueada** (`SERPRO_PROD_NOT_CONFIGURED`) | Integra: SAPI com mTLS e-CNPJ (`access_token` + `jwt_token`); Dívida Ativa: OAuth2 + mTLS + A1 |

Regras:

- As bases URL vêm de configuração de ambiente, não de constantes hardcoded no frontend.
- O token de Trial é configuração de ambiente e nunca é embutido no bundle Angular, no repositório ou em logs.
- Produção exige `consumer_key`/`consumer_secret` do escritório em `credenciais_serpro` (variáveis
  globais são apenas fallback de desenvolvimento), mTLS e certificado e-CNPJ/A1 válido; ausência
  resulta em erro explícito antes da chamada.
- O cache de token usa a chave `(escritorio_id, produto, ambiente, tipo_token, autor_pedido_dados)` e é invalidado na troca de modo e em `401`.
- O cache de procuração também inclui o ambiente; uma procuração verificada em Trial não vale para Produção.
- A troca de modo invalida caches voláteis do tenant para evitar reutilização indevida.
- Ativar Produção para a Dívida Ativa retorna `SERPRO_PROD_NOT_CONFIGURED` até a SERPRO confirmar a URL.

### 6. Efeitos da troca

Ao alternar o modo:

- novas requisições passam a usar o novo ambiente;
- requisições em andamento mantêm o ambiente anterior;
- caches de token/procuração do tenant são invalidados;
- a UI recarrega o estado e passa a exibir o novo badge;
- a alteração é auditada;
- resultados antigos não são apagados nem reinterpretados; cada consulta mantém o ambiente registrado.

## Consequências

### Positivas

- **Alternância sem redeploy:** o escritório pode demonstrar em Trial e operar em Produção sem nova imagem.
- **Isolamento por tenant:** um escritório não afeta o ambiente de outro.
- **Segurança:** o backend decide o ambiente; o frontend não pode forçar Produção ou Trial.
- **Auditoria:** toda troca registra autor, data, valores anterior/novo e IP.
- **UX clara:** o badge e o aviso de Trial reduzem o risco de interpretar dados de demonstração como reais.
- **Cache correto:** tokens e procurações são separados por ambiente.

### Negativas

- **Risco de chamada real acidental:** um clique em Produção pode consumir créditos e consultar dados reais; exige confirmação e permissão.
- **Complexidade de estado:** concorrência, cache e requisições em andamento precisam ser tratados.
- **Dois ambientes para testar:** a equipe precisa validar Trial e Produção separadamente.
- **Pré-requisitos de Produção:** certificado A1, mTLS e credenciais precisam estar corretos; erros de configuração bloqueiam a operação.
- **Resultados mistos:** consultas antigas de Trial e novas de Produção coexistem no histórico; a UI precisa marcar o ambiente.
- **Latência de confirmação:** o toggle pessimista não responde instantaneamente.

### Mitigações

- Confirmação explícita para entrar em Produção e badge visual permanente.
- Restringir a alteração ao papel `ADMIN`.
- Validar a configuração de Produção antes de permitir a troca.
- Invalidar caches do tenant na troca.
- Registrar `ambiente` em cada consulta e exibir na UI.
- Testar troca de modo, concorrência, permissão, cache e requisição em andamento.
- Monitorar chamadas por ambiente e alertar sobre uso inesperado de Produção.

## Conformidade e Regras de Negócio

1. **Persistência:** `escritorios.modo_trial_serpro` é `BOOLEAN NOT NULL DEFAULT TRUE`.
2. **Autoridade:** o backend lê o modo no início de cada requisição e registra o ambiente efetivo; o frontend não escolhe o ambiente de negócio.
3. **Sem troca em voo:** uma requisição não muda de ambiente no meio do processamento.
4. **Sem fallback silencioso:** falha em Produção não deve cair para Trial; falha em Trial não deve cair para Produção.
5. **Permissão:** somente `ADMIN` pode alterar o modo; a API retorna `403` para os demais papéis.
6. **Auditoria:** toda alteração registra quem, quando, valor anterior, valor novo e IP.
7. **Concorrência:** a API usa controle otimista e retorna `409` em conflito.
8. **Cache:** tokens e procurações são particionados por `(escritorio_id, produto, ambiente)` e invalidados na troca.
9. **Trial:** usa `SERPRO_TRIAL_BEARER_TOKEN` de ambiente; o valor não é versionado nem embutido no frontend.
10. **Produção:** Integra Contador exige SAPI, mTLS e e-CNPJ; Dívida Ativa exige OAuth2, mTLS e
    A1, mas permanece bloqueada por `SERPRO_PROD_NOT_CONFIGURED` até confirmação da SERPRO.
11. **UI:** o toggle usa Signals e atualização pessimista; o estado só muda após confirmação do backend.
12. **Resultados:** toda consulta registra `ambiente`; resultados de Trial são visualmente marcados.
13. **Segredos:** o frontend nunca recebe token, `Consumer Secret`, certificado ou chave privada.

## Critérios de verificação

- O toggle persiste por escritório e alterna entre Trial e Produção em tempo de execução.
- Um usuário sem papel `ADMIN` recebe `403` ao tentar alterar o modo.
- A troca para Produção exige confirmação e valida a configuração do tenant.
- Requisições em andamento mantêm o ambiente com que começaram.
- Tokens e procurações de Trial não são reutilizados em Produção.
- A troca invalida caches voláteis do tenant.
- Toda consulta registra o ambiente efetivo e a UI o exibe.
- Nenhum segredo de Trial ou Produção aparece no bundle Angular ou em logs.
- Testes cobrem permissão, concorrência, falha de configuração, troca durante requisição e ausência de fallback silencioso.

## Referências

- [README — Configuração SERPRO](../../README.md)
- [SPEC — Modo Trial SERPRO](../../SPEC.md)
- [ROADMAP — Fase 3](../../ROADMAP.md)
- [ADR-0003 — Envelope e verbos do Integra Contador](0003-envelope-verbos-integra-contador.md)
- [ADR-0004 — Segurança, cabeçalho e assinatura XML](0004-seguranca-cabecalho-assinatura-xml.md)
