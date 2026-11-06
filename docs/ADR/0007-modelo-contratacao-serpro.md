# ADR-0007 — Modelo de Contratação SERPRO e Origem das Credenciais

- **Status:** Atualizado (modelo Software House como padrão para as APIs da plataforma; modelo A conservado)
- **Data:** 2026-11-06
- **Decisores:** Arquitetura Accurit
- **Escopo:** Contratação do Integra Contador e Dívida Ativa, certificado e-CNPJ, credenciais por escritório, Termo de Autorização e fluxo Software House (AUTENTICAPROCURADOR)
- **Relaciona-se com:** [ADR-0003](0003-envelope-verbos-integra-contador.md), [ADR-0004](0004-seguranca-cabecalho-assinatura-xml.md), [ADR-0005](0005-alternador-modo-trial-serpro.md)

## Contexto

A Accurit é multi-tenant: cada escritório de contabilidade é um tenant e pode ter clientes PF/PJ
distintos. Inicialmente, o projeto adotou o **Modelo A** (por escritório), onde cada escritório
contrata com o próprio CNPJ e o `autorPedidoDados` é o próprio escritório. No entanto, para
suportar o consumo das APIs do SERPRO no modelo **Software House** — em que a plataforma SaaS é a
Contratante e os escritórios são Procuradores —, o modelo foi estendido.

Há agora três formas de organizar a contratação:

| Modelo | Contratante | Autor do pedido | Certificado | Termo |
| --- | --- | --- | --- | --- |
| **A — por escritório** | Cada escritório contrata com o próprio CNPJ | O próprio escritório | e-CNPJ A1 do escritório | Não é necessário quando autor = contratante |
| **B — centralizado** | Accurit ou entidade central | Escritório/procurador | e-CNPJ do contratante + procuração | Necessário quando autor ≠ contratante |
| **C — Software House** | Plataforma Accurit (SaaS) | Escritório/Procurador | e-CNPJ A1 do escritório (Procurador) | **Obrigatório** — termo com 3 atores (Contratante, AutorPedidoDados, Contribuinte) |

O modelo A preserva a autonomia e a responsabilidade de cada escritório, evita que a Accurit apareça
como contratante perante a SERPRO e simplifica a operação normal em integrações legadas. O modelo B
pode ser necessário para escritórios que não possuem e-CNPJ próprio ou para uma operação
centralizada. O modelo C (Software House) é o padrão definido para o consumo das APIs SERPRO
Integra Contador e Dívida Ativa pela plataforma.

O produto não pode assumir um único modelo de forma rígida: a origem da credencial e do certificado
precisa ser uma porta de domínio, para permitir evolução sem reescrever o adapter do Integra
Contador.

## Decisão

### 1. Modelo A (conservado, sem novas integrações)

1. Cada escritório contrata o Integra Contador com o **próprio e-CNPJ**.
2. O `contratante` do envelope é o CNPJ do escritório.
3. O `autorPedidoDados` é o CNPJ do escritório (autor = contratante).
4. O certificado usado na SAPI e na assinatura do Termo é o **e-CNPJ A1 do escritório**.
5. Quando `autorPedidoDados == contribuinte`, não há procuração a validar.
6. Quando o escritório atua por um cliente, o `contribuinte` é o cliente e o `autorPedidoDados`
   continua sendo o escritório; a procuração e-CAC do cliente para o escritório é validada via
   `OBTERPROCURACAO41`.
7. O Termo de Autorização via `ENVIOXMLASSINADO81` **não** é necessário no fluxo normal do modelo A,
   porque o autor é o contratante. Ele passa a ser exigido se o autor do pedido for um procurador
   diferente do contratante.

O modelo A permanece suportado (a identidade do contratante no envelope é resolvida pela porta de
contexto existente), mas **novas integrações SERPRO da plataforma devem nascer no modelo C**.
Os demais fluxos do Integra Contador continuam, por enquanto, com `contratante = escritório`
(modelo A) até a migração completa do envelope para o contratante SaaS ser aprovada e configurada
(ver "Pendências e follow-ups").

