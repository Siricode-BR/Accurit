---
origem_pdf: Envio de XML Assinado - Integra Contador.pdf
origem_url: https://apicenter.estaleiro.serpro.gov.br/documentacao/api-integra-contador/pt/solucoes/integra-contador-gerenciador/autenticaprocurador/servicos/envio_de_xml_assinado/
capturado_em: 2026-10-20
conversao: print-to-pdf via navegador headless (Edge) + extração do HTML oficial
aviso: Cópia de trabalho. Em divergência, vale o PDF.
---

# Envio de XML assinado com o Termo de Autorização (`ENVIOXMLASSINADO81`)

> Documento datado de **10 de abril de 2026** (rodapé da página oficial).

Permite o envio de um documento XML assinado digitalmente pelo Autor Pedido de Dados
(Procurador) para receber um **TOKEN de autorização** que permite o Contratante realizar as
requisições em nome do Autor Pedido de Dados. O XML tem como conteúdo um TERMO DE
AUTORIZAÇÃO estruturado na sintaxe XML, previamente assinado por um assinador de documentos
XML seguindo os padrões da página *Padrões Técnicos de Assinatura de XML*.

## Identificação no Pedido de Dados

| Campo | Valor |
|---|---|
| `idSistema` | `AUTENTICAPROCURADOR` |
| `idServico` | `ENVIOXMLASSINADO81` |
| `versaoSistema` | `"1.0"` |

## Dados de Entrada

| Campo | Descrição | Tipo | Obrigatório |
|---|---|---|---|
| `xml` | Documento XML assinado previamente com certificado digital do Autor Pedido de Dados (eCPF ou eCNPJ — ICP-Brasil) nos padrões XMLDSig e codificado em **base64**. | String escapada | Sim |

## Exemplo: conteúdo do body JSON de entrada

```json
{
  "contratante": { "numero": "99999999999999", "tipo": 2 },
  "autorPedidoDados": { "numero": "00000000000000", "tipo": 2 },
  "contribuinte": { "numero": "11111111111111", "tipo": 2 },
  "pedidoDados": {
    "idSistema": "AUTENTICAPROCURADOR",
    "idServico": "ENVIOXMLASSINADO81",
    "versaoSistema": "1.0",
    "dados": "{\"xml\": \"<base64 do XML assinado>\"}"
  }
}
```

## Dados de Saída

| Campo | Descrição | Tipo |
|---|---|---|
| `status` | Status HTTP retornado no acionamento do serviço. | Number |
| `mensagens` | Mensagem explicativa retornada no acionamento do serviço. Array composto de Código e texto da mensagem. O campo Código é um texto de tamanho **5** que representa um código interno do negócio. | Array |
| `dados` | Estrutura de dados de retorno. | String escapada — Object `Autentica` |

### Object `Autentica`

| Campo | Descrição | Tipo |
|---|---|---|
| `autenticar_procurador_token` | Token de acesso para ser utilizado nas requisições do Integra Contador via HTTP HEADER. | String |
| `data_hora_expiracao` | Data e hora da expiração do token. Formato: `yyyy-MM-dd'T'HH:mm:ss`. | String |

Json de exemplo: retorno do envio de XML assinado com o recebimento do token de acesso —
ver página *Retorno envio XML assinado* da documentação oficial (mesma seção).
