---
origem_pdf: Quick start - API CONSULTA DÍVIDA ATIVA.pdf
origem_url: https://apicenter.estaleiro.serpro.gov.br/documentacao/consulta-divida-at... (truncado no PDF)
capturado_em: 2026-10-17
conversao: manual-assistida
aviso: Cópia de trabalho. Em divergência, vale o PDF.
---

# Guias rápidos — Como se autenticar para consumir as APIs

As APIs disponibilizadas pela plataforma API SERPRO utilizam o protocolo OAuth2 para realizar a
autenticação e autorização de acesso das APIs contratadas. Siga os passos abaixo para se
autenticar e consumir qualquer API que se tenha contratado.

## 1. Primeiro passo — Obtenha Consumer Key e Consumer Secret

Para consumir as APIs, você deverá utilizar os dois códigos (Consumer Key e Consumer Secret)
disponibilizados na Área do Cliente. Esses códigos servem para identificar o contrato. As
credenciais de acesso devem ser obtidas a partir da *área do cliente Serpro*.

Exemplos de códigos:

- Consumer Key: `<CONSUMER_KEY>` <!-- exemplo redigido; valor no PDF -->
- Consumer Secret: `<CONSUMER_SECRET>` <!-- exemplo redigido; valor no PDF -->

O Consumer Key e Consumer Secret identificam seu usuário e seu contrato com o SERPRO. Mantenha
essas informações protegidas.

## 2. Segundo passo — Solicite o Bearer Token

Para consultar as APIs, é necessário obter um token de acesso temporário (Bearer). Esse token
possui uma (1) hora de validade e, sempre que expirado, repita este passo de requisição de um
novo token de acesso.

### Como solicitar o Token de Acesso (Bearer)

Para solicitar o token temporário é necessário realizar uma requisição HTTP POST para o endpoint
Token `https://gateway.apiserpro.serpro.gov.br/token`, informando as credenciais de acesso
(`consumerKey:consumerSecret`) no HTTP Header Authorization, no formato base64, conforme exemplo
abaixo.

```
[HEADER] Authorization: Basic base64(Consumer Key : Consumer Secret)
[HEADER] Content-Type: application/x-www-form-urlencoded
[POST] grant_type=client_credentials
```

```
curl -k -d "grant_type=client_credentials" \
--user "<CONSUMER_KEY>:<CONSUMER_SECRET>" \
https://gateway.apiserpro.serpro.gov.br/token
```

> Para utilização no parâmetro `Authorization`, é necessário concatenar os códigos Consumer Key e
> Consumer Secret, separados pelo caracter ":", e converter o resultado em BASE64. No exemplo a
> seguir, é retornada a string `ZGphUjIx[...]IzT3RlCg` (já truncada no PDF original):

Caso experiencie erros de "415 Unsupported Media Type" na chamada de solicitação do Token,
utilize o campo do Header "Content-Type" com o valor "application/x-www-form-urlencoded".

```
[HEADER] Content-type: "application/x-www-form-urlencoded"
```

## 3. Terceiro passo — Receba o Token

Como resultado do passo anterior, o endpoint informará o token de acesso a API, no campo
`access_token` da mensagem json de retorno. Este token deve ser informado nos próximos passos.
Fique atento ao tempo de renovação do Token de acesso.

Exemplo de resposta:

```
{
  "scope": "am_application_scope default",
  "token_type": "Bearer",
  "expires_in": 3295,
  "access_token": "<ACCESS_TOKEN_EXEMPLO>" <!-- exemplo redigido; valor no PDF -->
}
```

### Renovação do Token de Acesso

O token de acesso possui uma (1) hora de validade. Recomenda-se gerar um token de acesso por
hora. Utilize o token de acesso até o momento que o gateway retorne um HTTP CODE 401 após
realizar uma requisição para uma API. Para renovação do token, repita o *Segundo Passo — Como
solicitar o Token de Acesso (Bearer)*.

## 4. Quarto passo — Consumindo uma API pela interface