### 2. Modelo C — Software House (padrão atual)

O modelo Software House é o padrão definido para o consumo das APIs SERPRO (Integra Contador e
Dívida Ativa). Nessa arquitetura:

- A **Plataforma Accurit (SaaS)** é a **Contratante** da API do SERPRO (detém o Bearer Token
  principal/credenciais globais — `SERPRO_TRIAL_BEARER_TOKEN` no trial, `credenciais_serpro` do
  escritório como isolamento de tenant quando aplicável);
- O **Escritório de Contabilidade** é o **Procurador** (faz upload do seu próprio Certificado A1
  na plataforma da Accurit);
- Os **Clientes do escritório** são os **Contribuintes** (que outorgaram procuração eletrônica via
  e-CAC para o CNPJ do escritório).

#### 2.1 Identidade da Software House (Contratante)

A identidade do Contratante é **configuração da plataforma**, nunca do tenant nem do banco:

- `SERPRO_CONTRATANTE_CNPJ` — CNPJ da empresa dona do SaaS (14 dígitos, com DV válido; validado na
  carga da configuração, fail-closed);
- `SERPRO_CONTRATANTE_NOME` — razão social usada no atributo `nome` do `<contratante>` do termo.

Sem os dois valores configurados, o fluxo AUTENTICAPROCURADOR falha fechado
(`ConfiguracaoIncompleta` → HTTP 503) **sem chamar o SERPRO**. Em desenvolvimento, `.env.example`
documenta as variáveis.

#### 2.2 Fluxo técnico do AUTENTICAPROCURADOR (`/Apoiar`)

O SERPRO exige um **Termo de Autorização** assinado para liberar o uso da API em nome do
Procurador. O fluxo implementado em `AutenticarProcuradorUseCase` é:

1. **Resolução dos dados.** O backend recebe `escritorio_id` e `cliente_id` (a partir do contexto
   autenticado do usuário e do corpo da requisição). Busca o cliente (contribuinte) — `cpf_cnpj`,
   `nome_razao_social` — e resolve o contexto (CNPJ/nome do escritório, CPF do operador e
   contribuinte) via a porta `CatalogoEscritoriosIntegra`.
2. **Certificado A1 do Procurador.** Busca o certificado A1 **ativo e válido** do escritório
   (`certificados_a1`), decifra o PFX e a senha **apenas em memória** (AES-256-GCM, com `zeroize`
   ao final). Sem A1 ativo, a operação falha fechado.
3. **Geração do Termo de Autorização (XML).** Montagem de `TermoAutorizacao` com:

   | Ator XML | Identidade | Origem |
   | --- | --- | --- |
   | `<destinatario>` (papel `contratante`) | CNPJ + razão social da plataforma SaaS | Configuração (`SERPRO_CONTRATANTE_*`) |
   | `<assinadoPor>` (papel `autor pedido de dados`) | CNPJ + razão social do escritório | Contexto (`escritorios.cnpj`, `escritorios.nome`) |

   - `dataAssinatura`: data atual (Brasília); `vigencia`: 365 dias após a assinatura (configurável no caso
     de uso); o XML é **fail-closed** se `vigencia < dataAssinatura` ou se qualquer ator for inválido;
   - os dois nomes são obrigatórios e são escapados como atributos XML;
   - `tipo` do **envelope** deriva do documento (11 dígitos → `1` PF, 14 dígitos → `2` PJ); no **XML** do
     termo é `PF`/`PJ`.
4. **Validação explícita dos atores.** Antes de gerar/assinar, o backend valida o par de atores
   (`AtoresTermoAutorizacao::validar`, delegado por `TermoAutorizacao::validar`):
   - `<destinatario>` **obrigatoriamente CNPJ** (não CPF);
   - `<assinadoPor>` pode ser CPF ou CNPJ (deve ser o número do certificado A1 que assina);
   - os dois nomes são obrigatórios;
   - qualquer violação impede a assinatura e a chamada ao `/Apoiar`.
   Não existe ator `contribuinte` no Termo — ele vive apenas no envelope JSON da API.
