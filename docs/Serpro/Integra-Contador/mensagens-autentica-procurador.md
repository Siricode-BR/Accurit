---
origem_pdf: Mensagens do Autentica Procurador - Integra Contador.pdf
origem_url: https://apicenter.estaleiro.serpro.gov.br/documentacao/api-integra-contador/pt/solucoes/integra-contador-gerenciador/autenticaprocurador/mensagens/
capturado_em: 2026-10-20
conversao: print-to-pdf via navegador headless (Edge) + extração do HTML oficial
aviso: Cópia de trabalho. Em divergência, vale o PDF.
---

# Mensagens de negócio — Envio de XML Assinado (Termo de Autorização)

> Documento datado de **13 de fevereiro de 2026** (rodapé da página oficial).
> Código interno do negócio no campo `codigo` das `mensagens` (texto de tamanho 5).

| Código | Mensagem / Ação | HTTP |
|---|---|---|
| `[Sucesso-AUTENTICAPROCURADOR]` | Requisição efetuada com sucesso. | 200 |
| `[EntradaIncorreta-AUTENTICAPROCURADOR-002]` | Parâmetros de entrada inválidos. Campo xml nulo, vazio ou não encontrado. Verifique os dados de entrada informados. | 400 |
| `[Erro-AUTENTICAPROCURADOR-003]` | Erro ao efetuar login. Verifique os dados de entrada informados. | 500 |
| `[Erro-AUTENTICAPROCURADOR-008]` | Erro ao validar o certificado digital. | 500 |
| `[AcessoNegado-AUTENTICAPROCURADOR-009]` | Certificado Digital deve ser e-PF, e-CPF, e-PJ ou e-CNPJ. | 403 |
| `[AcessoNegado-AUTENTICAPROCURADOR-010]` | Certificado Digital está expirado. | 403 |
| `[AcessoNegado-AUTENTICAPROCURADOR-011]` | Acesso negado, NI do procurador diferente do NI do certificado. | 403 |
| `[AcessoNegado-AUTENTICAPROCURADOR-012]` | Acesso negado, NI do contratante diferente do NI do contratante do XML assinado. | 403 |
| `[AcessoNegado-AUTENTICAPROCURADOR-013]` | Acesso negado, XML assinado inválido: {0} | 403 |
| `[Erro-AUTENTICAPROCURADOR-014]` | Erro ao obter o CNPJ do certificado presente no XML assinado. | 500 |
| `[EntradaIncorreta-AUTENTICAPROCURADOR-015]` | Layout do XML inválido: tag {0} não encontrada. Tags existentes: `dados`, `assinadoPor`, `sistema`, `destinatario`, `vigencia`, `dataAssinatura`, `finalidade`, `avisoLegal`, `termo`. | 400 |
| `[EntradaIncorreta-AUTENTICAPROCURADOR-016]` | Layout do XML inválido: atributo {0} da tag {1} inválido. Atributos: `id`, `data`, `nome`, `papel`, `numero`, `texto`. | 400 |
| `[EntradaIncorreta-AUTENTICAPROCURADOR-017]` | Layout do XML inválido: data de vigência deve ser maior ou igual que a data corrente. | 400 |
| `[EntradaIncorreta-AUTENTICAPROCURADOR-018]` | Layout do XML inválido: data de assinatura não deve ser posterior à data atual. | 400 |
| `[EntradaIncorreta-AUTENTICAPROCURADOR-019]` | Layout do XML inválido: tag raiz deve ser `termoDeAutorizacao`. | 400 |
| `[EntradaIncorreta-AUTENTICAPROCURADOR-020]` | A string fornecida do xml não é uma base64 válida. Conteúdo do xml: {0}. | 400 |
| `[AcessoNegado-AUTENTICAPROCURADOR-021]` | Certificado fornecido não possui extensões da ICP-Brasil. | 403 |
| `[AcessoNegado-AUTENTICAPROCURADOR-023]` | O certificado informado não está autorizado. | 403 |
| `[EntradaIncorreta-AUTENTICAPROCURADOR-025]` | Xml sem assinatura. A validação de estrutura deveria encontrar esse problema (Namespace: {0}). | 400 |
| `[EntradaIncorreta-AUTENTICAPROCURADOR-026]` | Elemento 'CanonicalizationMethod' inválido: '{0}'. | 400 |
| `[EntradaIncorreta-AUTENTICAPROCURADOR-027]` | Elemento 'SignatureMethod' inválido: '{0}'. | 400 |
| `[EntradaIncorreta-AUTENTICAPROCURADOR-028]` | Apenas um Elemento 'Reference' é requerido/permitido. | 400 |
| `[EntradaIncorreta-AUTENTICAPROCURADOR-029]` | A assinatura do evento deverá ser realizada sobre todo documento Xml (Atributo 'URI' deve ser vazio). | 400 |
| `[EntradaIncorreta-AUTENTICAPROCURADOR-030]` | Elemento 'DigestMethod' inválido: '{0}'. | 400 |
| `[EntradaIncorreta-AUTENTICAPROCURADOR-031]` | As seguintes transformações são exigidas: `http://www.w3.org/2000/09/xmldsig#enveloped-signature` e `http://www.w3.org/TR/2001/REC-xml-c14n-20010315`. | 400 |
| `[EntradaIncorreta-AUTENTICAPROCURADOR-032]` | Apenas um Elemento 'KeyInfo/X509Data' é requerido/permitido. | 400 |
| `[EntradaIncorreta-AUTENTICAPROCURADOR-033]` | Ações Sugeridas: verificar se houve alteração do evento após a assinatura; verificar a validade da assinatura. | 400 |
| `[Erro-AUTENTICAPROCURADOR-034]` | Padrão de assinatura não reconhecido. | 500 |
| `[AcessoNegado-AUTENTICAPROCURADOR-037]` | O certificado digital do signatário ou do solicitante da informação encontra-se revogado. Verifique a mensagem e informe um certificado válido. | 403 |
| `[AcessoNegado-AUTENTICAPROCURADOR-038]` | Ocorreu um erro ao realizar a análise do XML. Verifique se o XML não está mal formado e se não existem caracteres inválidos no documento. A primeira linha deve conter uma tag XML válida e não pode conter linha com espaço em branco ou caracteres ocultos de quebra de linha. | 403 |
| `[AcessoNegado-AUTENTICAPROCURADOR-039]` | O certificado não foi considerado válido pelas verificações internas do conteúdo X509Certificate2 (padrão ICP-Brasil). | 403 |
| `[AcessoNegado-AUTENTICAPROCURADOR-040]` | A formação da cadeia de certificação até sua raiz deve ser confiável (padrão ICP-Brasil). | 403 |
| `[AcessoNegado-AUTENTICAPROCURADOR-041]` | O certificado não pode ser do tipo autoassinado (self-signed) (padrão ICP-Brasil). | 403 |
