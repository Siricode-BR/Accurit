# ADR-0004 — Segurança, Cabeçalho e Assinatura XML

- **Status:** Aceito
- **Data:** 2026-10-20
- **Decisores:** Arquitetura Accurit
- **Escopo:** Chamadas SERPRO, identidade do operador, XMLDSig, higienização e fallback de consulta
- **Relaciona-se com:** [ADR-0002](0002-escopo-consultivo-divida-ativa.md), [ADR-0003](0003-envelope-verbos-integra-contador.md), [ADR-0005](0005-alternador-modo-trial-serpro.md)

## Contexto

As APIs SERPRO impõem requisitos que não podem ser tratados como detalhes de integração:

- o header `x-cpf-usuario` é usado operacionalmente, mas **não consta na documentação oficial**
  consultada; não há confirmação de que seja obrigatório nem de que sua ausência retorne HTTP 401;
- serviços que exigem documento assinado só aceitam XMLDSig no padrão **Enveloped Signature**, com
  `<Signature>` no namespace default e `<Transform>` (a documentação oficial não usa prefixo `ds:`);
- CPF/CNPJ e valores monetários precisam chegar ao SERPRO sem pontuação, símbolos de moeda ou separadores de milhar;
- o fallback de CPF/CNPJ para **Código de Contribuinte** é **NÃO CONFIRMADO** e não existe na
  documentação oficial; fica desabilitado por padrão;
- certificados A1, chaves privadas, tokens e CPFs são dados sensíveis e não podem vazar para logs, frontend ou repositório.

A implementação ingênua desses pontos cria riscos concretos: spoofing do CPF do operador, assinatura inválida por alteração posterior ao XML, rejeição por caracteres ocultos, erros de arredondamento monetário, fallback disparado em erro de rede e exposição de dados pessoais em logs.

## Decisão

### 1. Header operacional `x-cpf-usuario` — não confirmado oficialmente

Quando `SERPRO_OPERATOR_HEADER_ENABLED=true`, toda chamada de saída para o SERPRO inclui:

```http
x-cpf-usuario: 12345678909
```

Regras:

- O valor é o CPF do **operador autenticado**, com exatamente 11 dígitos, sem pontuação.
- O CPF é carregado de `usuarios.cpf` (banco) por requisição, com cache curto; **não** é incluído no
  JWT nem vem de campo livre do corpo, da query ou de um header enviado pelo navegador.
- O Nginx remove o `x-cpf-usuario` enviado pelo cliente e o backend **sobrescreve** qualquer valor
  residual. O header do cliente nunca é confiável.
- Se o header estiver habilitado e o contexto não tiver um CPF válido, a chamada ao SERPRO é
  bloqueada com erro interno explícito (`SERPRO_OPERATOR_CPF_MISSING`).
- Se `SERPRO_OPERATOR_HEADER_ENABLED=false`, o header não é enviado e nenhuma validação de CPF é
  feita para esse fim.
- O CPF não aparece em logs. Quando necessário para correlação, usar máscara (`***.***.***-09`) ou hash.
- O header é injetado exclusivamente pela camada `infrastructure/serpro`; handlers e frontend não o manipulam.
- O mesmo middleware injeta `Authorization` e, quando o contrato exigir, `jwt_token`,
  `autenticar_procurador_token` e `x-jwt-assertion`.
- O comportamento real do SERPRO deve ser confirmado por teste live opt-in e registrado em
  `docs/DECISOES-PENDENTES.md` (D3).
- Headers de entrada não são repassados cegamente ao SERPRO; há uma allowlist de headers de correlação e de conteúdo.

### 2. Assinatura digital XMLDSig no padrão Enveloped Signature

Quando o serviço exigir documento assinado, o XML deve seguir o subconjunto XMLDSig aceito pela SERPRO:

| Item | Requisito |
|---|---|
| Documento | XML 1.0, UTF-8, uma única declaração `<?xml version="1.0" encoding="UTF-8"?>` |
| Assinatura | `<Signature xmlns="http://www.w3.org/2000/09/xmldsig#">`; a documentação oficial **não usa prefixo `ds:`** |
| Transformação | Enveloped Signature: `http://www.w3.org/2000/09/xmldsig#enveloped-signature` |
| Canonicalização | C14N: `http://www.w3.org/TR/2001/REC-xml-c14n-20010315` |
| Método de assinatura | RSA-SHA256: `http://www.w3.org/2001/04/xmldsig-more#rsa-sha256` |
| Digest | SHA-256: `http://www.w3.org/2001/04/xmlenc#sha256` |
| Certificado | ICP-Brasil série A, e-CPF ou e-CNPJ, A1 ou A3, X.509 v3, RSA 2048 bits |
| Cadeia | EndCertOnly: apenas o `X509Certificate` do usuário final |
| Codificação | Base64 para o XML assinado e para o certificado |
| Integridade | O XML não pode ser alterado após a assinatura |