5. **Assinatura XMLDSig.** `AssinadorXmlImpl` assina o termo conforme o subconjunto adotado pelo
   SERPRO (NFe/eSocial), com uma C14N 2001 mínima implementada no próprio adapter (não há
   biblioteca XMLDSig consolidada no lockfile):
   - assinatura **enveloped** (`<Signature>` dentro de `<termoDeAutorizacao>`);
   - canonicalização **C14N 2001** (`REC-xml-c14n-20010315`) aplicada ao documento para o
     `DigestValue` (SHA-256) e ao `SignedInfo` antes de assinar;
   - método de assinatura **RSA-SHA256** (`xmldsig-more#rsa-sha256`, PKCS#1 v1.5);
   - `Reference URI=""` (todo o documento) com enveloped-signature + C14N como transforms;
   - `<KeyInfo>` traz **apenas o certificado do titular** (EndCertOnly), em base64;
   - namespace default `http://www.w3.org/2000/09/xmldsig#` declarado na tag `<Signature>`;
   - RSASSA-PKCS1-v1_5 via `rsa` crate sobre o PKCS#8 DER extraído do PFX `.p12` (p12-keystore,
     política `Strict`: vínculo chave↔cadeia por `localKeyId`).
   **Limitação documentada:** a C14N mínima injetada no `SignedInfo` isolado não herda o
   `xmlns` default do `<Signature>` pai; o documento final ainda carrega o `xmlns` na tag
   `<Signature>`, conforme exige o SERPRO.
   A validação completa de cadeia ICP-Brasil + LCR + uso da chave + série A é delegada ao
   SERPRO no `/Apoiar` (que rejeita com 403/400); localmente o adapter garante apenas que há
   certificado e chave extratíveis.
6. **Envio ao SERPRO.** O XML assinado é codificado em **base64** e enviado como
   `pedidoDados.dados = '{"xml": "<base64>"}'` (string JSON escapada) no envelope:

   ```json
   {
     "contratante":     { "numero": "<CNPJ SaaS>",  "tipo": 2 },
     "autorPedidoDados": { "numero": "<CNPJ Escritório>", "tipo": 2 },
     "contribuinte":    { "numero": "<CPF/CNPJ Cliente>", "tipo": 1 ou 2 },
     "pedidoDados": {
       "idSistema": "AUTENTICAPROCURADOR",
       "idServico": "ENVIOXMLASSINADO81",
       "versaoSistema": "1.0",
       "dados": "{\"xml\": \"<base64 do XML assinado>\"}"
     }
   }
   ```

   - método: **POST** no endpoint `/Apoiar` (`{url_base}/Apoiar`), ver `AutenticaProcuradorClienteImpl`;
   - header `Authorization: Bearer <SERPRO_TRIAL_BEARER_TOKEN>` (o Bearer principal é da plataforma);
   - `idSistema`/`idServico` fixos conforme contrato oficial; `versaoSistema` = `"1.0"`.
7. **Retorno e persistência do token.** Em HTTP 200, o SERPRO devolve `dados` como string JSON
   escapada com o objeto `Autentica`:

   ```json
   { "autenticar_procurador_token": "<jwt>", "data_hora_expiracao": "yyyy-MM-dd'T'HH:mm:ss" }
   ```

   - `data_hora_expiracao` é interpretada como horário de **Brasília (UTC−3)** e convertida para UTC
     (formato desconhecido → expiração conservadora em 24h);
   - o `jwt_token` é cifrado com **AES-256-GCM** (mesmo formato de `credenciais_serpro`,
     `nonce||ciphertext`) e salvo em `tokens_serpro` com `tipo_token = 'AUTENTICAR_PROCURADOR'`,
     vinculado a `(escritorio_id, cliente_id, ambiente)` — upsert pela constraint nomeada
     `uq_tokens_serpro` (`UNIQUE NULLS NOT DISTINCT`);
   - o token **nunca sai do backend** (nem logs, nem resposta, nem bundle frontend); o frontend
     recebe apenas `{ clienteId, status: "AUTORIZADA", expiraEm }`.
