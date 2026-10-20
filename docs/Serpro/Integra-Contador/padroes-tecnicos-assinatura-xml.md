---
origem_pdf: Padrões Técnicos de Assinatura do XML - Integra Contador.pdf
origem_url: https://apicenter.estaleiro.serpro.gov.br/documentacao/api-integra-contador/pt/solucoes/integra-contador-gerenciador/autenticaprocurador/padroes_tecnicos_assinatura_xml/
capturado_em: 2026-10-20
conversao: print-to-pdf via navegador headless (Edge) + extração do HTML oficial
aviso: Cópia de trabalho. Em divergência, vale o PDF.
---

# Padrões Técnicos de Assinatura do XML (Termo de Autorização)

> Documento datado de **10 de abril de 2026** (rodapé da página oficial).

## Padrão de documento XML

- Especificação: recomendação W3C para **XML 1.0** (`http://www.w3.org/TR/REC-xml`).
- Codificação: **UTF-8**; todos os documentos começam com `<?xml version="1.0" encoding="UTF-8"?>`
  (uma única declaração por arquivo).
- Caracteres reservados devem usar escape: `>` → `&gt;`, `<` → `&lt;`, `&` → `&amp;`,
  `"` → `&quot;`, `'` → `&apos;`.

## Modelo do XML (Termo de Autorização) — estrutura obrigatória

A declaração do namespace da assinatura digital é feita na própria tag `<Signature>`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<termoDeAutorizacao>
  <dados>
    <sistema id="API Integra Contador" />
    <termo texto="Autorizo a empresa CONTRATANTE, identificada neste termo de autorização como DESTINATÁRIO, a executar as requisições dos serviços web disponibilizados pela API INTEGRA CONTADOR, onde terei o papel de AUTOR PEDIDO DE DADOS no corpo da mensagem enviada na requisição do serviço web. Esse termo de autorização está assinado digitalmente com o certificado digital do PROCURADOR ou OUTORGADO DO CONTRIBUINTE responsável, identificado como AUTOR DO PEDIDO DE DADOS." />
    <avisoLegal texto="O acesso a estas informações foi autorizado pelo próprio PROCURADOR ou OUTORGADO DO CONTRIBUINTE, responsável pela informação, via assinatura digital. É dever do destinatário da autorização e consumidor deste acesso observar a adoção de base legal para o tratamento dos dados recebidos conforme artigos 7º ou 11º da LGPD (Lei n.º 13.709, de 14 de agosto de 2018), aos direitos do titular dos dados (art. 9º, 17 e 18, da LGPD) e aos princípios que norteiam todos os tratamentos de dados no Brasil (art. 6º, da LGPD)." />
    <finalidade texto="A finalidade única e exclusiva desse TERMO DE AUTORIZAÇÃO, é garantir que o CONTRATANTE apresente a API INTEGRA CONTADOR esse consentimento do PROCURADOR ou OUTORGADO DO CONTRIBUINTE assinado digitalmente, para que possa realizar as requisições dos serviços web da API INTEGRA CONTADOR em nome do AUTOR PEDIDO DE DADOS (PROCURADOR ou OUTORGADO DO CONTRIBUINTE)." />
    <dataAssinatura data="20220614" />
    <vigencia data="20221231" />
    <destinatario numero="33683111000107" nome="SERVICO FEDERAL DE PROCESSAMENTO DE DADOS (SERPRO)" tipo="PJ" papel="contratante" />
    <assinadoPor numero="00000000000000" nome="NOME DO CONTRIBUINTE" tipo="PJ" papel="autor pedido de dados" />
  </dados>
  <Signature xmlns="http://www.w3.org/2000/09/xmldsig#">
    <SignedInfo>
      <CanonicalizationMethod Algorithm="http://www.w3.org/TR/2001/REC-xml-c14n-20010315" />
      <SignatureMethod Algorithm="http://www.w3.org/2001/04/xmldsig-more#rsa-sha256" />
      <Reference URI="">
        <Transforms>
          <Transform Algorithm="http://www.w3.org/2000/09/xmldsig#enveloped-signature" />
          <Transform Algorithm="http://www.w3.org/TR/2001/REC-xml-c14n-20010315" />
        </Transforms>
        <DigestMethod Algorithm="http://www.w3.org/2001/04/xmlenc#sha256" />
        <DigestValue>…</DigestValue>
      </Reference>
    </SignedInfo>
    <SignatureValue>…</SignatureValue>
    <KeyInfo>
      <X509Data>
        <X509Certificate>…</X509Certificate>
      </X509Data>
    </KeyInfo>
  </Signature>