Estrutura de referência:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<termoDeAutorizacao>
  <dados>
    <!-- conteúdo do termo -->
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
        <X509Certificate><!-- certificado, sem espaços ou quebras --></X509Certificate>
      </X509Data>
    </KeyInfo>
  </Signature>
</termoDeAutorizacao>
```

Regras de implementação:

- A assinatura ocorre **exclusivamente no backend**. Certificado e chave privada nunca chegam ao frontend.
- A chave privada do certificado A1 é decifrada apenas em memória, pelo tempo mínimo necessário, e zerada após o uso (`zeroize`).
- Usar biblioteca consolidada de XMLDSig/crypto; não implementar canonicalização, digest ou RSA manualmente.
- Validar o certificado antes de assinar: cadeia até a AC Raiz Brasileira, LCR, validade, uso da chave, certificado de usuário final e série A. Certificados série S são rejeitados.
- `destinatario.numero` deve ser o CNPJ do contratante; `assinadoPor.numero` deve ser exatamente o
  número do certificado que assinou (e igual a `autorPedidoDados`); `vigencia` ≥ hoje;
  `dataAssinatura` ≤ hoje. Divergência rejeita o documento.
- Remover indentação e quebras de linha do XML final; remover espaços e quebras do conteúdo de `X509Certificate`.
- Escapar caracteres reservados (`&`, `<`, `>`, `"`, `'`) e evitar caracteres Unicode ocultos.
- Após assinar, o XML não pode ser re-serializado, reformatado ou alterado.
- Antes de enviar, validar o `DigestValue`, a assinatura e a cadeia. Falha em qualquer etapa bloqueia o envio com erro explícito.
- Para `AUTENTICAPROCURADOR / ENVIOXMLASSINADO81`, o XML assinado é codificado em Base64 e enviado em `pedidoDados.dados` como string JSON: `{"xml":"<base64>"}`.
- O retorno bem-sucedido fornece `autenticar_procurador_token` e `data_hora_expiracao`; o token é armazenado cifrado e usado no header das requisições em nome do procurador.

### 3. Higienização de CPF, CNPJ e valores monetários

A normalização ocorre na borda (DTO → domínio) e antes da serialização do envelope.

| Entrada | Saída canônica | Regra |
|---|---|---|
| `123.456.789-09` | `12345678909` | Manter apenas dígitos; validar dígitos verificadores |
| `12.345.678/0001-90` | `12345678000190` | Manter apenas dígitos; validar dígitos verificadores |
| `R$ 1.234,56` | `1234.56` | Remover `R$`, espaços e milhar; vírgula decimal → ponto |
| `1.234,56` | `1234.56` | Formato pt-BR |
| `1234.56` | `1234.56` | Já normalizado |

Regras:

- CPF/CNPJ são persistidos e transmitidos somente com dígitos; a formatação é apenas de apresentação.
- Validar tamanho e dígitos verificadores; entradas inválidas são rejeitadas antes de qualquer chamada externa.
- Valores monetários usam `Decimal`/`NUMERIC` ou string no transporte. É proibido usar `f32`/`f64` para cálculos financeiros.
- Não remover zeros à esquerda de identificadores; não converter identificadores para número inteiro.
- A normalização é idempotente e coberta por testes com entradas formatadas, já limpas, inválidas e com símbolos.
- O valor canônico é o que segue para o SERPRO; a UI pode exibir o formato brasileiro, mas o transporte usa o formato canônico.

### 4. Política de fallback para “Código de Contribuinte” — não confirmada

A documentação oficial **não descreve** “Código de Contribuinte” como chave alternativa. O
parâmetro do path é `numeroInscricaoDevedor` e a amostra oficial usa CPF/CNPJ. O fallback:

1. Fica **desabilitado por padrão** (`DIVIDA_ATIVA_FALLBACK_CODIGO_CONTRIBUINTE=false`).
2. Quando habilitado, depende da porta `ResolvedorChaveAlternativa`, que retorna
   `Option<ChaveAlternativa>`; sem resolvedor configurado, a consulta segue apenas por CPF/CNPJ.