8. **Uso subsequente.** As chamadas seguintes à API Integra Contador do cliente usam o
   `autenticar_procurador_token` no header das requisições — a leitura/uso do token persistido na
   chamada de serviços (OBTERPROCURACAO41, etc.) é responsabilidade dos próximos estágios. Token
   válido até a meia-noite do dia seguinte (horário de Brasília).
9. **Exposição ao portal.** `POST /api/v1/serpro/integra-contador/procuracoes/autenticar`
   (autenticado, audiência `Client`) dispara a validação; o `apps/client` exibe o resultado no
   `acc-badge` da lista de clientes (`Autorizada`/`Pendente`).

#### 2.3 Estrutura do Termo de Autorização (destinatário + assinado por)

Conforme `docs/Serpro/Integra-Contador/padroes-tecnicos-assinatura-xml.md`, o Termo declara
exatamente dois atores no `<dados>` — `<destinatario>` (contratante) e `<assinadoPor>`
(autor do pedido de dados). Não existe ator `contribuinte` no Termo.

```xml
<termoDeAutorizacao>
  <dados>
    <sistema id="API Integra Contador"/>
    <termo texto="..."/>
    <avisoLegal texto="..."/>
    <finalidade texto="..."/>
    <dataAssinatura data="20261106"/>
    <vigencia data="20271106"/>
    <destinatario numero="[CNPJ SaaS]"       nome="[Nome SaaS]"      tipo="PJ" papel="contratante"/>
    <assinadoPor numero="[CNPJ Escritório]" nome="[Nome do Escritório]" tipo="PJ" papel="autor pedido de dados"/>
  </dados>
  <Signature xmlns="http://www.w3.org/2000/09/xmldsig#">…</Signature>
</termoDeAutorizacao>
```

**Regras de validação dos atores do Termo:**
- `<destinatario>` (contratante) **obrigatoriamente CNPJ** (não CPF).
- `<assinadoPor>` (procurador) pode ser CPF ou CNPJ (deve bater com o certificado A1 que assina).
- Os dois nomes são **obrigatórios** (não podem estar vazios).
- A validação é fail-closed: qualquer estrutura incompleta impede a assinatura e, portanto,
  a chamada a `/Apoiar`.

#### 2.4 Mapeamento de erros e fail-closed

| Situação | Comportamento |
| --- | --- |
| Sem `SERPRO_CONTRATANTE_CNPJ`/`_NOME` | HTTP 503 sem chamada ao SERPRO |
| Sem certificado A1 ativo do escritório | HTTP 503 sem chamada ao SERPRO |
| PFX inválido / cadeia sem vínculo localKeyId | HTTP 503 sem chamada ao SERPRO |
| XML reprovado (403 no SERPRO: certificado expirado, NI divergente, XML inválido, ICP-Brasil ausente) | `AcessoNegado` → HTTP 403 |
| Entrada inválida (400) | `ProcuracaoNegada` → HTTP 403 |
| 401 | `NaoAutorizado` → HTTP 401 |
| 429 | `LimiteExcedido` → HTTP 429 |
| 5xx/504 | `Indisponivel`/`Indeterminado` → HTTP 503/504 |
| Ambiente `producao` | `ProducaoBloqueada` — nunca habilitada sem confirmação contratual (D5) |

Toda chamada é auditada (`chamadas_operacionais` / `auditoria_eventos`) sem segredos; o verbo
`Apoiar` nunca é bilhetado.

### 3. Porta `SerproCredentialProvider`

O domínio define uma porta para resolver a credencial e o certificado, sem conhecer o modelo de
contratação:

```rust
pub struct CredencialSerpro {
    pub contratante: Identificacao,
    pub autor_pedido_dados: Identificacao,
    pub consumer_key: SecretString,
    pub consumer_secret: SecretString,
    pub certificado: Option<CertificadoA1>,
}

#[async_trait::async_trait]
pub trait SerproCredentialProvider: Send + Sync {
    async fn resolver(
        &self,
        escritorio_id: Uuid,
        produto: ProdutoSerpro,
        ambiente: AmbienteSerpro,
    ) -> Result<CredencialSerpro, ErroCredencial>;
}
```

