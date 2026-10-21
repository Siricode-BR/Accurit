# SPEC.md — Especificação Técnica da Plataforma Accurit

> Documento de referência técnica da plataforma **Accurit** — SaaS B2B multi-tenant de gestão contábil, compliance e consultas fiscais automatizadas via APIs SERPRO. Stack: PostgreSQL 16, backend Rust 2024/Axum com DDD, Angular 21 e Next.js.

**Versão deste documento:** 1.0 · **Última revisão:** 2026-10-20

---

## 1. Visão geral e escopo

A Accurit permite que escritórios de contabilidade administrem clientes PF/PJ, consultem pendências na Dívida Ativa da União e executem serviços fiscais autorizados no Integra Contador, sempre com isolamento por escritório e sem exposição de credenciais.

### 1.1 Identidade do produto

| Item | Definição |
|---|---|
| Nome oficial | **Accurit** |
| Descrição curta | SaaS multi-tenant de gestão contábil e consulta fiscal automatizada via APIs SERPRO |
| Público-alvo | Escritórios de contabilidade e seus clientes PJ/PF |
| Modelo | B2B, multi-tenant, isolamento por `escritorio_id` |
| Escopo fiscal | Consulta e operação autorizada — nunca representação legal ou cobrança |

### 1.2 Contextos delimitados (bounded contexts)

| Contexto | Responsabilidade | Módulo backend |
|---|---|---|
| Identidade e acesso | Login JWT, papéis, usuários | `domain/auth` |
| Tenancy | Escritórios, isolamento e resolução do tenant | `domain/tenancy` |
| Clientes | Cadastro PF/PJ, validação e filtros | `domain/clientes` |
| Certificados | Certificado A1 cifrado, validade e fingerprint | `domain/certificados` |
| Procurações | Verificação e cache de procurações e-CAC | `domain/procuracoes` |
| Dívida Ativa | Consulta por devedor e por inscrição | `domain/divida_ativa` |
| Integra Contador | Envelope, verbos e catálogo de serviços | `domain/integra_contador` |
| SERPRO | OAuth2, mTLS, rate limit e resiliência | `domain/serpro` |

### 1.3 Fora de escopo

- Parcelamento, negociação, transação, liquidação, cobrança ou emissão de DARF de Dívida Ativa.
- Custódia de dinheiro, emissão de nota fiscal própria ou substituição de sistemas contábeis.
- Uso de credenciais de trial como fonte de dados fiscais reais.
- Compartilhamento de certificados A1 entre escritórios.

---

## 2. Arquitetura

### 2.1 Backend Rust — DDD + Clean Architecture + SOLID

```text
backend/
├── Cargo.toml
├── diesel.toml
├── migrations/
│   └── <timestamp>_<nome>/{up.sql,down.sql}
└── src/
    ├── main.rs                 # bootstrap, migrations e servidor Axum
    ├── lib.rs
    ├── domain/
    │   ├── auth/
    │   ├── tenancy/
    │   ├── clientes/
    │   ├── certificados/
    │   ├── procuracoes/
    │   ├── divida_ativa/
    │   ├── integra_contador/
    │   └── serpro/
    ├── application/            # casos de uso; orquestra traits do domínio
    ├── infrastructure/
    │   ├── persistence/        # repositórios Diesel
    │   ├── serpro/             # clientes HTTP, OAuth2, mTLS, retry
    │   └── crypto/             # AES-256-GCM, zeroize
    └── presentation/
        ├── routes/             # composição dos routers Axum
        ├── handlers/           # handlers enxutos
        ├── dto/                # contratos HTTP e mapeamento
        └── middleware/         # auth, tenancy, request-id, rate limit
```

Regras de dependência:

- `domain/` contém entidades, value objects, invariantes e traits. **Não** depende de Axum, Diesel, Reqwest ou `serde` de transporte.
- `application/` depende apenas do domínio e de ports (traits). Orquestra transações e casos de uso.
- `infrastructure/` implementa os ports com Diesel, HTTP, criptografia e cache.
- `presentation/` traduz HTTP ↔ casos de uso; não contém regra de negócio.
- Erros de domínio são tipados e convertidos para o envelope HTTP estável em `presentation/`.
- `escritorio_id` é derivado exclusivamente do `RequestContext` autenticado (JWT + middleware de tenancy), nunca do corpo ou da query.

Pipeline de uma requisição autenticada:

```text
Request
  │
  ├─ request-id / tracing
  ├─ TLS (proxy) → rate limit por escritório
  ├─ autenticação JWT → RequestContext { usuario_id, escritorio_id, role }
  ├─ autorização por papel
  ├─ handler → caso de uso → repositório (filtro obrigatório por escritorio_id)
  └─ resposta JSON + cabeçalhos de correlação
```

### 2.2 Frontends

| Aplicação | Rota | Renderização | Arquitetura |
|---|---|---|---|
| Web | `/` | Next.js App Router (SSG/SSR) | SEO, conteúdo institucional e formulário de lead |
| Admin | `/admin` | Angular 21 SPA/CSR | Signals, Standalone Components, Clean Arch por feature |
| Cliente | `/app` e `/login` | Angular 21 SPA/CSR | Signals, Standalone Components, Clean Arch por feature |

```text
src/app/
├── core/                 # contratos, interceptors, guards e elementos transversais
├── features/<feature>/
│   ├── domain/           # modelos e regras da feature, quando aplicável
│   ├── application/      # casos de uso e estado
│   ├── data/             # DTOs, mappers e adaptadores HTTP
│   ├── presentation/     # páginas e componentes standalone
│   └── <feature>.routes.ts
└── shared/               # UI compartilhada
```

Convenções:

- A camada de apresentação não acessa `HttpClient` diretamente.
- Estado de tela usa `signal()`; valores derivados usam `computed()`.
- Guards por papel tratam 401/403 de forma consistente.
- O toggle de Trial é apenas apresentação; a autoridade é sempre o backend.

### 2.3 Infraestrutura e rede

| Serviço Compose | Imagem/build | Container | Porta interna |
|---|---|---|---|
| `postgres` | `postgres:16-alpine` | `accurit-postgres` | 5432 |
| `backend` | `backend/Dockerfile` | `accurit-backend` | 8080 |
| `web` | `web/Dockerfile` | `accurit-web` | 3000 |
| `admin` | `admin/Dockerfile` | `accurit-admin` | 80 |
| `client` | `client/Dockerfile` | `accurit-client` | 80 |
| `proxy` | `nginx:alpine` | `accurit-proxy` | 80/443 |

O proxy Nginx publica o site Next.js (`web`) em `/`, o admin em `/admin/`, o portal em `/app/`
(`/login` redireciona com `301` para `/app/login`), e a API em `/api/`. A API não é exposta
diretamente ao host em produção; apenas o proxy publica portas.

O app Next.js (`web`) pode ser entregue como export estático servido por `nginx:alpine` (melhor custo para SSG) ou por runtime Node quando houver SSR; em ambos os casos o domínio público e o TLS terminam no proxy.

---

## 3. Integração SERPRO — fundações

### 3.1 Autenticação SERPRO — duas estratégias distintas

A Accurit integra dois produtos com fluxos de autenticação diferentes. Eles **não** são intercambiáveis.

#### 3.1.1 Consulta Dívida Ativa — OAuth2 Client Credentials

Token endpoint do produto Dívida Ativa (trial e produção):

```http
POST https://gateway.apiserpro.serpro.gov.br/token
Authorization: Basic <base64(consumer_key:consumer_secret)>
Content-Type: application/x-www-form-urlencoded

grant_type=client_credentials
```

Regras:

- `Consumer Key` e `Consumer Secret` vêm de segredo de ambiente/secret manager por escritório quando aplicável.
- `Authorization: Basic` é montado com Base64 de `consumer_key:consumer_secret`; nunca registrar o header.
- Cachear o `access_token` com `expires_at = now + expires_in` (tipicamente 3600 s) e renovar proativamente antes do vencimento.
- Em HTTP 401, invalidar o cache e executar **no máximo uma** renovação/repetição controlada.
- Respeitar `Retry-After` em HTTP 429; aplicar backoff com jitter apenas em falhas transitórias.
- Produção exige mTLS com certificado A1 do escritório; trial usa o endpoint de homologação.
- A URL de produção da Dívida Ativa permanece **indefinida** e bloqueada por
  `SERPRO_PROD_NOT_CONFIGURED` até confirmação documental da SERPRO (D5).

