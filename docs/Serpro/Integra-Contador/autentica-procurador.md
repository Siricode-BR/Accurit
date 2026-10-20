---
origem_pdf: Autentica Procurador - Integra Contador.pdf
origem_url: https://apicenter.estaleiro.serpro.gov.br/documentacao/api-integra-contador/pt/solucoes/integra-contador-gerenciador/autenticaprocurador/
capturado_em: 2026-10-20
conversao: print-to-pdf via navegador headless (Edge) + extração do HTML oficial
aviso: Cópia de trabalho. Em divergência, vale o PDF.
---

# Autentica Procurador — Introdução

> Documento datado de **13 de fevereiro de 2026** (rodapé da página oficial).

## O que é o Autentica Procurador?

É uma funcionalidade de **apoio à segurança** da API Integra Contador, que permite o
**Autor do Pedido de Dados** autorizar o **Contratante** a enviar requisições em seu nome por
meio de um **Termo de Autorização** em formato de documento XML assinado digitalmente com
certificado digital. Em muitos casos o Autor do Pedido de Dados não é o Contratante da API,
mas é um **Procurador autorizado previamente pelo Contribuinte pelo portal eCAC**.

## Como funciona o uso via Integra Contador?

Para acionar os serviços do Autentica-Procurador é necessário utilizar a API Integra Contador,
que gerencia o pedido de dados contendo o XML assinado eletronicamente pelo Autor do Pedido
de Dados. Esse **TERMO DE AUTORIZAÇÃO** é validado junto da autenticidade da assinatura
digital e também do certificado digital do Autor do Pedido de Dados.

- Certificado digital aceito: **eCPF ou eCNPJ, padrão ICP-Brasil (A1 ou A3)**.
- ⚠️ Importante: o sistema aceita contribuintes do **tipo 1 (Pessoa Física)** e
  **2 (Pessoa Jurídica)**.
- Se o termo for válido, o Autentica-Procurador disponibiliza um **token de acesso com tempo
  de duração predeterminado**, informado no header das requisições da API no parâmetro
  `autenticar_procurador_token`. Com isso o Contratante terá autorização para acionar a API
  em nome do Autor do Pedido de Dados com procuração eletrônica no eCAC.
- A estrutura básica de entrada contém os dados do Contratante, Autor do Pedido de Dados,
  Contribuinte e os dados do Pedido de Dados — tudo no JSON do corpo (POST).
- O **TERMO DE AUTORIZAÇÃO** deve ser assinado com um assinador que siga os padrões
  **XMLDSig do W3C**. Após assinado, o XML deve ser submetido via POST como **string base64**.
  O padrão de assinatura digital segue o mesmo adotado pelos sistemas **NFe e eSocial da RFB**.

## Sobre a validade do token autenticado

O token válido fica disponível **até a meia-noite (horário de Brasília) do dia seguinte**.
Assim, se o cliente enviar o Termo de Autorização na madrugada (ex.: à meia-noite), o token
poderá valer até 24 horas, sem necessidade de submeter o mesmo documento assinado
repetidamente.

Para mais informações sobre como realizar a assinatura do TERMO DE AUTORIZAÇÃO, ver
[padroes-tecnicos-assinatura-xml.md](padroes-tecnicos-assinatura-xml.md).