3. Só pode ser acionado em resposta **bem-sucedida e vazia** — nunca em `401`, `403`, `404`, `429`,
   `5xx`, timeout, resposta malformada ou falha de transporte.
4. Executa no máximo **um** fallback por consulta, sem recursão.
5. Registra a fonte em `consultas_fiscais.fonte_consulta` (`CPF_CNPJ` ou `CODIGO_CONTRIBUINTE`).
6. Não inventa, completa ou infere código; a origem do valor deve ser confiável e documentada.
7. Permanece dentro do escopo consultivo da [ADR-0002](0002-escopo-consultivo-divida-ativa.md).

O teste live deve determinar se “sem pendências” é `200` com corpo vazio ou `404`; até lá, o
comportamento fica registrado em `docs/DECISOES-PENDENTES.md` (D2). Se a primeira consulta falhar
por erro de integração, o fallback não mascara o erro.

### 5. Defesa em profundidade

- TLS obrigatório; mTLS com e-CNPJ quando exigido pelo contrato SERPRO de produção.
- Dívida Ativa: OAuth2 Client Credentials com cache de token de até 1 hora e renovação controlada em HTTP 401.
- Integra Contador em produção: SAPI com mTLS e-CNPJ, `Role-Type: TERCEIROS`,
  `Content-Type: application/x-www-form-urlencoded`, retornando `access_token` e `jwt_token`; as
  chamadas enviam ambos. Trial usa o Bearer público de demonstração.
- Credenciais por escritório em `credenciais_serpro` (AES-256-GCM); variáveis globais são apenas
  fallback de desenvolvimento.
- Tokens, `Consumer Secret`, certificados e chaves nunca são versionados, logados ou enviados ao frontend.
- Certificado A1 cifrado em repouso com AES-256-GCM; decifrado apenas em memória durante o uso.
- Logs usam redaction para CPF/CNPJ, tokens, headers `Authorization`, `x-cpf-usuario`, `jwt_token` e `autenticar_procurador_token`.
- Auditoria registra metadados, não payload fiscal completo.
- Rate limiting por escritório, timeouts, backoff com jitter e respeito a `Retry-After`.

## Consequências

### Positivas

- **Conformidade com o contrato SERPRO:** o header operacional, a assinatura e a higienização deixam de ser responsabilidade difusa de cada serviço.
- **Integridade e não repúdio:** o XML assinado e validado reduz risco de adulteração e de rejeição pelo SERPRO.
- **Qualidade de dados:** CPF/CNPJ e valores canônicos evitam rejeições e arredondamentos indevidos.
- **Rastreabilidade do fallback:** a fonte da consulta fica registrada, facilitando suporte e auditoria.
- **Redução de exposição:** CPF, tokens e certificados não transitam pelo frontend nem aparecem em logs.
- **Fail-closed:** falhas de identidade, assinatura ou autorização bloqueiam o envio em vez de gerar chamadas inválidas.

### Negativas

- **Complexidade criptográfica:** XMLDSig, canonicalização, cadeia ICP-Brasil e LCR exigem biblioteca e testes especializados.
- **Custo de performance:** assinar e validar XML adiciona latência; a decifração do A1 precisa ser eficiente e segura.
- **Gestão de certificados:** expiração, renovação, revogação e senha do A1 exigem processo operacional.
- **Rigor de validação:** dados legados ou formatos inesperados podem ser rejeitados e precisar de correção manual.
- **Fallback adiciona latência e custo:** uma segunda consulta pode ocorrer quando a primeira retorna vazia.
- **Risco de propagação incorreta do header:** qualquer camada que monte requisições fora do adapter pode enviar CPF errado ou ausente.
- **Ambiguidade de resultados vazios:** é preciso distinguir “sem pendências” de falha de integração e de resposta incompleta.

### Mitigações

- Centralizar toda saída SERPRO no adapter de infraestrutura, com testes de headers.
- Usar biblioteca consolidada e vetores de teste oficiais para XMLDSig.
- Automatizar alertas de expiração do certificado A1 e testar a cadeia em ambiente de homologação.
- Cobertura de testes para CPF/CNPJ, moeda, XML, digest, certificado inválido e certificado revogado.
- Cachear resultados de procuração e token, mas nunca o CPF do operador em logs.
- Monitorar taxa de fallback e latência adicional.
- Redigir logs na origem, não apenas no destino.

## Conformidade e Regras de Negócio

1. **Header operacional:** quando `SERPRO_OPERATOR_HEADER_ENABLED=true`, toda chamada de saída ao
   SERPRO inclui `x-cpf-usuario` com 11 dígitos do operador autenticado. Ausência ou invalidez
   bloqueia a chamada; quando desabilitado, o header não é enviado.
