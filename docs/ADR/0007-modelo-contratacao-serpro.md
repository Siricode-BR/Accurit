# ADR-0007 — Modelo de Contratação SERPRO e Origem das Credenciais

- **Status:** Aceito (modelo A como padrão; porta para modelo B)
- **Data:** 2026-10-20
- **Decisores:** Arquitetura Accurit
- **Escopo:** Contratação do Integra Contador, certificado e-CNPJ, credenciais por escritório e Termo de Autorização
- **Relaciona-se com:** [ADR-0003](0003-envelope-verbos-integra-contador.md), [ADR-0004](0004-seguranca-cabecalho-assinatura-xml.md), [ADR-0005](0005-alternador-modo-trial-serpro.md)

## Contexto

A Accurit é multi-tenant: cada escritório de contabilidade é um tenant e pode ter clientes PF/PJ
distintos. O Integra Contador exige que o **contratante** seja uma pessoa jurídica com certificado
e-CNPJ e que o **autor do pedido de dados** (`autorPedidoDados`) seja o titular do certificado que
assina o Termo de Autorização, quando o autor não é o próprio contratante.

Há duas formas de organizar a contratação:

| Modelo | Contratante | Autor do pedido | Certificado | Termo |
|---|---|---|---|---|
| **A — por escritório** | Cada escritório contrata com o próprio CNPJ | O próprio escritório | e-CNPJ A1 do escritório | Não é necessário quando autor = contratante |
| **B — centralizado** | Accurit ou entidade central | Escritório/procurador | e-CNPJ do contratante + procuração | Necessário quando autor ≠ contratante |

O modelo A preserva a autonomia e a responsabilidade de cada escritório, evita que a Accurit apareça
como contratante perante a SERPRO e simplifica a operação normal. O modelo B pode ser necessário para
escritórios que não possuem e-CNPJ próprio ou para uma operação centralizada.

O produto não pode assumir um único modelo de forma rígida: a origem da credencial e do certificado
precisa ser uma porta de domínio, para permitir evolução sem reescrever o adapter do Integra
Contador.

## Decisão

### 1. Modelo A como padrão

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

### 2. Porta `SerproCredentialProvider`

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
- O modelo B pode ser implementado por outra implementação da mesma porta, sem alterar o adapter.
- As credenciais globais `SERPRO_CONSUMER_KEY`/`SERPRO_CONSUMER_SECRET` são apenas fallback de
  desenvolvimento; produção usa a credencial do escritório.
- A porta nunca retorna segredos para a camada de apresentação; o adapter os injeta diretamente na
  requisição.

### 3. Persistência

- `credenciais_serpro`: `(escritorio_id, produto, ambiente)` único, com `consumer_key_cifrado` e `consumer_secret_cifrado` em AES-256-GCM, com uso de `zeroize` na memória.
- `certificados_a1`: certificado e senha cifrados (AES-256-GCM), fingerprint SHA-256, validade e titular. Descriptografia em memória com `zeroize` após o uso.
- `tokens_serpro`: `tipo_token` (`ACCESS`, `JWT`, `AUTENTICAR_PROCURADOR`) e `autor_pedido_dados`.
- `procuracoes`: inclui `ambiente` e `procurador_cpf_cnpj` na chave única.

### 4. Modelo B (suportado pela porta, não habilitado por padrão)

No modelo B:

- o contratante é a entidade central;
- o autor do pedido é o escritório ou procurador;
- o Termo de Autorização é obrigatório quando o autor difere do contratante;
- o `autenticar_procurador_token` é obtido via `ENVIOXMLASSINADO81` e usado nas chamadas;
- a ativação exige configuração explícita por escritório e revisão jurídica.

O modelo B não é ativado por padrão e não altera o modelo A.

## Consequências

### Positivas

- **Responsabilidade clara:** cada escritório responde pela própria contratação e certificado.
- **Menos Termos:** o fluxo normal do modelo A não exige Termo de Autorização.
- **Evolução segura:** a porta permite adicionar o modelo B sem reescrever o adapter.
- **Segregação de segredos:** credenciais e certificados ficam por tenant, cifrados.
- **Auditoria:** a origem da credencial e o ambiente ficam registrados.

### Negativas

- **Dependência do e-CNPJ:** escritórios sem certificado válido não operam em produção.
- **Gestão de certificados:** expiração, renovação e revogação exigem processo operacional.
- **Complexidade do modelo B:** Termo, procuração e token adicional aumentam o risco operacional.
- **Configuração por tenant:** o onboarding precisa validar credencial e certificado antes de ativar
  produção.

### Mitigações

- Validar a configuração de produção antes de permitir a troca de ambiente.
- Alertar sobre expiração do certificado A1.
- Cobrir a porta com testes de modelo A e um teste de contrato para o modelo B.
- Manter o modelo B desabilitado até que haja necessidade e aprovação explícita.

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
8. **Auditoria:** toda resolução de credencial registra tenant, produto, ambiente e origem, sem
   segredos.

## Critérios de verificação

- O adapter do Integra Contador resolve credencial via `SerproCredentialProvider`.
- O modelo A envia `contratante == autorPedidoDados` e não exige Termo.
- O modelo B, quando habilitado, exige Termo e `autenticar_procurador_token`.
- Credenciais e certificados de um escritório nunca são usados por outro.
- Nenhum segredo aparece em logs, respostas ou bundle frontend.
- Testes cobrem ausência de credencial, certificado expirado e tenant sem configuração.

## Referências

- [SPEC — Autenticação SERPRO](../../SPEC.md)
- [SPEC — Modelo de dados](../../SPEC.md)
- [Autentica Procurador](../Serpro/Integra-Contador/autentica-procurador.md)
- [Envio de XML Assinado](../Serpro/Integra-Contador/envio-xml-assinado.md)
- [ADR-0003 — Envelope e verbos do Integra Contador](0003-envelope-verbos-integra-contador.md)
- [ADR-0004 — Segurança, cabeçalho e assinatura XML](0004-seguranca-cabecalho-assinatura-xml.md)
- [ADR-0005 — Alternador do modo Trial SERPRO](0005-alternador-modo-trial-serpro.md)
