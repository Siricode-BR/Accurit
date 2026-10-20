# ADR-0003 — Envelope JSON e Verbos do Integra Contador

- **Status:** Aceito
- **Data:** 2026-10-20
- **Decisores:** Arquitetura Accurit
- **Escopo:** Integração com a API Integra Contador da SERPRO
- **Relaciona-se com:** [ADR-0002](0002-escopo-consultivo-divida-ativa.md), [ADR-0004](0004-seguranca-cabecalho-assinatura-xml.md), [ADR-0005](0005-alternador-modo-trial-serpro.md)

## Contexto

O Integra Contador é uma plataforma SERPRO orientada a serviços. Todas as operações usam `POST` e compartilham o mesmo envelope JSON de entrada, mas cada serviço é identificado por `idSistema`, `idServico` e `versaoSistema`, com um payload específico em `pedidoDados.dados`.

Os cinco verbos operacionais são:

| Verbo | Método | Path | Uso |
|---|---|---|---|
| Apoiar | `POST` | `/Apoiar` | Serviços auxiliares de suporte à solução |
| Consultar | `POST` | `/Consultar` | Consulta de dados autorizados |
| Declarar | `POST` | `/Declarar` | Entrega/transmissão de declaração |
| Emitir | `POST` | `/Emitir` | Emissão/geração de documento de arrecadação |
| Monitorar | `POST` | `/Monitorar` | Acompanhamento de processamento/eventos |

A base Trial é `https://gateway.apiserpro.serpro.gov.br/integra-contador-trial/v1`; a base de
produção é `https://gateway.apiserpro.serpro.gov.br/integra-contador/v1`. No Trial usa-se o Bearer
público de demonstração. Em produção, a autenticação é feita na SAPI
(`https://autenticacao.sapi.serpro.gov.br/authenticate`) com mTLS e-CNPJ, `Role-Type: TERCEIROS` e
`Content-Type: application/x-www-form-urlencoded`; a resposta traz `access_token` e `jwt_token`, e
as chamadas enviam ambos. `autenticar_procurador_token` é adicionado quando o autor do pedido não é
o contratante. Todos esses headers são injetados exclusivamente pelo backend.

Muitos serviços exigem procuração eletrônica no e-CAC **somente quando `autorPedidoDados` difere do
contribuinte**. A verificação deve ocorrer **antes** do disparo do serviço operacional, usando o
serviço `PROCURACOES / OBTERPROCURACAO41` (tipo `Consultar`, `versaoSistema = "1"`). Uma linha
local na tabela `procuracoes` não é prova suficiente de autorização atual.

Sem uma camada única de envelope e de pré-validação, cada serviço tende a duplicar montagem de payload, headers, tratamento de erro e auditoria, aumentando o risco de divergência de contrato e de chamadas não autorizadas.

## Decisão

### 1. Um único adapter, cinco verbos

O backend terá um único adapter para o Integra Contador, parametrizado por um enum de verbo e por uma entrada de catálogo. Não haverá um cliente HTTP específico por serviço.

```rust
pub enum VerboIntegraContador {
    Apoiar,
    Consultar,
    Declarar,
    Emitir,
    Monitorar,
}

impl VerboIntegraContador {
    pub fn path(&self) -> &'static str {
        match self {
            Self::Apoiar => "/Apoiar",
            Self::Consultar => "/Consultar",
            Self::Declarar => "/Declarar",
            Self::Emitir => "/Emitir",
            Self::Monitorar => "/Monitorar",
        }
    }
}
```

A API interna da Accurit expõe `POST /api/v1/integra-contador/{verbo}`, com `verbo` restrito a `apoiar`, `consultar`, `declarar`, `emitir` e `monitorar`. O verbo é validado contra o tipo do serviço no catálogo; não é permitido acionar um serviço por um verbo diferente do classificado oficialmente.

### 2. Envelope de entrada de quatro camadas