2. **Fonte confiável:** o CPF do header vem de `usuarios.cpf`; headers do cliente são
   ignorados/sobrescritos. O frontend nunca define o CPF usado na integração.
3. **Sem log de CPF:** o valor completo de `x-cpf-usuario` não pode aparecer em logs, traces ou mensagens de erro.
4. **Assinatura Enveloped:** documentos assinados usam `<Signature xmlns="http://www.w3.org/2000/09/xmldsig#">`
   e `<Transform>` Enveloped, C14N, RSA-SHA256, SHA-256 e certificado ICP-Brasil série A. O
   namespace default é o formato oficial; não usar prefixo `ds:`.
5. **Validação antes do envio:** digest, assinatura, validade, revogação, uso da chave e cadeia ICP-Brasil são verificados; falha bloqueia o envio.
6. **Certificado:** somente e-CPF/e-CNPJ A1 ou A3, X.509 v3, RSA 2048; série S é rejeitada. A chave privada é usada apenas em memória e zerada após o uso.
7. **Imutabilidade:** o XML não pode ser alterado após a assinatura; indentação, quebras e espaços do certificado são removidos antes da codificação Base64.
8. **Higienização:** CPF/CNPJ somente dígitos e com dígitos verificadores válidos; moeda em formato canônico decimal; proibido `f32`/`f64`.
9. **Fallback:** desabilitado por padrão; quando habilitado, somente após resposta bem-sucedida e
   vazia, no máximo uma tentativa, com `fonte_consulta` registrada; não acionar em erro.
10. **Resultado vazio:** a distinção entre `200` vazio e `404` é **NÃO CONFIRMADA** (D2). Não
    converter `404` em “sem pendências” sem validação live; registrar a fonte e o código observado.
11. **Escopo consultivo:** o fallback não cria fluxo de negociação/parcelamento; ver [ADR-0002](0002-escopo-consultivo-divida-ativa.md).
12. **Segredos:** tokens, certificados, chaves e `Consumer Secret` nunca são versionados, logados ou enviados ao frontend.
13. **Produção:** mTLS e certificado e-CNPJ válido são obrigatórios no Integra Contador (SAPI) e na
    Dívida Ativa; a produção da Dívida Ativa permanece bloqueada por `SERPRO_PROD_NOT_CONFIGURED`.
    Trial usa a configuração de homologação definida na [ADR-0005](0005-alternador-modo-trial-serpro.md).

## Critérios de verificação

- Com `SERPRO_OPERATOR_HEADER_ENABLED=true`, toda chamada ao SERPRO inclui `x-cpf-usuario`; a
  ausência é bloqueada antes do envio. Com `false`, o header não é enviado.
- Um header `x-cpf-usuario` enviado pelo cliente não altera o valor usado pelo backend.
- XML assinado contém `<Signature>`/`<Transform>` Enveloped no namespace default e passa na
  validação de digest e cadeia ICP-Brasil.
- Certificado expirado, revogado, série S ou incompatível com `assinadoPor` é rejeitado.
- CPF/CNPJ formatado, com letras ou dígitos verificadores inválidos é rejeitado.
- `R$ 1.234,56` é transmitido como `1234.56`; valores não usam ponto flutuante.
- Fallback permanece desabilitado por padrão; quando habilitado, só ocorre em resposta vazia
  bem-sucedida, no máximo uma vez, com `fonte_consulta` registrada.
- Nenhum CPF, token, certificado ou chave aparece em logs ou no bundle frontend.
- Testes cobrem header, assinatura, sanitização, fallback e redaction.

## Referências

- [Padrões Técnicos de Assinatura do XML](../Serpro/Integra-Contador/padroes-tecnicos-assinatura-xml.md)
- [Autentica Procurador](../Serpro/Integra-Contador/autentica-procurador.md)
- [Envio de XML Assinado](../Serpro/Integra-Contador/envio-xml-assinado.md)
- [Quick start — Consulta Dívida Ativa](../Serpro/Divida-Ativa/quick-start-consulta-divida-ativa.md)
- [SPEC — Segurança, XML e fallback](../../SPEC.md)
- [ADR-0002 — Escopo consultivo da Dívida Ativa](0002-escopo-consultivo-divida-ativa.md)
- [ADR-0003 — Envelope e verbos do Integra Contador](0003-envelope-verbos-integra-contador.md)
- [ADR-0005 — Alternador do modo Trial SERPRO](0005-alternador-modo-trial-serpro.md)
