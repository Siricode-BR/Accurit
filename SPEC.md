# SPEC.md — Especificação Arquitetural

> Documento de referência técnica da plataforma **Accurit**. PostgreSQL 16, backend Rust/Axum com DDD e frontends Angular organizados por features.

## 1. Objetivos e limites funcionais

A plataforma permite a escritórios contábeis administrar clientes e realizar consultas fiscais autorizadas através dos serviços SERPRO.

### 1.1 Dívida Ativa: exclusivamente consultivo

O módulo de Dívida Ativa **somente consulta e apresenta informações** retornadas pela API dedicada SERPRO. Não implementa parcelamento, negociação, adesão a transação, geração de cobrança ou emissão de DARF de Dívida Ativa.

Endpoints de consulta configurados para a integração:

- `GET /v1/devedor/{cpfCnpj}` — consulta inscrições/pendências associadas ao CPF/CNPJ.
- `GET /v1/inscricao/{numeroInscricao}` — detalha uma inscrição individual.

Base URLs, autenticação, headers e formato de envelope devem ser configurados por ambiente e validados contra a documentação oficial vigente do produto SERPRO contratado. Não reutilizar automaticamente o contrato OAuth2/mTLS do Integra Contador para a API de Dívida Ativa sem confirmação documental.

## 2. Arquitetura

### 2.1 Backend Rust — DDD + Clean Architecture + SOLID

```text
backend/
├── migrations/
│   └── <versão>_<nome>/{up.sql,down.sql}
└── src/
    ├── domain/
    │   ├── auth/
    │   ├── tenancy/
    │   ├── clientes/
    │   ├── certificados/
    │   ├── procuracoes/
    │   ├── divida_ativa/
    │   ├── integra_contador/
    │   └── serpro/
    ├── application/
    ├── infrastructure/
    │   ├── persistence/
    │   ├── serpro/
    │   └── crypto/
    └── presentation/
        ├── routes/
        ├── handlers/
        ├── dto/
        └── middleware/
```

- `domain/`: entidades, value objects, invariantes e traits; não depende de Axum, Diesel ou Reqwest.
- `application/`: casos de uso e orquestração.
- `infrastructure/`: repositórios Diesel, clientes HTTP SERPRO, cache de token e criptografia.
- `presentation/`: routers/handlers Axum, DTOs HTTP, autenticação e validação.
- `escritorio_id` é obtido do contexto autenticado (`RequestContext`), nunca confiado a um valor enviado livremente pelo cliente.

### 2.2 Frontends

Angular v18+ (Admin e Cliente) usa Clean Architecture por feature:

```text
src/app/
├── core/                 # contratos e elementos transversais
├── features/<feature>/
│   ├── domain/           # modelos/regras da feature quando aplicável
│   ├── application/      # casos de uso
│   ├── data/             # DTOs, mappers e adaptadores HTTP
│   ├── presentation/     # páginas e componentes standalone
│   └── <feature>.routes.ts
└── shared/               # UI compartilhada
```

A camada de apresentação não acessa `HttpClient` diretamente. Estado de tela usa Signals; `computed` deriva filtros e valores de visualização. Next.js é usado na landing page institucional.

## 3. Modelo de dados — PostgreSQL 16

Todas as tabelas de negócio pertencentes a um escritório devem ser isoladas por `escritorio_id`. Criar índices compostos começando por esse campo para consultas multi-tenant.

### 3.1 `escritorios`

Campos mínimos:

```sql
CREATE TABLE escritorios (
    id UUID PRIMARY KEY,
    nome TEXT NOT NULL,
    cnpj VARCHAR(14) UNIQUE,
    modo_trial_serpro BOOLEAN NOT NULL DEFAULT TRUE,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

`modo_trial_serpro = TRUE` seleciona a configuração de trial; `FALSE` seleciona produção. A seleção não pode ser usada para contornar permissões, contratos, certificados ou controles de segurança. Mudanças devem ser autorizadas, auditadas e aplicadas de forma consistente a novas requisições.

### 3.2 `clientes`

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

Validar CPF/CNPJ e normalizar para dígitos antes de persistir. A regra de unicidade é por escritório, permitindo que a mesma identificação exista em escritórios distintos quando a regra de negócio permitir. Em cadastro PF, `porte_regime` deve ser nulo. Para PJ, o regime pode ser desconhecido/não informado, portanto a coluna é anulável.

### 3.3 Tabelas complementares sugeridas pelo modelo

As migrations existentes podem incluir outras tabelas, por exemplo:

- `usuarios`: `escritorio_id`, e-mail, hash de senha, papel e estado.
- `certificados_a1`: certificado cifrado, validade e estado.
- `tokens_serpro`: tokens cifrados, expiração e escopo; não armazenar tokens em texto puro.
- `procuracoes`: registros/cache de autorização e validade, com origem e data da verificação.
- `consultas_fiscais`: tenant, cliente, tipo de serviço, request/response redigidos, protocolo/bilhete, estado, ambiente e timestamps.

Não registrar CPF/CNPJ completo, certificados, tokens, payloads sensíveis ou respostas fiscais sem política explícita de minimização, retenção e controle de acesso.

## 4. Configuração de ambiente SERPRO

O adaptador deve selecionar a configuração por escritório e por produto:

| Produto | Trial indicado para o projeto | Produção |
|---|---|---|
| Integra Contador | `https://gateway.apiserpro.serpro.gov.br/integra-contador-trial/v1` | Base URL de produção fornecida pela SERPRO para o contrato |
| Consulta Dívida Ativa | `https://gateway.apiserpro.serpro.gov.br/consulta-divida-ativa-trial/api/v1` | Base URL de produção fornecida pela SERPRO para o contrato |