- A implementação padrão lê `credenciais_serpro` e `certificados_a1` do escritório.
- O modelo B pode ser implementado por outra implementação da mesma porta, sem alterar o adapter do Integra Contador.
- As credenciais globais `SERPRO_CONSUMER_KEY`/`SERPRO_CONSUMER_SECRET` são apenas fallback de
  desenvolvimento; produção usa a credencial do escritório.
- A porta nunca retorna segredos para a camada de apresentação; o adapter os injeta diretamente na
  requisição.

### 4. Persistência

- `credenciais_serpro`: `(escritorio_id, produto, ambiente)` único, com `consumer_key_cifrado` e
  `consumer_secret_cifrado` em AES-256-GCM, com uso de `zeroize` na memória.
- `certificados_a1`: certificado e senha cifrados (AES-256-GCM), fingerprint SHA-256, validade e
  titular. Descriptografia em memória com `zeroize` após o uso.
- `tokens_serpro`: `tipo_token` (`ACCESS`, `JWT`, `AUTENTICAR_PROCURADOR`), `autor_pedido_dados` e
  `cliente_id`. O token do AUTENTICAPROCURADOR é **sempre por cliente**: `cliente_id` é exigido
  para `tipo_token = 'AUTENTICAR_PROCURADOR'` (CHECK `tokens_serpro_autenticar_procurador_exige_cliente`)
  e opcional para `ACCESS`/`JWT` (tokens do contratante, sem vínculo com contribuinte). Unicidade
  por `(escritorio_id, cliente_id, produto, ambiente, tipo_token, autor_pedido_dados)` —
  `UNIQUE NULLS NOT DISTINCT`, nomeada `uq_tokens_serpro` — com upsert pela mesma constraint via
  `ON CONFLICT ON CONSTRAINT`. O `jwt_token` nunca é devolvido ao chamador em claro após a gravação.
- `procuracoes`: inclui `ambiente` e `procurador_cpf_cnpj` na chave única.

### 5. Modelo B (suportado pela porta, não habilitado por padrão)

No modelo B:

- o contratante é a entidade central;
- o autor do pedido é o escritório ou procurador;
- o Termo de Autorização é obrigatório quando o autor difere do contratante;
- o `autenticar_procurador_token` é obtido via `ENVIOXMLASSINADO81` e usado nas chamadas;
- a ativação exige configuração explícita por escritório e revisão jurídica.

O modelo B não é ativado por padrão e não altera o modelo A.

### Pendências e follow-ups

- Migrar os demais fluxos do Integra Contador (envelope `contratante`/`autorPedidoDados` e
  resolução de credencial) para o modelo C de ponta a ponta — hoje o `resolver_contexto` ainda
  devolve `contratante = autorPedidoDados = escritório` (modelo A) nos outros fluxos;
- Usar o `autenticar_procurador_token` persistido nas chamadas subsequentes (header
  `autenticar_procurador_token`), hoje o port de chamada ainda envia somente o Bearer do
  contratante;
- Confirmar com a SERPRO a assinatura completa exigida (LCR/revogação ICP-Brasil fora do escopo do
  `AssinadorXmlImpl`); hoje a validação de cadeia é delegada ao SERPRO (`/Apoiar` valida).

## Consequências

### Positivas

- **Responsabilidade clara:** cada escritório responde pela própria contratação e certificado no modelo A.
- **Menos Termos:** o fluxo normal do modelo A não exige Termo de Autorização.
- **Evolução segura:** a porta permite adicionar o modelo B e o modelo C sem reescrever o adapter.
- **Segregação de segredos:** credenciais e certificados ficam por tenant, cifrados.
- **Auditoria:** a origem da credencial, o ambiente e o token ficam registrados.
- **Modelo Software House:** permite consumir APIs SERPRO com a estrutura corporativa adequada (SaaS como Contratante).
- **Isolamento do token:** o `jwt_token` por cliente fica cifrado e nunca chega ao navegador.

### Negativas