O contrato canônico é:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identificacao {
    /// CPF (11 dígitos) ou CNPJ (14 dígitos), somente números.
    pub numero: String,
    /// 1 = CPF; 2 = CNPJ.
    pub tipo: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PedidoDados {
    pub id_sistema: String,
    pub id_servico: String,
    pub versao_sistema: String,
    /// JSON específico do serviço, serializado como string.
    pub dados: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegraContadorRequest {
    /// Escritório contratante do serviço SERPRO.
    pub contratante: Identificacao,
    /// Certificado A1 ou procurador autorizado.
    pub autor_pedido_dados: Identificacao,
    /// Cliente da contabilidade sobre quem a operação é executada.
    pub contribuinte: Identificacao,
    pub pedido_dados: PedidoDados,
}
```

Exemplo de corpo:

```json
{
  "contratante": { "numero": "99999999999999", "tipo": 2 },
  "autorPedidoDados": { "numero": "00000000000000", "tipo": 2 },
  "contribuinte": { "numero": "11111111111111", "tipo": 2 },
  "pedidoDados": {
    "idSistema": "PGDASD",
    "idServico": "CONSDECLARACAO13",
    "versaoSistema": "1.0",
    "dados": "{\"periodoAtual\":\"202601\"}"
  }
}
```

Regras do envelope:

- `contratante`, `autorPedidoDados`, `contribuinte` são obrigatórios e usam `Identificacao`.
- `pedidoDados.dados` é **string JSON escapada**, não um objeto JSON. O adapter serializa o payload específico com `serde_json::to_string` e o inclui como string.
- `idSistema`, `idServico` e `versaoSistema` são validados contra o catálogo vigente antes da
  chamada. `versaoSistema` é **por serviço**: `OBTERPROCURACAO41` usa `"1"` e
  `ENVIOXMLASSINADO81` usa `"1.0"`. O exemplo acima é ilustrativo; não generalizar.
- `tipo` é validado contra o ambiente e o serviço; alguns serviços Trial aceitam apenas CNPJ (`tipo = 2`).
- A serialização usa camelCase para os nomes do contrato SERPRO.
- O payload específico é validado por um schema/validador do serviço antes de ser serializado. Não existe “payload genérico” sem contrato.

### 3. Envelope de resposta

```rust
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegraContadorResponse {
    pub status: i32,
    #[serde(default)]
    pub mensagens: Vec<MensagemSerpro>,
    #[serde(default)]
    pub dados: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MensagemSerpro {
    pub codigo: String,
    pub descricao: String,
    #[serde(rename = "tipo")]
    pub tipo: Option<String>,
}
```

O campo `dados` da resposta também é uma string contendo JSON específico do serviço. O backend deve:

- interpretar `status` e `mensagens` conforme o contrato do serviço;
- mapear erros de negócio para códigos internos estáveis, sem convertê-los automaticamente em HTTP 500;
- nunca repassar a resposta bruta ao frontend sem mapeamento para DTOs internos;
- persistir apenas metadados e versões redigidas, nunca o payload fiscal completo sem política explícita.

### 4. Pré-validação de procuração e-CAC

Antes de disparar um serviço que exige representação, **somente quando `autorPedidoDados` difere do
contribuinte**, o backend executa uma chamada de pré-validação:

1. Identifica `contratante`, `autorPedidoDados`, `contribuinte` e o serviço solicitado.
2. Consulta o catálogo interno para obter a `RegraProcuracao`:
   `NaoSeAplica`, `AnyOf(Vec<String>)` ou `Condicional { codigos, condicao }`.
3. Se `NaoSeAplica` ou se o autor é o próprio contribuinte, segue sem a chamada de pré-validação.
4. Se `AnyOf`, aceita qualquer código listado; se `Condicional`, avalia a condição antes de escolher.
5. Executa `POST /Consultar` com:
   - `pedidoDados.idSistema = "PROCURACOES"`;
   - `pedidoDados.idServico = "OBTERPROCURACAO41"`;
   - `pedidoDados.versaoSistema = "1"`;
   - `dados = {"outorgante":"...","tipoOutorgante":"2","outorgado":"...","tipoOutorgado":"1"}`.
6. Interpreta a resposta e verifica contribuinte, procurador/autor, serviço, validade e situação.
7. Nega a execução quando a procuração estiver ausente, vencida, cancelada, incompatível ou não verificável.
8. Registra auditoria mínima: resultado, data, serviço, ambiente e referência, sem expor dados sensíveis.

Mapa de referência (confirmar no catálogo/PDF vigente):

| Sistema | Código(s) de procuração | Serviços de referência |
|---|---|---|
| PGDASD (inclui DEFIS) | `00146` | `CONSDECLARACAO13`, `TRANSDECLARACAO11`, `CONSEXTRATO16` |
| DCTFWeb | `00103` | `GERARGUIA31`, `TRANSDECLARACAO310` |
| SITFIS | `00002` | `RELATORIOSITFIS92`, `SOLICITARPROTOCOLO91` |
| Caixa Postal | `00006` | `INNOVAMSG63`, `MSGCONTRIBUINTE61` |
| DTE | `00050` | `CONSULTASITUACAODTE111` |
| Pagamentos (`PAGTOWEB`) | `00004` | `PAGAMENTOS71`, `COMPARRECADACAO72`, `CONTACONSDOCARRPG73` |
| Regime de apuração (`REGIMEAPURACAO`) | `00060` | `EFETUAROPCAOREGIME101` e consultas |
| MEI (`DASNSIMEI`) | `00229` | `CONSULTIMADECREC152`, `GERARDASEXCESSO153` |
| e-Processo (`EPROCESSO`) | `00051` | `CONSPROCPORINTER271` e demais |
| Parcelamentos (`PARCSN`) | `00076` **ou** `00188` | `AnyOf`; sistema bloqueado pela allowlist (ADR-0002) |
| Parcelamento Especial (`PARCSN-ESP`) | `00125` | Sistema bloqueado pela allowlist |
| `EVENTOSATUALIZACAO` | `n/a` / `sim¹` | `sim¹` mapeado para `00103`; validar caso a caso |

Observações obrigatórias:

- `PROCURACOES / OBTERPROCURACAO41` é classificado como `Consultar` e **não** exige pré-validação de si mesmo; a chamada deve evitar recursão.
- `AUTENTICAPROCURADOR / ENVIOXMLASSINADO81` é classificado como `Apoiar` e não exige procuração e-CAC para ser executado; ele produz o token de autorização do procurador.
- Serviços marcados como `n/a` no catálogo não disparam a pré-validação.
- A ausência de procuração não pode ser “compensada” por uma linha local. O cache local é apenas otimização, particionado por `(escritorio_id, ambiente, contribuinte, procurador, código)`, e deve ser invalidado após falha de autorização (`403` / `AcessoNegado-ICGERENCIADOR-022`).
- O código de procuração e a `RegraProcuracao` devem ser mantidos em configuração versionada e cobertos por testes de catálogo.
- `RELATORIOSITFIS92` é **Emitir**, não `Consultar`; `SOLICITARPROTOCOLO91` é **Apoiar**.

### 5. Fluxo de execução

```text
POST /api/v1/integra-contador/{verbo}
  -> autenticação JWT e resolução de escritorio_id
  -> autorização por papel
  -> resolução do ambiente (Trial/Produção) no início da requisição
  -> validação do verbo, catálogo e payload específico
  -> normalização de CPF/CNPJ e valores
  -> pré-validação de procuração (quando exigida)
  -> injeção de headers (x-cpf-usuario, Authorization, jwt_token, autenticar_procurador_token)
  -> assinatura XML, quando o serviço exigir
  -> POST /{Verbo} no Integra Contador
  -> mapeamento da resposta e auditoria
  -> resposta JSON interna
```

### 6. Headers e autenticação

Toda chamada ao Integra Contador inclui:

- `Authorization: Bearer <access_token>`; no Trial, o Bearer público de demonstração; em produção, o
  `access_token` retornado pela SAPI;
- `jwt_token: <jwt_token>` em produção, retornado pela SAPI junto ao `access_token`;
- `x-cpf-usuario: <CPF do operador autenticado, 11 dígitos>` **quando
  `SERPRO_OPERATOR_HEADER_ENABLED=true`**; header operacional não confirmado nos docs oficiais;
- `Content-Type: application/json`;
- `autenticar_procurador_token` quando `autorPedidoDados` difere do contratante;
- `X-Request-Tag` opcional (até 32 caracteres) para correlação.

Os headers são injetados exclusivamente pelo backend. O frontend não recebe, não escolhe e não sobrescreve tokens, certificados ou o CPF do operador. Detalhes de segurança e assinatura estão na [ADR-0004](0004-seguranca-cabecalho-assinatura-xml.md).

### 7. Idempotência, retry e timeout

- `Consultar`, `Apoiar` e `Monitorar` podem ser repetidos **uma vez** apenas em falhas transitórias
  (timeout, 5xx, 429) e quando a operação for idempotente.
- `Declarar` e `Emitir` são potencialmente não idempotentes: exigem chave de idempotência/controle de duplicidade e confirmação explícita. Não podem ser repetidos automaticamente sem verificar se a RFB já recebeu a solicitação.
- `202 Accepted`: processamento assíncrono; aguardar o `tempoEspera` antes de nova consulta. Cada
  polling é uma nova requisição potencialmente bilhetada.
- `204 No Content`: processamento ainda não concluído; aguardar o novo tempo estimado no header
  `eTag` antes de repetir.
- `304 Not Modified`: token do Termo ainda válido; reutilizar o valor em cache. O `eTag` tem o
  formato `autenticar_procurador_token:<token>` e `expires` informa a validade.
- Não são bilhetados: `204`, `304`, `400`, `401`, `404`, `429`, `500`, `503`, `504` e as chamadas a
  `/Apoiar` e `/Monitorar`.
- Timeout de conexão e de requisição segue o contrato: referência de 5 s para conexão e 30 s para requisição, respeitando o teto do gateway.
- `HTTP 504` é estado indeterminado: não reenviar imediatamente; consultar o status quando possível e reenviar apenas após confirmar que a solicitação não foi recebida.
- `HTTP 429` deve respeitar `Retry-After`; retries usam backoff exponencial com jitter e limite máximo.
- Erros `401` de OAuth2/SAPI podem disparar no máximo uma renovação de token e uma repetição controlada.

### 8. Mapeamento de erros

| Situação | Tratamento |
|---|---|
| `400` / `EntradaIncorreta-ICGERENCIADOR-*` | Erro de validação interno; não repetir; registrar detalhe redigido |
| `401` | Renovar token uma vez; se persistir, falha de autenticação explícita |
| `403` / `AcessoNegado-ICGERENCIADOR-022` | Falha de autorização/procuração; não repetir; invalidar cache de procuração |
| `429` | Respeitar `Retry-After`; backoff com jitter |
| `500` / `503` | Falha transitória do provedor; retry limitado apenas para operações idempotentes |
| `504` | Estado indeterminado; não reenviar automaticamente |
| `200` com `status`/`mensagens` de negócio | Mapear para erro de domínio estável, sem HTTP 500 genérico |
| `202 Accepted` | Processamento assíncrono; aguardar `tempoEspera` e consultar depois |
| `204 No Content` | Processamento pendente; aguardar `eTag` e repetir com cautela |
| `304 Not Modified` | Token do Termo válido; reutilizar cache de `autenticar_procurador_token` |

## Consequências

### Positivas

- **Contrato único:** todos os serviços usam o mesmo envelope, reduzindo divergências e duplicação de código.
- **Pré-validação central:** a checagem de procuração ocorre em um único ponto, antes de qualquer chamada operacional.
- **Segurança por padrão:** headers sensíveis são injetados no backend, nunca no frontend.
- **Testabilidade:** o adapter pode ser testado por verbo, catálogo e contrato de resposta.
- **Auditoria consistente:** cada chamada registra verbo, serviço, ambiente, tenant e resultado.
- **Evolução do catálogo:** novos serviços podem ser adicionados por configuração e validador específico, sem criar um novo cliente HTTP.

### Negativas

- **Payloads continuam específicos:** o envelope é comum, mas cada serviço exige schema, validação e mapeamento próprios.
- **Latência adicional:** a pré-validação de procuração adiciona uma chamada antes do serviço operacional.
- **Complexidade de cache:** é preciso definir TTL, chave e invalidação do cache de procuração.
- **Risco de drift de catálogo:** `idSistema`, `idServico`, `versaoSistema` e códigos de procuração podem mudar; é necessário atualizar a configuração e os testes.
- **Idempotência difícil:** `Declarar` e `Emitir` exigem controles adicionais e tratamento cuidadoso de timeout.
- **Erros heterogêneos:** o envelope de resposta pode variar por serviço/ambiente; o mapeamento precisa tolerar variações.

### Mitigações

- Manter o catálogo em configuração versionada, com validação de schema no startup/testes.
- Criar contract tests por verbo e por serviço de referência.
- Cachear procuração com TTL curto e invalidar em `403`.
- Medir latência da pré-validação e do serviço operacional separadamente.
- Persistir chave de idempotência e estado de operações mutáveis.
- Redigir logs e nunca persistir payload fiscal completo sem política explícita.

## Conformidade e Regras de Negócio

1. **Cinco verbos, sem rotas adicionais:** a API interna só pode expor `apoiar`, `consultar`, `declarar`, `emitir` e `monitorar`; cada um mapeia para o path correspondente.
2. **Envelope obrigatório:** toda chamada usa `contratante`, `autorPedidoDados`, `contribuinte` e `pedidoDados`; campos ausentes ou fora do contrato são rejeitados antes do envio.
3. **`dados` como string:** o payload específico é serializado como string JSON; não pode ser enviado como objeto.
4. **Catálogo como fonte de verdade:** `idSistema`, `idServico`, `versaoSistema` e o verbo devem existir no catálogo vigente. Serviço inexistente ou verbo incompatível resulta em erro de validação.
5. **Procuração obrigatória quando aplicável:** serviços que exigem representação só podem ser executados após `OBTERPROCURACAO41` retornar autorização válida **e somente quando `autorPedidoDados` difere do contribuinte**. A ausência, expiração, cancelamento ou incompatibilidade bloqueia a chamada.
6. **Fail-closed:** se a verificação de procuração falhar por erro de transporte ou resposta ambígua, a operação não é executada.
7. **Headers:** toda chamada inclui `Authorization`; no Trial, o Bearer demo; em produção,
   `Authorization` + `jwt_token`. `x-cpf-usuario` é operacional e condicionado a
   `SERPRO_OPERATOR_HEADER_ENABLED`. Demais headers são injetados pelo backend conforme o contrato.
8. **Assinatura:** quando o serviço exigir XML assinado, a assinatura segue a [ADR-0004](0004-seguranca-cabecalho-assinatura-xml.md) e ocorre antes da montagem do envelope.
9. **Idempotência:** `Declarar` e `Emitir` exigem chave/estratégia de idempotência e não podem ser repetidos automaticamente sem confirmação.
10. **Timeout:** `504` é estado indeterminado; não reenviar imediatamente. Respeitar `Retry-After` em `429`.
11. **Auditoria:** registrar verbo, `idSistema`, `idServico`, ambiente, tenant, cliente, status e referência, sem payload sensível.
12. **Escopo:** nenhum serviço de parcelamento/negociação/liquidação pode ser habilitado, conforme a [ADR-0002](0002-escopo-consultivo-divida-ativa.md).

## Critérios de verificação

- Os cinco verbos são roteados pelo mesmo adapter e pelo mesmo envelope.
- `pedidoDados.dados` é enviado como string JSON e validado por serviço.
- Serviços protegidos bloqueiam a chamada quando `OBTERPROCURACAO41` não autoriza **e
  `autorPedidoDados` difere do contribuinte**.
- `PROCURACOES / OBTERPROCURACAO41` não dispara pré-validação recursiva.
- Toda chamada inclui `Authorization`; em produção inclui também `jwt_token`. `x-cpf-usuario` é
  enviado apenas quando habilitado.
- `Declarar`/`Emitir` possuem controle de idempotência e não são repetidos em `504`.
- Erros de negócio da SERPRO são mapeados para códigos internos estáveis.
- Testes cobrem envelope, catálogo, procuração, `401`, `403`, `429`, `5xx`, `504` e respostas malformadas.

## Referências

- [API Reference — Integra Contador](../Serpro/Integra-Contador/api-reference.md)
- [Catálogo de Serviços](../Serpro/Integra-Contador/catalogo-de-servicos.md)
- [Serviços x Procurações](../Serpro/Integra-Contador/servicos-x-procuracoes.md)
- [Códigos de Retorno](../Serpro/Integra-Contador/codigos-de-retorno.md)
- [Política de Timeout](../Serpro/Integra-Contador/timeout.md)
- [Identificador Opcional de Requisições](../Serpro/Integra-Contador/identificador-opcional-de-requisicoes.md)
- [SPEC — Integra Contador e Procurações](../../SPEC.md)
- [ADR-0002 — Escopo consultivo da Dívida Ativa](0002-escopo-consultivo-divida-ativa.md)
- [ADR-0004 — Segurança, cabeçalho e assinatura XML](0004-seguranca-cabecalho-assinatura-xml.md)
- [ADR-0005 — Alternador do modo Trial SERPRO](0005-alternador-modo-trial-serpro.md)