```rust
#[derive(Debug, Clone, serde::Deserialize)]
pub struct OAuthTokenResponse {
    pub access_token: String,
    pub token_type: Option<String>,
    pub expires_in: u64,
    pub scope: Option<String>,
}
```

#### 3.1.2 Integra Contador — Trial e produção via SAPI

No **Trial**, o Integra Contador usa o Bearer público de demonstração
(`SERPRO_TRIAL_BEARER_TOKEN`) diretamente no header `Authorization`.

Em **produção**, o Integra Contador autentica na SAPI com mTLS e certificado e-CNPJ:

```http
POST https://autenticacao.sapi.serpro.gov.br/authenticate
Authorization: Basic <base64(consumer_key:consumer_secret)>
Role-Type: TERCEIROS
Content-Type: application/x-www-form-urlencoded

grant_type=client_credentials
```

- Requer certificado digital e-CNPJ padrão ICP-Brasil válido (`.p12`/`.pfx` + senha), o mesmo
  utilizado na contratação do produto.
- A resposta contém `access_token` e `jwt_token`, além de `expires_in`, `scope` e `token_type`.
- As chamadas de produção enviam `Authorization: Bearer <access_token>` **e** o header
  `jwt_token: <jwt_token>`.
- Quando o autor do pedido não é o contratante, o header `autenticar_procurador_token` (36 bytes,
  obtido no `ENVIOXMLASSINADO81`) é obrigatório.
- A base de produção é `https://gateway.apiserpro.serpro.gov.br/integra-contador/v1`.
- A chave pública do Trial é configuração de ambiente e nunca é usada em produção.

```rust
#[derive(Debug, Clone, serde::Deserialize)]
pub struct SapiTokenResponse {
    pub access_token: String,
    pub jwt_token: String,
    pub token_type: Option<String>,
    pub expires_in: u64,
    pub scope: Option<String>,
}
```

O modelo de contratação e a origem do e-CNPJ são definidos na
[ADR-0007](docs/ADR/0007-modelo-contratacao-serpro.md). O backend é o único responsável por
injetar `Authorization`, `jwt_token` e `autenticar_procurador_token`.

### 3.2 Header operacional `x-cpf-usuario` — não confirmado oficialmente

O header `x-cpf-usuario` **não consta na documentação oficial consultada** da Consulta Dívida
Ativa nem do Integra Contador. Ele é mantido como header operacional por compatibilidade com
integrações anteriores, sem afirmar que o SERPRO o exige ou que sua ausência retorna HTTP 401.

```http
x-cpf-usuario: 12345678909
```

- O valor é o CPF do **usuário operador autenticado**, com 11 dígitos e sem pontuação.
- O CPF vem do contexto autenticado no backend (`usuarios.cpf`), nunca de um campo livre do corpo
  da requisição, da query ou de um header enviado pelo navegador.
- O envio é controlado por `SERPRO_OPERATOR_HEADER_ENABLED` (padrão `true`).
- O comportamento real do SERPRO (exigência, formato e código de erro) deve ser confirmado por
  teste live opt-in e registrado em `docs/DECISOES-PENDENTES.md` (D3).
- O valor não deve aparecer em logs; quando necessário, mascarar (`***.***.***-09`) ou usar hash.
- O header é injetado em **um único ponto** da camada `infrastructure/serpro`; handlers e frontend
  não o manipulam.
- O contrato pode exigir headers adicionais de autenticação (`jwt_token`,
  `autenticar_procurador_token`, `x-jwt-assertion`); a camada `infrastructure/serpro` é a única
  responsável por injetá-los.

### 3.3 Cliente HTTP e resiliência

| Parâmetro | Valor de referência | Observação |
|---|---|---|
| Connect timeout | 5 s | Falha rápida evita esgotar workers |
| Request timeout | 30 s | Limite do gateway SERPRO |
| Pool de conexões | 32 por worker | Reutilização TLS |
| Retry | 2 tentativas (1 repetição) | Apenas leituras (`Consultar`, `Apoiar`, `Monitorar`) e falhas transitórias |
| Backoff | Exponencial com jitter | Respeitar `Retry-After` |
| Rate limit | Por escritório | Evita bloqueio no gateway SERPRO |

Regras de repetição:

- `Consultar`, `Apoiar` e `Monitorar` podem ser repetidos **uma vez** após atraso, somente em falha
  transitória (timeout, 5xx, 429).
- `Declarar` e `Emitir` **nunca** são repetidos automaticamente; exigem chave de idempotência,
  confirmação explícita e verificação de que a RFB não recebeu a solicitação.
- `202 Accepted` significa processamento assíncrono: aguardar o `tempoEspera` informado antes de
  nova consulta. Cada polling é uma nova requisição potencialmente bilhetada.
- `204 No Content` significa que o processamento ainda não terminou: aguardar o novo tempo
  estimado no header `eTag` antes de repetir.
- `304 Not Modified` indica que o token do Termo de Autorização ainda é válido: reutilizar o valor
  em cache; o header `eTag` tem o formato
  `autenticar_procurador_token:<token>` e `expires` informa a validade.
- Não são bilhetados: `204`, `304`, `400`, `401`, `404`, `429`, `500`, `503`, `504` e as chamadas
  a `/Apoiar` e `/Monitorar`.
- Toda chamada deve propagar `request-id`/correlação e registrar metadados redigidos.

**HTTP 504 é estado indeterminado:** o gateway pode ter concluído o processamento mesmo sem resposta. Não reenviar imediatamente; consultar o status quando disponível e reenviar apenas após confirmar que a RFB não recebeu a solicitação.

---

## 4. Módulo Dívida Ativa (API SERPRO)

### 4.1 Natureza exclusivamente consultiva

O módulo **apenas consulta e apresenta** inscrições, situações e valores retornados pela API SERPRO. Não existem rotas, casos de uso ou telas para parcelamento, negociação, adesão a transação, liquidação, cobrança ou emissão de DARF.

### 4.2 Endpoints

| Método | Endpoint (relativo à base) | Finalidade |
|---|---|---|
| `GET` | `/devedor/{cpfCnpj}` | Lista e resumo de inscrições/pendências do CPF/CNPJ |
| `GET` | `/inscricao/{numeroInscricao}` | Detalhamento de uma inscrição individual |

Base Trial: `https://gateway.apiserpro.serpro.gov.br/consulta-divida-ativa-trial/api/v1`
(os paths são concatenados a essa base; **não** adicionar outro `/v1`).

Base de produção: **indefinida**; a chamada de produção é bloqueada com
`SERPRO_PROD_NOT_CONFIGURED` até confirmação documental da SERPRO (D5). Não inferir a URL de
produção.

A autenticação segue o OAuth2 Client Credentials descrito na seção 3.1.1, com token Bearer e o
header operacional `x-cpf-usuario` conforme a seção 3.2. Não reutilizar automaticamente
configurações de outro produto SERPRO sem confirmação documental. Os paths diferenciam
maiúsculas e minúsculas.

### 4.3 DTOs de desserialização — Rust / Serde

Os nomes abaixo refletem os campos de negócio esperados. O envelope externo e a nulabilidade podem variar por versão/ambiente; os DTOs devem tolerar campos opcionais e ser ajustados aos exemplos oficiais vigentes.