- **Dependência do e-CNPJ:** escritórios sem certificado válido não operam em produção.
- **Gestão de certificados:** expiração, renovação e revogação exigem processo operacional.
- **Complexidade do modelo B e C:** Termo, procuração e token adicional aumentam o risco operacional.
- **Configuração por tenant:** o onboarding precisa validar credencial e certificado antes de ativar produção.
- **Modelo Software House:** requer geração e assinatura de XML com estrutura de 2 atores (destinatário + assinado por), o que adiciona uma etapa técnica ao fluxo (além de depender de `SERPRO_CONTRATANTE_CNPJ`/`_NOME`).

### Mitigações

- Validar a configuração de produção antes de permitir a troca de ambiente.
- Alertar sobre expiração do certificado A1.
- Cobrir a porta com testes de modelo A e testes de contrato para os modelos B e C.
- Manter o modelo B desabilitado até que haja necessidade e aprovação explícita.
- Documentar a estrutura de 2 atores (destinatário + assinado por) do termo de autorização com testes unitários.
- Fail-closed em toda a cadeia: sem config, sem certificado, sem par de atores válido → nenhuma chamada ao SERPRO.

## Conformidade e Regras de Negócio

1. **Modelo A padrão:** contratante e `autorPedidoDados` são o CNPJ do escritório.
2. **Termo condicional:** `ENVIOXMLASSINADO81` só é exigido quando `autorPedidoDados` difere do
   contratante.
3. **Procuração condicional:** `OBTERPROCURACAO41` só é consultado quando `autorPedidoDados` difere
   do contribuinte.
4. **Sem credencial global em produção:** variáveis globais são fallback de desenvolvimento.
5. **Segredos cifrados:** credenciais, certificados e tokens usam AES-256-GCM.
6. **Sem exposição:** o frontend nunca recebe `consumer_secret`, certificado, chave privada ou
   tokens.
7. **Modelo B:** exige configuração explícita, revisão jurídica e nova decisão de produto.
8. **Modelo Software House (C):** exige geração de termo com 2 atores (destinatário = contratante SaaS,
   assinado por = Procurador escritório) e assinatura com certificado A1 do Procurador. O
   `jwt_token` obtido via `ENVIOXMLASSINADO81` é usado como Bearer Token nas chamadas subsequentes.
9. **Auditoria:** toda resolução de credencial registra tenant, produto, ambiente e origem, sem
   segredos.

## Critérios de verificação

- O adapter do Integra Contador resolve credencial via `SerproCredentialProvider`.
- O modelo A envia `contratante == autorPedidoDados` e não exige Termo.
- O modelo C monta o termo com 2 atores (destinatário = contratante, assinado por = procurador),
  valida CNPJ do contratante, assina XMLDSig e grava o token por `(escritório, cliente, ambiente)`.
- O modelo B, quando habilitado, exige Termo e `autenticar_procurador_token`.
- Credenciais e certificados de um escritório nunca são usados por outro.
- Nenhum segredo aparece em logs, respostas ou bundle frontend.
- Testes cobrem ausência de credencial, certificado expirado e tenant sem configuração.

## Referências

- [SPEC — Autenticação SERPRO](../../SPEC.md)
- [SPEC — Modelo de dados](../../SPEC.md)
- [Autentica Procurador](../Serpro/Integra-Contador/autentica-procurador.md)
- [Envio de XML Assinado](../Serpro/Integra-Contador/envio-xml-assinado.md)
- [Padrões Técnicos de Assinatura de XML](../Serpro/Integra-Contador/padroes-tecnicos-assinatura-xml.md)
- [Mensagens de Negócio — Envio de XML Assinado](../Serpro/Integra-Contador/mensagens-autentica-procurador.md)
- [Dados de Domínio — Termo de Autorização](../Serpro/Integra-Contador/dados-dominio-autentica-procurador.md)
- [ADR-0003 — Envelope e verbos do Integra Contador](0003-envelope-verbos-integra-contador.md)
- [ADR-0004 — Segurança, cabeçalho e assinatura XML](0004-seguranca-cabecalho-assinatura-xml.md)
- [ADR-0005 — Alternador do modo Trial SERPRO](0005-alternador-modo-trial-serpro.md)