Após executar os passos de 1 a 3 e de posse do token de acesso, acesse as sessões *Demonstração*
ou *Já Contratou?* dentro desta documentação, e na página de *Chamada do Serviço*, procure e
clique no botão "Authorize" (na janela que aparecer, insira o token de acesso no campo value e
clique em *Authorize*; depois clique em *Close*). Após autenticado, clique sobre o serviço que
deseja consumir; na página do serviço clique em *Try it out*, preencha os campos e clique em
*Execute*.

A sessão *Responses* será complementada com um campo *Curl* (com o comando que pode ser copiado,
colado e executado na console do seu computador), o campo *Request URL* com a URL completa
executada, incluindo o contexto e parâmetros, e o *Server response*, com a resposta do consumo da
API, incluindo o código de retorno.

### 4.1 Links para as interfaces de consumo (Clientes da Plataforma de Inteligência do Serpro)

Após executar os passos 1, 2 e 3, e de posse do token de acesso, acesse as chamadas aos serviços:

- Consulta Dívida Ativa (Deprecated)
- Consulta Dívida Ativa (DF) (Deprecated)

### 4.2 Link para a interface de demonstração

Para não clientes, é possível executar as APIs trial para verificação das funcionalidades de
dados não produtivos:

- Consulta Dívida Ativa

## Documentação complementar

### Consultando dados da Dívida Ativa de uma Inscrição

No exemplo acima foram utilizados os seguintes parâmetros:

- `[HEADER] Accept: application/json` — Informamos o tipo de dados que estamos requerendo, nesse caso JSON
- `[HEADER] Authorization: Bearer <TOKEN_TRIAL_PUBLICO>` — Informamos o token de acesso recebido <!-- token redigido -->
- `[GET] https://gateway.apiserpro.serpro.gov.br/consulta-divida-ativa-trial/api/v1/inscricao/9061700096419` — Chamamos a url da API e o método desejado. No caso, a url base é `https://gateway.apiserpro.serpro.gov.br/consulta-divida-ativa-trial/api/v1`, e o método é o `/v1/inscricao/{numeroInscricao}`.

Exemplo de chamada:

```
curl -X GET "https://gateway.apiserpro.serpro.gov.br/consulta-divida-ativa-trial/api/v1/inscricao/9061700096419" \
-H "accept: application/json" \
-H "Authorization: Bearer <TOKEN_TRIAL_PUBLICO>"
```

Exemplo de resposta esperada:

```json
{
  "inscricao": "90 6 17 000964-19",
  "processoAdministrativo": "46206 099222/99-01",
  "situacao": "ATIVA NAO PRIORIZADA PARA AJUIZAMENTO",
  "dataInscricao": "16/12/2003",
  "numeroPFNResponsavel": "1000",
  "nomePFNResponsavel": "PRIMEIRA REGIAO",
  "numeroPFNInscricao": "1000",
  "nomePFNInscricao": "PRIMEIRA REGIAO",
  "numeroProcessoJudicial": "",
  "numeroProcessoJudicialNovo": "",
  "orgaoOrigem": "DRT/DF",
  "codigoNaturezaReceita": "0002",
  "nomeNaturezaReceita": "NAO TRIBUTARIA",
  "codigoReceitaPrincipal": "3623",
  "nomeReceita": "DIV.ATIVA-CLT",
  "codigoSerie": "5",
  "nomeSerie": "CONSOLIDACAO DAS LEIS TRABALHISTAS",
  "codigoOrgaoJustica": "100030",
  "nomeOrgaoJustica": "VARA DO TRABALHO - BRASILIA",
  "numeroJuizo": "000000",
  "descricaoJuizo": "Número da Vara Desconhecido",
  "dataProtocoloJudExecucao": "",
  "dataDistribuicaoJudicial": "",
  "indicadorMoedaTotalInscrito": "R$",
  "valorTotalInscritoMoeda": "805,06",
  "valorTotalInscritoIndex": "756,56",
  "indicadorMoedaTotalConsolidado": "R$",
  "valorTotalConsolidadoMoeda": "2.403,59",
  "indicadorMoedaTotalRemanescente": "R$",
  "valorRemanescenteMoeda": "805,06",
  "valorRemanescenteIndex": "756,56",
  "dataDevolucaoProcesso": "",
  "numeroAutoInfracao": "",
  "indicadorPrescricaoSV8": "NAO",
  "dataDecretacaoFalencia": "",
  "dataFimProcurador": "",
  "numeroImovelITR": "",
  "dataExtincaoInscricao": "",
  "motivoSuspensaoExigilidade": "",
  "numeroRipSpu": "",
  "indicadorAnaliseOrgaoOrigem": "NAO",
  "motivoExtincaoInscricao": "",
  "indicadorProtImpedAjuiz": "",
  "numeroAgrupamento": "",
  "numeroInscricaoOriginal": "00 0 00 000000-00",
  "numeroInscricaoDerivada1": "00 0 00 000000-00",
  "numeroInscricaoDerivada2": "00 0 00 000000-00",
  "numeroInscricaoDerivada3": "00 0 00 000000-00",
  "numeroInscricaoDerivada4": "00 0 00 000000-00",
  "numeroInscricaoDerivada5": "00 0 00 000000-00",
  "numeroInscricaoDerivada6": "00 0 00 000000-00",
  "numeroInscricaoDerivada7": "00 0 00 000000-00",
  "dcomp": "",
  "descricaoNaoCalculado": "",
  "codigoMunicipioSPU": 0,
  "codigoSistemaOrigem": "M",
  "descricaoSistemaOrigem": "Manual"
}
```

> As amostras de resposta contêm espaçamentos internos nos valores (ex.: `"90 6 17 000964-19"`)
> que refletem a renderização do PDF. Se os valores exatos da amostra importarem, confira no PDF.

### Consultando dados do Devedor (CPF ou CNPJ)

Parâmetros utilizados:

- `[HEADER] Accept: application/json`
- `[HEADER] Authorization: Bearer <TOKEN_TRIAL_PUBLICO>` <!-- token redigido -->
- `[GET] https://gateway.apiserpro.serpro.gov.br/consulta-divida-ativa-trial/api/v1/devedor/09781911768` — a url base é `https://gateway.apiserpro.serpro.gov.br/consulta-divida-ativa-trial/api/v1`, e o método é o `/v1/devedor/{numeroInscricaoDevedor}`.

Exemplo de chamada:

```
curl -X GET "https://gateway.apiserpro.serpro.gov.br/consulta-divida-ativa-trial/api/v1/devedor/09781911768" \
-H "accept: application/json" \
-H "Authorization: Bearer <TOKEN_TRIAL_PUBLICO>"
```

Exemplo de resposta esperada:

```json
{
  "numeroInscricao": "10 5 03 003880-14",
  "numeroProcesso": "10166 677771/2014-12",
  "situacaoInscricao": "121105",
  "situacaoDescricao": "ATIVA NAO PRIORIZADA PARA AJUIZAMENTO",
  "nomeDevedor": "DEVEDOR DA SILVA",
  "tipoDevedor": "PRINCIPAL",
  "valorTotalConsolidadoMoeda": "4.457,56",
  "cpfCnpj": "097.819.117-68",
  "codigoSida": "1000",
  "nomeUnidade": "PROCURADORIA REGIONAL DA FAZENDA NACIONAL DA 1A REGIAO",
  "codigoComprot": "1101951",
  "codigoUorg": "0005786"
}
```

> IMPORTANTE: As chamadas à consulta da API Dívida Ativa Trial acima são apenas para
> demonstração. As APIs disponíveis e suas respectivas URLs (endpoints) para consumo são
> disponibilizadas através da documentação dos seus respectivos swaggers. O gateway interpreta
> as letras maiúsculas e minúsculas de forma diferente. Certifique-se de enviar os endpoints
> conforme Referência da API sem alterar para maiúsculo ou minúsculo qualquer parte do endpoint.