```rust
use serde::Deserialize;

/// Item retornado por `/devedor/{cpfCnpj}`. A amostra oficial é um objeto plano;
/// o enum abaixo tolera também uma lista, caso o ambiente varie.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DevedorPayload {
    pub numero_inscricao: Option<String>,
    pub numero_processo: Option<String>,
    pub situacao_inscricao: Option<String>,
    pub situacao_descricao: Option<String>,
    pub nome_devedor: Option<String>,
    pub tipo_devedor: Option<String>,
    pub valor_total_consolidado_moeda: Option<String>,
    pub cpf_cnpj: Option<String>,
    pub codigo_sida: Option<String>,
    pub nome_unidade: Option<String>,
    pub codigo_comprot: Option<String>,
    pub codigo_uorg: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum DevedorResponse {
    Lista(Vec<DevedorPayload>),
    Item(DevedorPayload),
}

impl DevedorResponse {
    pub fn into_itens(self) -> Vec<DevedorPayload> {
        match self {
            Self::Lista(itens) => itens,
            Self::Item(item) => vec![item],
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InscricaoDetalheResponse {
    /// Número formatado retornado pela API (ex.: "90 6 17 000964-19").
    pub inscricao: String,
    pub processo_administrativo: Option<String>,
    pub situacao: Option<String>,
    pub data_inscricao: Option<String>,
    #[serde(rename = "numeroPFNResponsavel")]
    pub numero_pfn_responsavel: Option<String>,
    #[serde(rename = "nomePFNResponsavel")]
    pub nome_pfn_responsavel: Option<String>,
    #[serde(rename = "numeroPFNInscricao")]
    pub numero_pfn_inscricao: Option<String>,
    #[serde(rename = "nomePFNInscricao")]
    pub nome_pfn_inscricao: Option<String>,
    pub codigo_receita_principal: Option<String>,
    pub nome_receita: Option<String>,
    /// "SIM" ou "NAO" na amostra oficial. O rename explícito preserva o "SV8".
    #[serde(rename = "indicadorPrescricaoSV8")]
    pub indicador_prescricao_sv8: Option<String>,
    pub valor_total_inscrito_moeda: Option<String>,
    pub valor_total_consolidado_moeda: Option<String>,
    pub valor_remanescente_moeda: Option<String>,
    /// Presentes em alguns envelopes; opcionais para tolerar variações.
    pub nome_devedor: Option<String>,
    pub tipo_devedor: Option<String>,
    pub nome_unidade: Option<String>,
}
```

**Importante:** a amostra oficial de `/devedor/{cpfCnpj}` é um **objeto plano** com os campos
`numeroInscricao`, `numeroProcesso`, `situacaoInscricao`, `situacaoDescricao`, `nomeDevedor`,
`tipoDevedor`, `valorTotalConsolidadoMoeda`, `cpfCnpj`, `codigoSida`, `nomeUnidade`,
`codigoComprot` e `codigoUorg`. O `DevedorResponse` com `#[serde(untagged)]` aceita objeto ou
lista sem quebrar a desserialização. A amostra oficial de `/inscricao/{numeroInscricao}` usa
`inscricao`, `situacao`, `indicadorPrescricaoSV8`, `valorTotalInscritoMoeda`,
`valorTotalConsolidadoMoeda` e `valorRemanescenteMoeda`; os campos `numeroPFNResponsavel`,
`nomePFNResponsavel`, `numeroPFNInscricao` e `nomePFNInscricao` exigem rename explícito para não
virarem `numeroPfnResponsavel`/`nomePfnResponsavel`. Se a API responder com envelope externo
(`status`, `mensagens`, `dados`), criar DTOs de envelope correspondentes e mapear para o domínio.

Mapeamento recomendado para o domínio:

| Campo da API | Campo de domínio | Regra |
|---|---|---|
| `inscricao` | `numero_inscricao` | Preservar a formatação de exibição e manter uma versão somente dígitos para busca |
| `situacao` | `situacao_descricao` | Texto retornado, sem interpretação local |
| `indicadorPrescricaoSV8` | `indicador_prescricao: bool` | `SIM` → `true`; `NAO` → `false`; ausente → `None` |
| `valorTotalInscritoMoeda` | `valor_inscrito: Decimal` | Sanitizar e converter para decimal exato |
| `valorTotalConsolidadoMoeda` | `valor_consolidado: Decimal` | Sanitizar e converter para decimal exato |
| `valorRemanescenteMoeda` | `valor_remanescente: Decimal` | Sanitizar e converter para decimal exato |
| `codigoReceitaPrincipal` / `nomeReceita` | `receita_principal` | Concatenar apenas para exibição; preservar os campos originais |
| `numeroPFNResponsavel` / `nomePFNResponsavel` | `unidade_responsavel` | Preservar código e nome separadamente |

### 4.4 Fluxo de consulta e fallback

1. Autenticar o usuário e resolver `escritorio_id` no `RequestContext`.
2. Validar o formato do CPF/CNPJ e a autorização de acesso ao cliente consultado.
3. Resolver Trial/Produção conforme `modo_trial_serpro` e selecionar credencial/endpoint.
4. Normalizar a chave de busca (somente dígitos).
5. Executar `GET {base}/devedor/{cpfCnpj}` com timeout, correlação e o header operacional
   `x-cpf-usuario` quando habilitado.
6. **Fallback (NÃO CONFIRMADO):** não há documentação oficial de um “Código de Contribuinte”
   como chave alternativa. O parâmetro do path é `numeroInscricaoDevedor` e a amostra oficial
   usa CPF/CNPJ. O fallback só pode ser habilitado com
   `DIVIDA_ATIVA_FALLBACK_CODIGO_CONTRIBUINTE=true` e depende da porta
   `ResolvedorChaveAlternativa`; o teste live deve determinar se “sem pendências” é `200` com
   corpo vazio ou `404`. Ver D2 e `docs/DECISOES-PENDENTES.md`.
7. Se a chave consultada retornar vazio de forma inequívoca, responder como “sem pendências” —
   nunca inventar dados nem tratar ausência como erro.
8. Persistir metadados mínimos (tenant, cliente, ambiente, chave utilizada, status, data) e devolver situação/valores com indicação de ambiente e horário.

O fallback, quando habilitado, deve ser registrado na trilha de auditoria
(`fonte_consulta = CPF_CNPJ | CODIGO_CONTRIBUINTE`) para diagnóstico e suporte.

### 4.5 Regras de apresentação

- Valores monetários são recebidos como string e exibidos como retornados, sem `f32`/`f64`.
- A UI deve indicar claramente quando o resultado vem do ambiente Trial.
- Ações de negociação/parcelamento não existem na interface nem na API.
- Ausência de pendências, erro de integração e falha de autorização são estados distintos.

---

## 5. Integra Contador

### 5.1 Os cinco verbos operacionais

Os endpoints são `POST` dedicados sob a base URL do Integra Contador. O envelope comum é o mesmo; `idSistema`, `idServico`, `versaoSistema`, payload, headers e regras de negócio são específicos de cada serviço.

| Verbo | Endpoint | Exemplos de serviço | Responsabilidade |
|---|---|---|---|
| Apoiar | `POST /Apoiar` | `SICALC CONSULTAAPOIORECEITAS52`, `SITFIS SOLICITARPROTOCOLO91`, `AUTENTICAPROCURADOR ENVIOXMLASSINADO81` | Operações de apoio ao serviço |
| Consultar | `POST /Consultar` | `PGDASD CONSDECLARACAO13`, `PROCURACOES OBTERPROCURACAO41` | Consulta de informações autorizadas |
| Declarar | `POST /Declarar` | `PGDASD TRANSDECLARACAO11` | Transmissão de declarações quando habilitado |
| Emitir | `POST /Emitir` | `SITFIS RELATORIOSITFIS92`, `DCTFWEB GERARGUIA31`, `PGDASD GERARDAS12` | Emissão de guias/documentos autorizados |
| Monitorar | `POST /Monitorar` | `CAIXAPOSTAL INNOVAMSG63` | Acompanhamento de processamento |

> O catálogo oficial classifica `OBTERPROCURACAO41` como **Consultar**, `INNOVAMSG63` como
> **Monitorar** e `RELATORIOSITFIS92` como **Emitir** (não `Consultar`). `SOLICITARPROTOCOLO91` é
> **Apoiar**. Identificadores como `CONSDECLARACA013`/`TRANSDECLARACA011` devem ser conferidos no
> catálogo vigente, que usa `CONSDECLARACAO13`/`TRANSDECLARACAO11`.

No Trial, o `tipo` de `contratante`, `autorPedidoDados` e `contribuinte` pode ser restrito a `2` (CNPJ), enquanto `Identificacao.tipo` aceita `[1, 2]`; validar o contrato do ambiente antes de montar o envelope.

Não inferir semântica, parâmetros ou disponibilidade apenas pelo nome do verbo. Usar o catálogo oficial e os exemplos da versão habilitada para o escritório.