A chave Bearer de demonstração fornecida para o ambiente trial deve ser lida de configuração segura (por exemplo, `SERPRO_TRIAL_BEARER_TOKEN`), nunca duplicada em código-fonte. Não presumir que uma chave de trial é uma credencial universal, permanente ou adequada a produção.

Produção do Integra Contador pode exigir OAuth2, mTLS e certificado A1 conforme o contrato habilitado. A integração de Consulta Dívida Ativa deve seguir o mecanismo de autenticação específico de sua documentação; não presumir que seja idêntico ao Integra Contador.

## 5. Consulta Dívida Ativa

### 5.1 DTOs de resposta — Rust / Serde

Os nomes abaixo refletem os campos de negócio esperados para desserialização. Como o envelope externo e a nulabilidade podem variar por versão/ambiente, os DTOs devem ser ajustados aos exemplos oficiais vigentes e tolerar campos opcionais quando apropriado.

```rust
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DevedorResponse {
    pub nome_devedor: Option<String>,
    pub tipo_devedor: Option<String>,
    pub inscricoes: Vec<InscricaoResumoDto>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InscricaoResumoDto {
    pub numero_inscricao: String,
    pub situacao_descricao: Option<String>,
    pub valor_total_consolidado_moeda: Option<String>,
    pub nome_unidade: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InscricaoDetalheResponse {
    pub numero_inscricao: String,
    pub nome_devedor: Option<String>,
    pub tipo_devedor: Option<String>,
    pub situacao_descricao: Option<String>,
    pub processo_administrativo: Option<String>,
    pub receita_principal: Option<String>,
    pub indicador_prescricao: Option<bool>,
    pub valor_inscrito: Option<String>,
    pub valor_consolidado: Option<String>,
    pub valor_remanescente: Option<String>,
    pub nome_unidade: Option<String>,
}
```

**Importante:** `inscricoes` e o envelope `DevedorResponse`/`InscricaoDetalheResponse` são modelos internos ilustrativos, não uma afirmação de que a resposta HTTP oficial é exatamente esse objeto. Se a API responder com envelope externo (status, mensagens, dados ou lista), criar DTOs de envelope correspondentes e mapear para os modelos de domínio. Valores monetários devem ser tratados com decimal exato (`Decimal`/`NUMERIC`) ou mantidos como string no DTO de transporte; não usar `f32`/`f64` para cálculos financeiros.

### 5.2 Fluxo de consulta

1. Autenticar usuário e resolver `escritorio_id`.
2. Validar formato e autorização de acesso ao cliente consultado.
3. Selecionar endpoint/credencial conforme `modo_trial_serpro`.
4. Executar `GET /v1/devedor/{cpfCnpj}` ou `GET /v1/inscricao/{numeroInscricao}` com timeout e correlação.
5. Desserializar a resposta oficial, mapear para DTO interno e persistir metadados necessários.
6. Retornar situação e valores ao usuário, indicando ambiente e horário da consulta.
7. Redigir logs e armazenar somente os dados estritamente necessários.

Não expor operações de parcelamento, negociação, transação, cobrança ou emissão de DARF de Dívida Ativa na API, UI ou casos de uso.

## 6. Integra Contador

### 6.1 Autenticação OAuth2 Client Credentials

Para o endpoint de token informado para o ambiente de produção:

`POST https://gateway.apiserpro.serpro.gov.br/token`

- Enviar `grant_type=client_credentials` no corpo conforme o formato exigido pelo gateway (normalmente `application/x-www-form-urlencoded`).
- Construir `Authorization: Basic <base64(consumer_key:consumer_secret)>`.
- Usar TLS; aplicar mTLS com o certificado A1 quando exigido pelo contrato.
- Cachear `access_token` com `expires_at` calculado a partir de `expires_in`; renovar antes de expirar.
- Em HTTP 401, invalidar o token em cache e realizar no máximo uma renovação/repetição controlada para evitar loops.
- Não registrar token, Consumer Secret, conteúdo de certificado ou header Authorization.