</termoDeAutorizacao>
```

<!-- conferir no PDF: as URLs dos Algorithm no HTML oficial aparecem sem separadores ("2001REC-xml..."); o PDF prevalece -->

## Descrição das tags e atributos

| Tag | Atributos / descrição | Ocorrência | Obrigatória |
|---|---|---|---|
| `termoDeAutorizacao` | raiz do documento XML com os dados da autorização e assinado | única | sim |
| `dados` | dados de identificação do Contratante (Loja Serpro) e do Autor do Pedido de Dados | única | sim |
| `sistema` | `id="API Integra Contador"` | única | sim |
| `termo` | `texto` = texto de autorização (fixo, ver *Dados de Domínio*) | única | sim |
| `avisoLegal` | `texto` = texto LGPD (fixo, ver *Dados de Domínio*) | única | sim |
| `finalidade` | `texto` = finalidade do termo (fixo, ver *Dados de Domínio*) | única | sim |
| `dataAssinatura` | `data` = data da assinatura no formato **AAAAMMDD** | única | sim |
| `vigencia` | `data` = data de validade do termo no formato **AAAAMMDD** | única | sim |
| `destinatario` | `numero` = CNPJ (14 posições, só números, sem máscara); `nome` = razão social; `tipo` = `"PJ"`; `papel` = `"contratante"` | única | sim |
| `assinadoPor` | `numero` = CNPJ (14) ou CPF (11), só números; `nome`; `tipo` = `"PJ"` ou `"PF"`; `papel` = `"autor pedido de dados"` | única | sim |
| `Signature` | assinatura digital XMLDSig (namespace declarado na própria tag) | única | sim |

## Schema do XML

A estrutura dos XML recebidos é especificada e checada por um **Schema**; a validação é feita
por um parser — qualquer divergência da estrutura em relação ao Schema provoca erro de
validação.

## Padrão de comunicação

Web Service (Autenticação Procurador); troca de mensagens por **JSON**; o XML, após assinado,
é transformado em **string codificada em base64**.

## Padrão de certificado digital

- Emitido por AC credenciada pela **ICP-Brasil**, **série A** (a série S é de sigilo e não é
  aceita).
- Tipo **A1** (armazenado no computador) ou **A3** (smart card/token).
- Para o transmissor de solicitações: **e-CPF (e-PF)** ou **e-CNPJ (e-PJ)**.
- X.509 versão 3.

## Padrão de assinatura digital (subconjunto do XMLDSig)

- Padrão: **XML Digital Signature**, formato **Enveloped** (`http://www.w3.org/TR/xmldsig-core/`).
- Cadeia de certificação: **EndCertOnly** (incluir apenas o certificado do usuário final) —
  o arquivo assinado deve conter apenas a tag `X509Certificate` nas informações do certificado.
- Chave criptográfica: compatível com A1/A3 (**2048 bits**); RSA
  (`http://www.w3.org/2001/04/xmldsig-more#rsa-sha256`).
- Message digest: **SHA-256** (`http://www.w3.org/2001/04/xmlenc#sha256`).
- Codificação: Base64.
- Transformações exigidas:
  1. Enveloped (`http://www.w3.org/2000/09/xmldsig#enveloped-signature`);
  2. C14N (`http://www.w3.org/TR/2001/REC-xml-c14n-20010315`).

## Processo de validação da assinatura digital (mesmo do eSocial)

1. extrair a chave pública do certificado;
2. verificar o prazo de validade do certificado;
3. montar e validar a cadeia de confiança, validando também a **LCR** de cada certificado da
   cadeia;
4. validar o uso da chave (assinatura digital) — aceitar somente certificados **tipo A**
   (nunca tipo S);
5. garantir que o certificado é de usuário final (não de AC);
6. regras do **RFC 3280** para LCR e cadeia de confiança;
7. validar a integridade de todas as LCR utilizadas;
8. prazo de validade de cada LCR (data inicial e final).

## Validação do certificado digital (critérios; qualquer falha rejeita o documento)

| Critério |
|---|
| A cadeia de certificação até a raiz deve ser confiável. |
| A raiz da cadeia deve pertencer à AC Raiz Brasileira (ICP-Brasil). |
| O certificado não pode estar revogado. |
| O certificado não pode estar expirado na data da verificação. |
| Pessoa Jurídica ⇒ certificado **e-CNPJ/e-PJ**; Pessoa Física ⇒ **e-CPF/e-PF**. |
| Certificado não encontrado ⇒ rejeição. |
| Falha ao acessar a LCR ⇒ rejeição. |

## Validações aplicadas ao layout

| Critério | Efeito |
|---|---|
| CNPJ inválido | Rejeição do documento assinado |
| CPF inválido | Rejeição do documento assinado |
| Número de inscrição com tamanho ≠ 11 (CPF) ou 14 (CNPJ) | Rejeição do documento assinado |
| Número do `assinadoPor` ≠ CNPJ/CPF do certificado que assinou | Rejeição do documento assinado |

## Recomendações adicionais (do documento oficial)

- Utilizar aspas no lugar do apóstrofo.
- Antes de adicionar o elemento X509 com o certificado, remover espaços em branco e
  caracteres de quebra de linha (`\r \n CR LF`).
- Remover a indentação do XML e quebras de linha.
- Testar a conversão da string base64 para XML antes do envio.
- Após assinar, o documento deve permanecer inalterado (qualquer modificação invalida o hash).
- Ferramenta gratuita para testar a integridade da assinatura: `https://validar.iti.gov.br/`.
- Caracteres UNICODE ocultos no Termo podem provocar
  `[AcessoNegado-AUTENTICAPROCURADOR-013]`.