### 5.2 Envelope JSON padronizado

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identificacao {
    /// CPF (11 dígitos) ou CNPJ (14 dígitos), sem pontuação.
    pub numero: String,
    /// 1 = CPF; 2 = CNPJ.
    pub tipo: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PedidoDados {
    /// Ex.: PGDASD, DCTFWEB, SITFIS, SICALC.
    pub id_sistema: String,
    /// Ex.: CONSDECLARACAO13, GERARGUIA31, OBTERPROCURACAO41.
    pub id_servico: String,
    pub versao_sistema: String,
    /// JSON específico do serviço, serializado como string.
    pub dados: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegraContadorRequest {
    /// CPF/CNPJ do escritório contratante.
    pub contratante: Identificacao,
    /// Certificado A1 ou procurador autorizado.
    pub autor_pedido_dados: Identificacao,
    /// Cliente da contabilidade.
    pub contribuinte: Identificacao,
    pub pedido_dados: PedidoDados,
}
```

O campo `dados` é uma **string contendo JSON** — não substituir por objeto JSON sem verificar o contrato do serviço. A serialização de `autorPedidoDados`, `pedidoDados`, `idSistema`, `idServico` e `versaoSistema` segue camelCase.

### 5.3 Envelope de resposta

```rust
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegraContadorResponse {
    pub status: i32,
    pub mensagens: Vec<MensagemSerpro>,
    /// JSON específico do serviço, serializado como string.
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

O envelope real pode variar. `status` e `mensagens` devem ser interpretados conforme o catálogo oficial antes de qualquer decisão de negócio. Erros de negócio do SERPRO não devem ser convertidos em HTTP 500 genérico; mapear para códigos internos estáveis.

### 5.4 Catálogo e payloads

- `idSistema`, `idServico` e `versaoSistema` devem ser validados contra o catálogo/contrato
  vigente. `versaoSistema` é um atributo **por serviço**, não um valor global: `OBTERPROCURACAO41`
  usa `"1"` e `ENVIOXMLASSINADO81` usa `"1.0"`; os demais serviços devem ser habilitados somente
  após confirmação no catálogo oficial.
- O catálogo interno deve registrar por serviço, no mínimo: `id_sistema`, `id_servico`,
  `versao_sistema`, verbo, `tipos_contribuinte` aceitos, `regra_procuracao` e estado da allowlist.
- Catálogo inicial habilitado: apenas `AUTENTICAPROCURADOR / ENVIOXMLASSINADO81` (`"1.0"`,
  `Apoiar`) e `PROCURACOES / OBTERPROCURACAO41` (`"1"`, `Consultar`). Os demais serviços ficam
  `AguardandoDecisao` e são habilitados um a um, com validador e fixture próprios.
- A allowlist bloqueia **todos** os sistemas de parcelamento (`PARCSN`, `PARCSN-ESP`, `PERTSN`,
  `RELPSN`, `PARCMEI`, `PARCMEI-ESP`, `PERTMEI`, `RELPMEI`, `PARC-PAEX`, `PARC-SIPADE`), inclusive
  serviços `Consultar`; bloqueia também `SICALC` `Emitir` (`CONSOLIDARGERARDARF51`,
  `GERARDARFCODBARRA53`) e mantém `PGMEI / DIVIDAATIVA24` como `AguardandoDecisao` (D4). A tabela
  completa está na [ADR-0002](docs/ADR/0002-escopo-consultivo-divida-ativa.md).
- Payloads de `Declarar`/`Emitir` exigem autorização explícita, validação de procuração, idempotência e auditoria.
- `403 / AcessoNegado-ICGERENCIADOR-022` é o backstop de autorização: invalidar o cache de
  procuração e negar a operação; não tentar contornar a checagem.
- O payload nunca deve ser logado integralmente; persistir somente metadados e uma versão redigida.
- Operações de consulta podem ser cacheadas apenas quando o contrato permitir e com validade curta.
- A resposta bruta não deve ser exposta ao frontend sem mapeamento para DTOs internos.

---

## 6. Procurações e-CAC

Antes de chamar um serviço que exige representação eletrônica, consultar
`PROCURACOES / OBTERPROCURACAO41` e verificar se o contratante/procurador possui autorização
válida para o contribuinte e para a operação. A pré-validação é executada **somente quando
`autorPedidoDados` for diferente de `contribuinte`**; quando o autor é o próprio contribuinte,
não há procuração a validar.

O catálogo deve modelar `RegraProcuracao` como:

```rust
pub enum RegraProcuracao {
    NaoSeAplica,
    AnyOf(Vec<String>),
    Condicional { codigos: Vec<String>, condicao: CondicaoProcuracao },
}
```

| Sistema/serviço | Código(s) de procuração | Observação |
|---|---|---|
| PGDASD (inclui DEFIS) | `00146` | Também usado por DEFIS no catálogo oficial |
| DCTFWeb | `00103` | Inclui `TRANSDECLARACAO310` |
| SITFIS | `00002` | `RELATORIOSITFIS92` e `SOLICITARPROTOCOLO91` |
| Caixa Postal | `00006` | `MSGCONTRIBUINTE61`, `MSGDETALHAMENTO62`, `INNOVAMSG63` |
| DTE | `00050` | `CONSULTASITUACAODTE111`; não confundir com `00006` |
| Pagamentos (`PAGTOWEB`) | `00004` | `PAGAMENTOS71`, `COMPARRECADACAO72`, `CONTACONSDOCARRPG73` |
| Regime de apuração (`REGIMEAPURACAO`) | `00060` | `EFETUAROPCAOREGIME101` e consultas |
| MEI (`DASNSIMEI`) | `00229` | `CONSULTIMADECREC152`, `GERARDASEXCESSO153` |
| e-Processo (`EPROCESSO`) | `00051` | `CONSPROCPORINTER271` e demais |
| Parcelamentos (`PARCSN`) | `00076` **ou** `00188` | `AnyOf`; sistema bloqueado pela allowlist (D4) |
| Parcelamento Especial (`PARCSN-ESP`) | `00125` | Sistema bloqueado pela allowlist (D4) |
| `EVENTOSATUALIZACAO` | `n/a` / `sim¹` | `sim¹` mapeado para `00103`; validar caso a caso |
| Demais serviços | conforme catálogo | Não habilitar sem confirmação |

> `n/a` significa que o serviço não exige procuração; `AnyOf` significa que qualquer um dos
> códigos satisfaz a autorização; `Condicional` exige avaliação de contexto (por exemplo, tipo de
> contribuinte). O código `00006` pertence à Caixa Postal; o DTE usa `00050`. O catálogo deverá
> ser confirmado no PDF oficial antes de habilitar novos serviços.

Fluxo obrigatório:

1. Identificar `contratante`, `autorPedidoDados` e `contribuinte`.
2. Se `autorPedidoDados == contribuinte`, pular a pré-validação de procuração.
3. Determinar o código (ou conjunto `AnyOf`) exigido pelo serviço no catálogo.
4. Consultar `OBTERPROCURACAO41` pelo Integra Contador usando o envelope oficial, com
   `versaoSistema = "1"` e `dados` contendo
   `{"outorgante":"...","tipoOutorgante":"2","outorgado":"...","tipoOutorgado":"1"}`.
5. Interpretar contribuinte, procurador, serviço, validade e situação.
6. Negar a execução quando a autorização estiver ausente, vencida, cancelada, incompatível ou não verificável.
7. Registrar auditoria mínima (resultado, data, serviço, ambiente e referência), sem expor dados sensíveis.
8. Tratar `403 / AcessoNegado-ICGERENCIADOR-022` como backstop: invalidar o cache e negar a
   operação.

A linha local em `procuracoes` **não** é prova suficiente de autorização atual. Aplicar cache de
validade curta (`SERPRO_PROCURACAO_CACHE_TTL_SECS`, padrão 300 s) e invalidar após falha de
autorização. O cache é particionado por `(escritorio_id, ambiente, contribuinte, procurador,
código)`. A tabela `procuracoes` deve incluir `ambiente` e `procurador_cpf_cnpj` na chave única.

No modelo de contratação A (ADR-0007), `autorPedidoDados` é o próprio contratante e o Termo de
Autorização não é necessário para as operações normais. O Termo via `ENVIOXMLASSINADO81` só é
exigido quando o autor do pedido não é o contratante (modelo B ou procurador).

---

## 7. Assinatura eletrônica, XML e formatação de payload

### 7.1 Assinatura digital de XML

Quando o serviço exigir payload assinado, o XML deve conter `<Signature>` com transformação
**Enveloped Signature** e `<Transform>` dentro do namespace XMLDSig (sem prefixo `ds:`). Para o
`AUTENTICAPROCURADOR / ENVIOXMLASSINADO81`, o documento é o **Termo de Autorização**:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<termoDeAutorizacao>
  <dados>
    <sistema id="API Integra Contador"/>
    <termo texto="<!-- texto fixo do Termo de Autorização -->"/>
    <avisoLegal texto="<!-- texto fixo LGPD -->"/>
    <finalidade texto="<!-- texto fixo de finalidade -->"/>
    <dataAssinatura data="20261020"/>
    <vigencia data="20271231"/>
    <destinatario numero="00000000000000" nome="RAZAO SOCIAL DO CONTRATANTE"
                  tipo="PJ" papel="contratante"/>
    <assinadoPor numero="00000000000000" nome="NOME DO AUTOR DO PEDIDO"
                 tipo="PJ" papel="autor pedido de dados"/>
  </dados>
  <Signature xmlns="http://www.w3.org/2000/09/xmldsig#">
    <SignedInfo>
      <CanonicalizationMethod Algorithm="http://www.w3.org/TR/2001/REC-xml-c14n-20010315"/>
      <SignatureMethod Algorithm="http://www.w3.org/2001/04/xmldsig-more#rsa-sha256"/>
      <Reference URI="">
        <Transforms>
          <Transform Algorithm="http://www.w3.org/2000/09/xmldsig#enveloped-signature"/>
          <Transform Algorithm="http://www.w3.org/TR/2001/REC-xml-c14n-20010315"/>
        </Transforms>
        <DigestMethod Algorithm="http://www.w3.org/2001/04/xmlenc#sha256"/>
        <DigestValue><!-- digest calculado --></DigestValue>
      </Reference>
    </SignedInfo>
    <SignatureValue><!-- assinatura --></SignatureValue>
    <KeyInfo>
      <X509Data>
        <X509Certificate><!-- certificado A1, sem espaços/quebras --></X509Certificate>
      </X509Data>
    </KeyInfo>
  </Signature>
</termoDeAutorizacao>
```

Regras:

- XML 1.0, UTF-8, uma única declaração `<?xml ...?>`; caracteres reservados escapados.
- A documentação oficial usa o namespace XMLDSig **default**, declarado na própria tag
  `<Signature xmlns="http://www.w3.org/2000/09/xmldsig#">`, **sem prefixo `ds:`** nos elementos
  internos. Seguir esse formato.
- Raiz `termoDeAutorizacao`; um único `Reference URI=""`; duas transformações (Enveloped e C14N).
- `destinatario.numero` = CNPJ do contratante; `assinadoPor.numero` = número do certificado que
  assinou e deve ser exatamente o `autorPedidoDados`; `vigencia` ≥ hoje; `dataAssinatura` ≤ hoje.
- Certificado **ICP-Brasil série A** (A1 ou A3), e-CPF/e-CNPJ, X.509 v3, RSA 2048 bits; certificados série S são rejeitados.
- Cadeia **EndCertOnly**: incluir apenas `X509Certificate` do usuário final; a cadeia é validada contra a AC Raiz Brasileira e as LCRs.
- Assinatura RSA-SHA256, digest SHA-256, canonicalização C14N e transformação Enveloped; validar `DigestValue`, assinatura, validade, revogação e uso da chave.
- Remover indentação e quebras de linha do XML final, bem como espaços/quebras do conteúdo de `X509Certificate`, antes de codificar em Base64.
- Não inserir caracteres Unicode ocultos ou invisíveis; os textos fixos devem ser copiados exatamente de
  [`docs/Serpro/Integra-Contador/dados-dominio-autentica-procurador.md`](docs/Serpro/Integra-Contador/dados-dominio-autentica-procurador.md).
- O XML assinado é enviado em `pedidoDados.dados` como string JSON contendo o Base64 (`{"xml":"<base64>"}`), com `idSistema=AUTENTICAPROCURADOR`, `idServico=ENVIOXMLASSINADO81` e `versaoSistema="1.0"`.
- O retorno bem-sucedido fornece `autenticar_procurador_token`, válido até a meia-noite (horário de Brasília) do dia seguinte, e `data_hora_expiracao`; o token deve ser usado no header das requisições em nome do procurador.
- A chave privada do A1 é decifrada apenas em memória, pelo tempo mínimo necessário, e zerada após o uso (`zeroize`).
- A assinatura ocorre exclusivamente no backend; o certificado e a chave nunca chegam ao frontend.
- XML malformado, digest divergente ou cadeia inválida devem bloquear o envio com erro explícito.

Os textos fixos de `termo`, `avisoLegal` e `finalidade` estão transcritos em
[`docs/Serpro/Integra-Contador/dados-dominio-autentica-procurador.md`](docs/Serpro/Integra-Contador/dados-dominio-autentica-procurador.md);
qualquer divergência invalida o termo. No modelo A (ADR-0007), o Termo só é necessário quando
`autorPedidoDados` difere do contratante.

### 7.2 Higienização de CPF, CNPJ e valores monetários

Antes de serializar o envelope JSON, remover pontuação, símbolos de moeda e separadores de milhar.

| Entrada | Saída | Regra |
|---|---|---|
| `123.456.789-09` | `12345678909` | Manter apenas dígitos; validar dígitos verificadores |
| `12.345.678/0001-90` | `12345678000190` | Manter apenas dígitos; validar dígitos verificadores |
| `R$ 1.234,56` | `1234.56` | Remover `R$`, espaços e milhar; vírgula decimal → ponto |
| `1.234,56` | `1234.56` | Formato pt-BR |
| `1234.56` | `1234.56` | Formato já normalizado |

```rust
use rust_decimal::Decimal;

pub fn somente_digitos(input: &str) -> String {
    input.chars().filter(|c| c.is_ascii_digit()).collect()
}

/// Estrito para valores retornados pela API SERPRO no formato pt-BR:
/// remove todos os pontos de milhar e troca a vírgula decimal por ponto.
/// Ex.: "805,06" -> 805.06; "2.403,59" -> 2403.59; "0,00" -> 0.00.
pub fn parse_moeda_ptbr_estrito(input: &str) -> Option<Decimal> {
    let limpo = input.trim().replace("R$", "").replace(' ', "");
    if !limpo.contains(',') {
        return limpo.parse::<Decimal>().ok();
    }
    limpo.replace('.', "").replace(',', ".").parse::<Decimal>().ok()
}

/// Entrada de usuário: aceita "1.234,56", "1234,56" e "1234.56".
/// Não remove pontos cegamente quando não há vírgula.
pub fn parse_moeda_entrada_usuario(input: &str) -> Option<Decimal> {
    let limpo = input.trim().replace("R$", "").replace(' ', "");
    match (limpo.contains(','), limpo.contains('.')) {
        (true, _) => limpo.replace('.', "").replace(',', ".").parse::<Decimal>().ok(),
        (false, true) => {
            let partes: Vec<&str> = limpo.split('.').collect();
            if partes.len() == 2 && partes[1].len() <= 2 {
                limpo.parse::<Decimal>().ok()
            } else {
                limpo.replace('.', "").parse::<Decimal>().ok()
            }
        }
        (false, false) => limpo.parse::<Decimal>().ok(),
    }
}
```

Casos de teste obrigatórios: `805,06` → `805.06`; `2.403,59` → `2403.59`; `0,00` → `0.00`;
`1.234.567,89` → `1234567.89`; `R$ 1.234,56` → `1234.56`. Valores monetários usam
`Decimal`/`NUMERIC` ou string no transporte. **Nunca** usar `f32`/`f64` para cálculos financeiros.
A normalização é aplicada na borda (DTO → domínio) e o valor canônico é o que segue para o SERPRO.

### 7.3 Fallback de busca

A consulta prioriza CPF/CNPJ. O parâmetro oficial do path é `numeroInscricaoDevedor`; a amostra
oficial usa CPF/CNPJ. **Não há confirmação documental** de um “Código de Contribuinte” como chave
alternativa. O fallback só pode ser habilitado com
`DIVIDA_ATIVA_FALLBACK_CODIGO_CONTRIBUINTE=true`, atrás da porta
`ResolvedorChaveAlternativa`, e deve registrar a fonte utilizada. O teste live deve determinar se
“sem pendências” é `200` com corpo vazio ou `404`. Ver D2 em `docs/DECISOES-PENDENTES.md`.

---

## 8. Modo Trial SERPRO

O painel `/admin` oferece um toggle reativo (Angular Signals) para habilitar/desabilitar o Trial. O estado é persistido em `escritorios.modo_trial_serpro BOOLEAN NOT NULL DEFAULT TRUE`.

| Modo | Integra Contador | Dívida Ativa | Credencial |
|---|---|---|---|
| Trial (`TRUE`) | `https://gateway.apiserpro.serpro.gov.br/integra-contador-trial/v1` | `https://gateway.apiserpro.serpro.gov.br/consulta-divida-ativa-trial/api/v1` | Bearer demo via `SERPRO_TRIAL_BEARER_TOKEN` |
| Produção (`FALSE`) | `https://gateway.apiserpro.serpro.gov.br/integra-contador/v1` | **Bloqueada** (`SERPRO_PROD_NOT_CONFIGURED`) até confirmação SERPRO | Integra: SAPI (`access_token` + `jwt_token`) com mTLS e-CNPJ; Dívida Ativa: OAuth2 + mTLS + A1 |

> **Chave pública de demonstração do trial:** `<TOKEN_TRIAL_PUBLICO>`, fornecida pela SERPRO para homologação. Configure-a localmente em `SERPRO_TRIAL_BEARER_TOKEN`. Por política de segurança, o valor não é versionado no repositório, não é embutido no código-fonte/bundle frontend e nunca deve ser usado em produção.

Regras:

- O backend é a autoridade: lê a flag no início de cada requisição e registra o ambiente efetivo.
- A alteração exige papel autorizado, confirmação explícita para produção e auditoria (`quem`, `quando`, valor anterior/novo).
- Não trocar de ambiente no meio de uma requisição.
- Falha de configuração de produção resulta em erro explícito; **nunca** fazer fallback silencioso de produção para trial ou vice-versa.
- A chave de demonstração do trial é configuração de ambiente e não deve ser embutida no bundle Angular, no repositório ou em logs.

---

## 9. Modelo de dados — PostgreSQL 16

Todas as tabelas de negócio são isoladas por `escritorio_id`. Índices compostos começam por esse campo.

### 9.1 `escritorios`

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

O `slug` é o identificador público estável do escritório, normalizado na aplicação e validado pelo
`CHECK`. O CNPJ é opcional no cadastro inicial, mas, quando informado, contém exatamente 14 dígitos.
`versao` é reservado para concorrência otimista nas fases seguintes.

### 9.2 `usuarios`

```sql
CREATE TABLE usuarios (
    id UUID PRIMARY KEY,
    escritorio_id UUID NOT NULL REFERENCES escritorios(id),
    email TEXT NOT NULL,
    cpf VARCHAR(11),
    senha_hash TEXT NOT NULL,
    papel VARCHAR(32) NOT NULL CHECK (papel IN ('ADMIN', 'OPERADOR', 'CLIENTE')),
    ativo BOOLEAN NOT NULL DEFAULT TRUE,
    ultimo_login_em TIMESTAMPTZ,
    falhas_consecutivas INTEGER NOT NULL DEFAULT 0,
    bloqueado_ate TIMESTAMPTZ,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (cpf IS NULL OR cpf ~ '^[0-9]{11}$'),
    CHECK (papel = 'CLIENTE' OR cpf IS NOT NULL),
    UNIQUE (escritorio_id, email),
    UNIQUE (escritorio_id, cpf)
);
CREATE INDEX idx_usuarios_escritorio ON usuarios(escritorio_id);
```

O CPF é obrigatório para `ADMIN` e `OPERADOR` e opcional para `CLIENTE`; quando presente, é
normalizado para 11 dígitos e validado pelos dígitos verificadores na aplicação. O CPF do operador
**nunca** é incluído no JWT, em logs ou em respostas da API. Ele é carregado do banco por requisição
(com cache curto) e injetado apenas na camada `infrastructure/serpro` para o header operacional.
`falhas_consecutivas` e `bloqueado_ate` preparam o bloqueio temporário de login da Fase 2.

### 9.3 `clientes`

```sql
CREATE TABLE clientes (
    id UUID PRIMARY KEY,
    escritorio_id UUID NOT NULL REFERENCES escritorios(id),
    tipo VARCHAR(2) NOT NULL CHECK (tipo IN ('PF', 'PJ')),
    porte_regime VARCHAR(32),
    cpf_cnpj VARCHAR(14) NOT NULL,
    nome_razao_social TEXT NOT NULL,
    nome_fantasia TEXT,
    email TEXT,
    telefone TEXT,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    atualizado_em TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (
      (tipo = 'PF' AND porte_regime IS NULL) OR
      (tipo = 'PJ' AND (porte_regime IS NULL OR porte_regime IN
        ('MEI','SIMPLES_NACIONAL','LUCRO_PRESUMIDO','LUCRO_REAL')))
    ),
    UNIQUE (escritorio_id, cpf_cnpj)
);
CREATE INDEX idx_clientes_escritorio_tipo ON clientes(escritorio_id, tipo);
CREATE INDEX idx_clientes_escritorio_regime ON clientes(escritorio_id, porte_regime);
```

Validar CPF/CNPJ e normalizar para dígitos antes de persistir. A unicidade é por escritório. Em cadastro PF, `porte_regime` deve ser nulo; para PJ, o regime pode ser desconhecido/não informado.

### 9.4 `credenciais_serpro`

```sql
CREATE TABLE credenciais_serpro (
    id UUID PRIMARY KEY,
    escritorio_id UUID NOT NULL REFERENCES escritorios(id),
    produto VARCHAR(32) NOT NULL CHECK (produto IN ('INTEGRA_CONTADOR', 'DIVIDA_ATIVA')),
    ambiente VARCHAR(16) NOT NULL CHECK (ambiente IN ('TRIAL', 'PRODUCAO')),
    consumer_key_cifrado BYTEA NOT NULL,
    consumer_secret_cifrado BYTEA NOT NULL,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    atualizado_em TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (escritorio_id, produto, ambiente)
);
CREATE INDEX idx_credenciais_escritorio_produto
    ON credenciais_serpro(escritorio_id, produto, ambiente);
```

As credenciais são cifradas com AES-256-GCM usando a chave mestra do ambiente. As variáveis
globais `SERPRO_CONSUMER_KEY`/`SERPRO_CONSUMER_SECRET` são apenas fallback de desenvolvimento;
produção usa a credencial do escritório. O modelo de contratação está na
[ADR-0007](docs/ADR/0007-modelo-contratacao-serpro.md).

### 9.5 `certificados_a1`

```sql
CREATE TABLE certificados_a1 (
    id UUID PRIMARY KEY,
    escritorio_id UUID NOT NULL REFERENCES escritorios(id),
    titular_cpf_cnpj VARCHAR(14) NOT NULL,
    conteudo_cifrado BYTEA NOT NULL,
    senha_cifrada BYTEA,
    algoritmo VARCHAR(32) NOT NULL DEFAULT 'AES-256-GCM',
    fingerprint_sha256 CHAR(64) NOT NULL,
    valido_de TIMESTAMPTZ NOT NULL,
    valido_ate TIMESTAMPTZ NOT NULL,
    ativo BOOLEAN NOT NULL DEFAULT TRUE,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (escritorio_id, fingerprint_sha256)
);
CREATE INDEX idx_certificados_escritorio_validade
    ON certificados_a1(escritorio_id, valido_ate);
```

### 9.6 `tokens_serpro`

```sql
CREATE TABLE tokens_serpro (
    id UUID PRIMARY KEY,
    escritorio_id UUID NOT NULL REFERENCES escritorios(id),
    produto VARCHAR(32) NOT NULL CHECK (produto IN ('INTEGRA_CONTADOR', 'DIVIDA_ATIVA')),
    ambiente VARCHAR(16) NOT NULL CHECK (ambiente IN ('TRIAL', 'PRODUCAO')),
    tipo_token VARCHAR(32) NOT NULL CHECK (tipo_token IN ('ACCESS', 'JWT', 'AUTENTICAR_PROCURADOR')),
    autor_pedido_dados VARCHAR(14) NOT NULL DEFAULT '',
    token_cifrado BYTEA NOT NULL,
    escopo TEXT,
    expira_em TIMESTAMPTZ NOT NULL,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (escritorio_id, produto, ambiente, tipo_token, autor_pedido_dados)
);
```

Tokens são armazenados cifrados e nunca em texto puro. O cache em memória é preferencial; a tabela serve para persistência/recuperação controlada. `ACCESS` e `JWT` são os tokens da SAPI; `AUTENTICAR_PROCURADOR` é o token de 36 bytes do Termo, particionado por `autor_pedido_dados`.

### 9.7 `procuracoes`

```sql
CREATE TABLE procuracoes (
    id UUID PRIMARY KEY,
    escritorio_id UUID NOT NULL REFERENCES escritorios(id),
    ambiente VARCHAR(16) NOT NULL CHECK (ambiente IN ('TRIAL', 'PRODUCAO')),
    contribuinte_cpf_cnpj VARCHAR(14) NOT NULL,
    procurador_cpf_cnpj VARCHAR(14) NOT NULL,
    codigo_servico VARCHAR(5) NOT NULL,
    status VARCHAR(32) NOT NULL,
    valido_de TIMESTAMPTZ,
    valido_ate TIMESTAMPTZ,
    verificado_em TIMESTAMPTZ NOT NULL,
    origem VARCHAR(32) NOT NULL DEFAULT 'OBTERPROCURACAO41',
    UNIQUE (escritorio_id, ambiente, contribuinte_cpf_cnpj, procurador_cpf_cnpj, codigo_servico)
);
CREATE INDEX idx_procuracoes_escritorio_servico
    ON procuracoes(escritorio_id, ambiente, codigo_servico);
```

### 9.8 `consultas_fiscais`

```sql
CREATE TABLE consultas_fiscais (
    id UUID PRIMARY KEY,
    escritorio_id UUID NOT NULL REFERENCES escritorios(id),
    cliente_id UUID REFERENCES clientes(id),
    produto VARCHAR(32) NOT NULL,
    servico VARCHAR(64) NOT NULL,
    verbo VARCHAR(16),
    ambiente VARCHAR(16) NOT NULL CHECK (ambiente IN ('TRIAL', 'PRODUCAO')),
    chave_consulta_hash CHAR(64),
    chave_consulta_mascarada VARCHAR(20),
    fonte_consulta VARCHAR(32),
    status_code INTEGER,
    estado VARCHAR(32) NOT NULL,
    protocolo TEXT,
    request_redigido JSONB,
    response_redigido JSONB,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_consultas_escritorio_criado
    ON consultas_fiscais(escritorio_id, criado_em DESC);
CREATE INDEX idx_consultas_cliente ON consultas_fiscais(cliente_id);
```

`chave_consulta_hash` guarda SHA-256 da chave consultada; `chave_consulta_mascarada` guarda apenas
a versão exibível (ex.: `***.***.***-09`). A chave completa nunca é persistida.

### 9.9 `auditoria_eventos`

```sql
CREATE TABLE auditoria_eventos (
    id UUID PRIMARY KEY,
    escritorio_id UUID NOT NULL REFERENCES escritorios(id),
    usuario_id UUID,
    acao VARCHAR(64) NOT NULL,
    entidade VARCHAR(64),
    entidade_id UUID,
    valor_anterior JSONB,
    valor_novo JSONB,
    ip INET,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_auditoria_escritorio_criado
    ON auditoria_eventos(escritorio_id, criado_em DESC);
```

`usuario_id` e `entidade_id` são referências lógicas sem `FOREIGN KEY`, para preservar o histórico
de auditoria mesmo se o usuário ou a entidade forem removidos. O acesso continua isolado por
`escritorio_id` e o índice composto atende a consulta cronológica do tenant.

### 9.10 Retenção e minimização

- Não registrar CPF/CNPJ completo, certificados, tokens ou payloads sensíveis sem política explícita.
- `request_redigido`/`response_redigido` guardam apenas o necessário para auditoria e suporte.
- Definir prazo de retenção por tipo de consulta e rotina de expurgo.
- Backups devem seguir a mesma classificação de sensibilidade e controle de acesso.

---

## 10. API interna e filtros de clientes

### 10.1 Endpoints principais

| Método | Rota | Descrição |
|---|---|---|
| `POST` | `/api/v1/auth/login` | Autenticação e emissão de JWT |
| `GET` | `/api/v1/auth/me` | Contexto do usuário autenticado |
| `GET` | `/api/v1/clientes?filtro=...` | Listagem paginada com filtro tributário |
| `POST` | `/api/v1/clientes` | Cadastro PF/PJ |
| `GET` | `/api/v1/clientes/{id}` | Detalhe do cliente |
| `PATCH` | `/api/v1/clientes/{id}` | Atualização |
| `DELETE` | `/api/v1/clientes/{id}` | Remoção lógica ou física conforme política |
| `GET` | `/api/v1/divida-ativa/devedores/{cpfCnpj}` | Consulta devedor |
| `GET` | `/api/v1/divida-ativa/inscricoes/{numero}` | Detalhe da inscrição |
| `POST` | `/api/v1/integra-contador/{verbo}` | Execução de `apoiar`, `consultar`, `declarar`, `emitir` ou `monitorar` |
| `GET` | `/api/v1/procuracoes/{cpfCnpj}` | Situação das procurações do contribuinte |
| `GET` | `/api/v1/admin/serpro/configuracao` | Estado atual do ambiente SERPRO |
| `PATCH` | `/api/v1/admin/serpro/trial` | Alternância do modo Trial |
| `GET` | `/api/v1/health` | Health check |

Todos os endpoints de domínio validam o tenant no repositório e ignoram qualquer `escritorio_id` enviado pelo usuário.

### 10.2 Filtros obrigatórios

| Filtro UI | Query | Predicado SQL |
|---|---|---|
| Todos | `todos` | Sem filtro de tipo/regime, mantendo `escritorio_id` |
| CPF | `cpf` | `tipo = 'PF'` |
| CNPJ - MEI | `cnpj_mei` | `tipo = 'PJ' AND porte_regime = 'MEI'` |
| CNPJ - Simples Nacional | `cnpj_simples` | `tipo = 'PJ' AND porte_regime = 'SIMPLES_NACIONAL'` |
| CNPJ - Lucro Presumido | `cnpj_presumido` | `tipo = 'PJ' AND porte_regime = 'LUCRO_PRESUMIDO'` |
| CNPJ - Lucro Real | `cnpj_real` | `tipo = 'PJ' AND porte_regime = 'LUCRO_REAL'` |

### 10.3 Envelope de erro

```json
{
  "error": {
    "code": "SERPRO_AUTH_INVALID",
    "message": "Falha de autenticação junto ao SERPRO.",
    "requestId": "01J9Z8Y6Q4W3T2S1R0P9N8M7K6",
    "details": []
  }
}
```

- `code` é estável e documentado; `message` é segura para o usuário.
- Detalhes internos, stack traces, tokens e payloads nunca são retornados.
- Mapear 401/403 do SERPRO para códigos distintos (`SERPRO_UNAUTHORIZED`, `SERPRO_PROXY_DENIED`).
- Paginação usa `page`/`pageSize` com limite máximo e ordenação determinística.

### 10.4 Health check

`GET /api/v1/health` é público, não exige autenticação e é isento de rate limit. Com o banco
saudável:

```json
{"status":"ok","service":"accurit-backend","db":"ok"}
```

Com o banco indisponível, retorna `503` e mantém o mesmo envelope de erro, acrescido do estado do
serviço:

```json
{
  "error": {
    "code": "BANCO_INDISPONIVEL",
    "message": "Banco de dados indisponível.",
    "requestId": "01J9Z8Y6Q4W3T2S1R0P9N8M7K6",
    "details": []
  },
  "status": "unavailable",
  "service": "accurit-backend",
  "db": "unavailable"
}
```

- O `requestId` aceita apenas `^[0-9A-Za-z_-]{8,64}$`; caso contrário, gera UUIDv7 e sempre ecoa o valor efetivo.
- Respostas da API usam `Cache-Control: no-store` e `X-Content-Type-Options: nosniff`.
- O rate limit padrão é 30 requisições/s com burst 60, chaveado por IP (`X-Real-IP`) na Fase 1; a abstração `RateLimitKey` permite trocar a chave para `escritorio_id` na Fase 2.
- `404`, `405`, `429` e `503` usam o envelope de erro da seção 10.3; `429` inclui `Retry-After: 1`.

---

## 11. Startup, migrations e operação

As migrations são SQL versionado em `backend/migrations/` e embutidas no binário por `diesel_migrations::embed_migrations!`. O startup aguarda o PostgreSQL com retry limitado, executa as migrations pendentes sob advisory lock e só então cria o pool e inicia o Axum.

```rust
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

// startup.rs (resumo da ordem real)
let config = Config::from_env()?;
let database_url = config.database.url.expose_secret().to_owned();

let aplicadas = tokio::task::spawn_blocking(move || {
    let conexao = conectar_com_retry(&database_url, 15)?; // 1s, backoff exponencial, teto 5s
    aplicar_migrations_com_lock(conexao, MIGRATIONS)      // guard RAII dentro da função
}).await??;

let database_url = config.database.url.expose_secret().to_owned();
let pool = tokio::task::spawn_blocking(move || criar_pool(&database_url)).await??;

let app = rotas(AppState::new(pool));
let listener = TcpListener::bind(("0.0.0.0", config.port)).await?;
axum::serve(listener, app)
    .with_graceful_shutdown(encerrar_em_sigterm_ou_sigint())
    .await?;
```

Ordem obrigatória:

1. Carregar e validar a configuração (`APP_ENV`, `DATABASE_URL`, segredos, CORS, timeouts).
2. Conectar ao PostgreSQL com retry/backoff limitado; falha após 15 tentativas encerra com erro, sem `panic`.
3. Adquirir `pg_advisory_lock(727001)` em uma conexão dedicada. O guard libera o lock ao sair do escopo, inclusive em erro.
4. Executar `run_pending_migrations` em `spawn_blocking` para não bloquear o runtime Tokio.
5. Registrar a quantidade de migrations aplicadas e liberar o lock.
6. Criar o pool Diesel `r2d2`.
7. Iniciar o Axum em `0.0.0.0:PORT` com graceful shutdown para `SIGTERM`/`SIGINT`.
8. Expor `GET /api/v1/health` somente depois que migrations e pool estiverem prontos.

Observações de produção:

- O pool Diesel é criado após as migrations e é a única fonte de conexões da aplicação.
- Cada migration contém `up.sql` e `down.sql`; durante o pré-MVP, `down.sql` contém apenas `DROP`.
- O advisory lock serializa múltiplas réplicas; a estratégia de deploy deve ser compatível com migrations longas.
- Se uma migration falhar, o processo encerra com erro claro — nunca inicia parcialmente.
- `docker compose up --build` é o caminho suportado; `diesel_cli` não é necessário no host.
- O usuário de aplicação `POSTGRES_APP_USER` é não-superuser e recebe apenas os privilégios de DDL em `public` necessários às migrations. O superusuário `POSTGRES_USER` é usado somente por administração e pelos serviços do profile `tools` (`schema-tool`, `backend-test`).
- `backend/src/infrastructure/persistence/schema.rs` é gerado fora do runtime com `docker compose --profile tools run --rm schema-tool` e formatado por `rustfmt`; o build de runtime não executa `diesel_cli`.

---

## 12. Segurança, privacidade e resiliência

- TLS obrigatório; mTLS quando exigido pelo contrato SERPRO (SAPI/Integra Contador em produção).
- Certificado A1 cifrado em repouso (AES-256-GCM) e decifrado apenas em memória durante o uso.
- Segredos injetados por ambiente/secret manager; nunca versionados. Credenciais por escritório em
  `credenciais_serpro`; variáveis globais são apenas fallback de desenvolvimento.
- Token, `Consumer Secret`, certificado, `jwt_token`, `autenticar_procurador_token` e header
  `Authorization` nunca em logs.
- O CPF do operador (`usuarios.cpf`) não é incluído no JWT nem em respostas; é carregado do banco e
  usado apenas na camada de integração.
- Rate limiting por escritório, timeouts configuráveis, retries com jitter apenas para falhas transitórias e respeito a `Retry-After`.
- `Declarar` e `Emitir` não são repetidos automaticamente sem chave/estratégia de idempotência.
- Auditoria de consultas e ações administrativas sem payload fiscal excessivo.
- Erros externos convertidos em resposta estável e sem vazar detalhes internos.
- Testes de isolamento multi-tenant, permissões, procuração, alternância Trial/Produção e respostas malformadas.
- LGPD: minimização, controle de acesso, retenção definida e redação de CPF/CNPJ em logs.
- Dependências auditadas (`cargo audit`, `npm audit`) e imagens Docker com versões fixadas.

---

## 13. Critérios de aceite

- [ ] Consulta por devedor e por inscrição desserializa o contrato SERPRO do ambiente configurado e permanece exclusivamente consultiva.
- [ ] O fallback por Código de Contribuinte permanece desabilitado por padrão e só é acionado com
  `DIVIDA_ATIVA_FALLBACK_CODIGO_CONTRIBUINTE=true`, atrás de `ResolvedorChaveAlternativa` (D2).
- [ ] Os cinco verbos usam o envelope comum e parâmetros específicos do catálogo habilitado.
- [ ] O header operacional `x-cpf-usuario` é injetado apenas quando
  `SERPRO_OPERATOR_HEADER_ENABLED=true`; nenhuma documentação oficial é citada como se o exigisse.
- [ ] Serviços protegidos verificam `OBTERPROCURACAO41` antes da chamada operacional **somente quando
  `autorPedidoDados` difere do contribuinte**.
- [ ] O XML assinado usa namespace XMLDSig default, sem prefixo `ds:`, e passa na validação de digest
  e cadeia ICP-Brasil.
- [ ] `202`, `204` e `304` são tratados conforme `tempoEspera`, `eTag` e `expires`; `Declarar` e
  `Emitir` nunca são repetidos automaticamente.
- [ ] CPF/CNPJ e valores monetários são higienizados antes da serialização; a moeda pt-BR usa
  `parse_moeda_ptbr_estrito` e a entrada de usuário usa `parse_moeda_entrada_usuario`.
- [ ] O CPF do operador é obrigatório em `usuarios`, nunca vai no JWT/logs e é carregado do banco.
- [ ] A autenticação da Dívida Ativa e a da SAPI/Integra Contador são implementadas separadamente.
- [ ] O modo Trial é persistido por escritório e decidido pelo backend, sem fallback silencioso.
- [ ] A produção da Dívida Ativa falha com `SERPRO_PROD_NOT_CONFIGURED` até confirmação da SERPRO.
- [ ] Os filtros de clientes estão disponíveis na API e no Angular.
- [ ] `docker compose up --build` sobe a stack e o backend aplica migrations automaticamente.
- [ ] Nenhum segredo, token ou dado pessoal aparece no repositório, nos logs ou no bundle frontend.

---

## 14. Glossário

| Termo | Significado |
|---|---|
| Escritório | Tenant da plataforma; unidade de isolamento de dados |
| Contratante | CPF/CNPJ do escritório que contrata o serviço SERPRO |
| Autor do pedido de dados | CNPJ/CPF informado em `autorPedidoDados`, titular do certificado que assina o Termo quando aplicável |
| Contribuinte | Cliente da contabilidade sobre quem a consulta é executada |
| Procuração e-CAC | Autorização eletrônica que habilita o procurador a agir pelo contribuinte |
| Trial | Ambiente de homologação SERPRO, sem validade fiscal real |
| Enveloped Signature | Transformação XMLDSig que assina o documento incluindo a própria assinatura |
| Código de Contribuinte | Termo de versões anteriores; **não confirmado** como chave alternativa nos docs oficiais (ver D2) |