O endpoint e o formato exatos de autenticação devem ser confirmados para o produto/ambiente habilitado. Não reutilizar o fluxo OAuth2 de Integra Contador para Consulta Dívida Ativa sem respaldo na documentação dessa API.

DTO de token ilustrativo:

```rust
#[derive(Debug, serde::Deserialize)]
pub struct OAuthTokenResponse {
    pub access_token: String,
    pub token_type: Option<String>,
    pub expires_in: u64,
    pub scope: Option<String>,
}
```

### 6.2 Envelope comum dos cinco verbos

Os endpoints operacionais são POST dedicados: `/Apoiar`, `/Consultar`, `/Declarar`, `/Emitir` e `/Monitorar`, sob a base URL configurada para o Integra Contador.

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identificacao {
    pub numero: String, // CPF ou CNPJ, conforme tipo
    pub tipo: u8,        // 1 = CPF; 2 = CNPJ
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PedidoDados {
    pub id_sistema: String,       // ex.: PGDASD, DCTFWEB, SITFIS
    pub id_servico: String,       // ex.: CONSDECLARACA013, GERARGUIA31
    pub versao_sistema: String,
    pub dados: String,            // JSON específico serializado como string
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegraContadorRequest {
    pub contratante: Identificacao,
    #[serde(rename = "autorPedidoDados")]
    pub autor_pedido_dados: Identificacao,
    pub contribuinte: Identificacao,
    #[serde(rename = "pedidoDados")]
    pub pedido_dados: PedidoDados,
}
```

`dados` é uma string contendo o JSON específico do serviço — não substituir por objeto JSON sem verificar o contrato. O envelope comum pode ser serializado para qualquer um dos cinco verbos, mas `idSistema`, `idServico`, `versaoSistema`, payload, headers, códigos de retorno e regras de negócio são específicos de cada serviço.

| Verbo | Endpoint | Responsabilidade |
|---|---|---|
| Apoiar | `POST /Apoiar` | Operação de apoio prevista pelo catálogo do serviço |
| Consultar | `POST /Consultar` | Consulta de informações autorizadas |
| Declarar | `POST /Declarar` | Transmissão/declaração quando o serviço e o usuário estiverem habilitados |
| Emitir | `POST /Emitir` | Emissão de documento/resultado para serviços autorizados |
| Monitorar | `POST /Monitorar` | Acompanhamento de processamento quando suportado pelo serviço |

Não inferir semântica, parâmetros ou disponibilidade apenas pelo nome do verbo. Usar catálogo oficial de serviços e exemplos da versão habilitada. Para serviços que alterem estado ou transmitam declarações, exigir autorização explícita, validação de procuração, idempotência quando aplicável e trilha de auditoria.

## 7. Procurações e-CAC

Antes de chamar um serviço que exige representação eletrônica, consultar o serviço `PROCURACOES / OBTERPROCURACAO41` e verificar se o procurador/contratante tem autorização válida para o contribuinte e para a operação solicitada.

Códigos de referência informados para validação (confirmar no catálogo SERPRO/e-CAC vigente):

| Sistema/serviço | Código de procuração |
|---|---:|
| PGDASD | `00146` |
| DCTFWeb | `00103` |
| SITFIS | `00002` |
| Caixa Postal | `00006` |

Fluxo obrigatório:

1. Identificar `contratante`, `autorPedidoDados` e `contribuinte`.
2. Determinar o código de procuração requerido pelo serviço.
3. Consultar `OBTERPROCURACAO41` pelo Integra Contador usando o envelope oficial.
4. Interpretar a resposta conforme o contrato oficial: contribuinte, procurador, serviço, validade e situação.
5. Negar a execução quando a autorização estiver ausente, vencida, cancelada, incompatível ou não puder ser verificada.
6. Registrar auditoria mínima (resultado, data, serviço e referência), sem expor dados sensíveis.

Não tratar a existência de uma linha local em `procuracoes` como prova suficiente de autorização atual. Aplicar política de cache com validade curta e invalidar após falha de autorização.

## 8. Modo Trial SERPRO no Admin

A feature do Admin `/admin` oferece toggle para habilitar/desabilitar Trial. O estado é armazenado em `escritorios.modo_trial_serpro BOOLEAN NOT NULL DEFAULT TRUE`.

- `TRUE`: selecionar endpoints e credenciais de demonstração para os produtos configurados.
- `FALSE`: selecionar endpoints de produção e credenciais/certificado válidos para o escritório.
- A API deve retornar o estado atual e aceitar alterações apenas de papel autorizado.
- Persistir alteração com auditoria (`quem`, `quando`, valor anterior/novo).
- Ler o estado no backend; a decisão de ambiente não pode depender exclusivamente do estado do frontend.
- O frontend Angular usa `signal` para estado e `computed` quando houver valores derivados.
- Não trocar ambiente no meio de uma requisição. Cada chamada deve resolver e registrar o ambiente efetivo no início.
- Falha de configuração de produção deve resultar em erro explícito; nunca fazer fallback silencioso de produção para trial ou vice-versa.

A credencial Bearer de trial indicada no requisito deve ser configurada como segredo (`SERPRO_TRIAL_BEARER_TOKEN`) e não incorporada ao bundle Angular nem ao repositório.

## 9. API de clientes e filtros

Cadastro suporta PF e PJ. Filtros obrigatórios na API e na UI:

- `Todos`
- `CPF`
- `CNPJ - MEI`
- `CNPJ - Simples Nacional`
- `CNPJ - Lucro Presumido`
- `CNPJ - Lucro Real`

Mapeamento:

| Filtro UI | Predicado |
|---|---|
| Todos | Sem filtro de tipo/regime, mantendo isolamento por escritório |
| CPF | `tipo = 'PF'` |
| CNPJ - MEI | `tipo = 'PJ' AND porte_regime = 'MEI'` |
| CNPJ - Simples Nacional | `tipo = 'PJ' AND porte_regime = 'SIMPLES_NACIONAL'` |
| CNPJ - Lucro Presumido | `tipo = 'PJ' AND porte_regime = 'LUCRO_PRESUMIDO'` |
| CNPJ - Lucro Real | `tipo = 'PJ' AND porte_regime = 'LUCRO_REAL'` |

Endpoints sugeridos: `POST /api/v1/clientes`, `GET /api/v1/clientes?filtro=...`, `GET /api/v1/clientes/{id}`, `PATCH /api/v1/clientes/{id}` e `DELETE /api/v1/clientes/{id}`. Todos devem validar tenant no repositório e não confiar em `escritorio_id` enviado pelo usuário.

## 10. Migrations automáticas no startup

Migrations são SQL versionado em `backend/migrations/` e embutido no binário pelo crate `diesel_migrations`. Não é necessário Diesel CLI no host.

Exemplo ilustrativo de `backend/src/main.rs`:

```rust
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use diesel::pg::PgConnection;
use diesel::Connection;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

fn run_migrations(database_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut connection = PgConnection::establish(database_url)?;
    connection.run_pending_migrations(MIGRATIONS)
        .map_err(|e| format!("falha ao executar migrations: {e}"))?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = std::env::var("DATABASE_URL")?;
    // Em produção, aplicar retry/backoff para aguardar o healthcheck do PostgreSQL.
    run_migrations(&database_url)?;
    // Montar estado da aplicação e iniciar Axum somente após migrations concluídas.
    Ok(())
}
```

Ajustar o caminho de `embed_migrations!` ao diretório real relativo ao crate backend. A dependência `diesel_migrations` deve habilitar o recurso PostgreSQL compatível com a versão do Diesel. No startup real, usar pool de conexões se apropriado, aguardar banco saudável com retry limitado e encerrar o processo com erro claro se uma migration falhar. Em implantação com múltiplas réplicas, evitar corridas de migration usando lock/estratégia de deploy apropriada.

Convenção: cada migration contém `up.sql` e `down.sql`. Migration já aplicada em ambiente compartilhado não deve ser editada; criar uma nova.

## 11. Segurança, privacidade e resiliência

- TLS obrigatório; mTLS quando exigido pelo contrato SERPRO.
- Certificado A1 cifrado em repouso e decifrado apenas em memória durante o uso.
- Token e credenciais nunca em logs; segredos injetados por ambiente/secret manager.
- Rate limiting por escritório, timeouts configuráveis, retries com jitter apenas para falhas transitórias e respeito a `Retry-After`.
- Não repetir automaticamente operações potencialmente não idempotentes de `Declarar` ou `Emitir` sem chave/estratégia de idempotência definida.
- Auditoria de consultas e ações administrativas sem guardar payload fiscal excessivo.
- Erros externos convertidos em resposta estável e sem vazar detalhes internos.
- Testes de isolamento multi-tenant, permissões, procuração, alternância Trial/Produção e respostas malformadas.
- LGPD: minimização, controle de acesso, retenção definida e redação de CPF/CNPJ em logs.

## 12. Critérios de aceite

- Consulta por devedor e por inscrição desserializa o contrato SERPRO do ambiente configurado e permanece exclusivamente consultiva.
- Os cinco verbos usam envelope comum e parâmetros específicos do catálogo habilitado.
- Serviços protegidos verificam procuração antes da chamada operacional.
- O modo Trial é persistido por escritório e decidido pelo backend.
- Os filtros de clientes estão disponíveis na API e no Angular.
- `docker compose up --build` sobe a stack e o backend aplica migrations automaticamente.
