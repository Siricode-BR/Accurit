---
origem_pdf: Catálogo de Serviços - Integra Contador.pdf
origem_url: https://apicenter.estaleiro.serpro.gov.br/documentacao/api-integra-conta... (truncado no PDF)
capturado_em: 2026-10-17
conversao: manual-assistida
aviso: Cópia de trabalho. Em divergência, vale o PDF.
---

# Catálogo de Serviços

Este catálogo de serviços do Integra Contador exibe a situação de todos os serviços. Não é uma
página de Health Check, mas sim um local que contém o "roadmap" de todos os serviços do produto.

As colunas `idSistema` e `idServico` são utilizados no pedido de dados. A coluna `Tipo` representa
o método a ser invocado pelo serviço.

> Os `idSistema`/`idServico` aparecem partidos no texto extraído do PDF (ex.: `CAIXAPOST`+`AL`,
> `INNOVAMSG`+`63`). Aqui foram juntados e validados contra a lista de serviços e contra
> `servicos-x-procuracoes.md`. Pontos duvidosos estão marcados com `<!-- conferir -->`.

## Integra-SN

| Seq | Código | idSistema | idServico | Situação e Data da Implantação | Tipo |
|---|---|---|---|---|---|
| 001 | 1.1 | PGDASD | TRANSDECLARACAO11 | 23/09/2022 | Declarar |
| 002 | 1.2 | PGDASD | GERARDAS12 | 23/09/2022 | Emitir |
| 003 | 1.3 | PGDASD | CONSDECLARACAO13 | 23/09/2022 | Consultar |
| 004 | 1.4 | PGDASD | CONSULTIMADECREC14 | 23/09/2022 | Consultar |
| 005 | 1.5 | PGDASD | CONSDECREC15 | 23/09/2022 | Consultar |
| 006 | 1.6 | PGDASD | CONSEXTRATO16 | 23/09/2022 | Consultar |
| 007 | 1.7 | PGDASD | GERARDASCOBRANCA17 | 27/11/2024 | Emitir |
| 008 | 1.8 | PGDASD | GERARDASPROCESSO18 | 27/11/2024 | Emitir |
| 009 | 1.9 | PGDASD | GERARDASAVULSO19 | 27/11/2024 | Emitir |
| 010 | 10.1 | REGIMEAPURACAO | EFETUAROPCAOREGIME101 | 24/07/2023 | Declarar |
| 011 | 10.2 | REGIMEAPURACAO | CONSULTARANOSCALENDARIOS102 | 24/07/2023 | Consultar |
| 012 | 10.3 | REGIMEAPURACAO | CONSULTAROPCAOREGIME103 | 24/07/2023 | Consultar |
| 013 | 10.4 | REGIMEAPURACAO | CONSULTARRESOLUCAO104 | 24/07/2023 | Consultar |
| 014 | 14.1 | DEFIS | TRANSDECLARACAO141 | 25/09/2023 | Declarar |
| 015 | 14.2 | DEFIS | CONSDECLARACAO142 | 25/09/2023 | Consultar |
| 016 | 14.3 | DEFIS | CONSULTIMADECREC143 | 25/09/2023 | Consultar |
| 017 | 14.4 | DEFIS | CONSDECREC144 | 25/09/2023 | Consultar |

## Integra-MEI

| Seq | Código | idSistema | idServico | Situação e Data da Implantação | Tipo |
|---|---|---|---|---|---|
| 018 | 2.1 | PGMEI | GERARDASPDF21 | 23/09/2022 | Emitir |
| 019 | 2.2 | PGMEI | GERARDASCODBARRA22 | 23/09/2022 | Emitir |
| 020 | 2.3 | PGMEI | ATUBENEFICIO23 | 23/09/2022 | Emitir |
| 021 | 2.4 | PGMEI | DIVIDAATIVA24 | 23/09/2022 | Consultar |
| 022 | 12.1 | CCMEI | EMITIRCCMEI121 | 01/10/2024 | Emitir |
| 023 | 12.2 | CCMEI | DADOSCCMEI122 | 01/10/2024 | Consultar |
| 024 | 12.3 | CCMEI | CCMEISITCADASTRAL123 | 01/10/2024 | Consultar |
| 025 | 15.1 | DASNSIMEI | TRANSDECLARACAO151 | — | Declarar |
| 026 | 15.2 | DASNSIMEI | CONSULTIMADECREC152 | — | Consultar |
| 027 | 15.3 | DASNSIMEI | GERARDASEXCESSO153 | — | Emitir |

## Integra-DCTFWeb

| Seq | Código | idSistema | idServico | Situação e Data da Implantação | Tipo |
|---|---|---|---|---|---|
| 028 | 3.1 | DCTFWEB | GERARGUIA31 | 23/09/2022 | Emitir |
| 029 | 3.2 | DCTFWEB | CONSRECIBO32 | 23/09/2022 | Consultar |
| 030 | 3.3 | DCTFWEB | CONSDECCOMPLETA33 | 23/09/2022 | Consultar |
| 031 | 3.4 | DCTFWEB | CONSRELCREDITO34 | — | Consultar |
| 032 | 3.5 | DCTFWEB | CONSRELDEBITO35 | — | Consultar |
| 033 | 3.6 | DCTFWEB | GERARGUIAMAED36 | — | Emitir |
| 034 | 3.7 | DCTFWEB | CONSNOTIFMAED37 | — | Consultar |
| 035 | 3.8 | DCTFWEB | CONSXMLDECLARACAO38 | 23/09/2022 | Consultar |
| 036 | 3.9 | DCTFWEB | APLVINCULACAO39 | — | Emitir |
| 037 | 3.10 | DCTFWEB | TRANSDECLARACAO310 | 23/09/2022 | Declarar |
| 038 | 3.11 | DCTFWEB | GERARGUIACOMABATIMENTO311 | — | Emitir |
| 039 | 3.12 | DCTFWEB | EDITARVALORSUSPENSO312 | — | Emitir |
| 040 | 3.13 | DCTFWEB | GERARGUIAANDAMENTO313 | 28/03/2025 | Emitir |
| 041 | 3.14 | MIT | ENCAPURACAO314 | 28/03/2025 | Declarar |
| 042 | 3.15 | MIT | SITUACAOENC315 | 28/03/2025 | Apoiar |
| 043 | 3.16 | MIT | CONSAPURACAO316 | 28/03/2025 | Consultar |
| 044 | 3.17 | MIT | LISTAAPURACOES317 | 28/03/2025 | Consultar |

## Integra-Procurações

| Seq | Código | idSistema | idServico | Situação e Data da Implantação | Tipo |
|---|---|---|---|---|---|
| 045 | 4.1 | PROCURACOES | OBTERPROCURACAO41 | 23/09/2022 | Consultar |

## Integra-Sicalc

| Seq | Código | idSistema | idServico | Situação e Data da Implantação | Tipo |
|---|---|---|---|---|---|
| 046 | 5.1 | SICALC | CONSOLIDARGERARDARF51 | 23/09/2022 | Emitir |
| 047 | 5.2 | SICALC | CONSULTAAPOIORECEITAS52 | 23/09/2022 | Apoiar |
| 048 | 5.3 | SICALC | GERARDARFCODBARRA53 | 04/12/2023 | Emitir |
| 049 | 5.4 | SICALC | CONSOLIDAR54 | 15/10/2026 | Consultar |

## Integra-CaixaPostal

| Seq | Código | idSistema | idServico | Situação e Data da Implantação | Tipo |
|---|---|---|---|---|---|
| 050 | 6.1 | CAIXAPOSTAL | MSGCONTRIBUINTE61 | 23/09/2022 | Consultar |
| 051 | 6.2 | CAIXAPOSTAL | MSGDETALHAMENTO62 | 23/09/2022 | Consultar |
| 052 | 6.3 | CAIXAPOSTAL | INNOVAMSG63 | 23/09/2022 | Monitorar |
| 053 | 11.1 | DTE | CONSULTASITUACAODTE111 | 01/09/2023 | Consultar |

## Integra-Pagamento

| Seq | Código | idSistema | idServico | Situação e Data da Implantação | Tipo |
|---|---|---|---|---|---|
| 054 | 7.1 | PAGTOWEB | PAGAMENTOS71 | 23/09/2022 | Consultar |
| 055 | 7.2 | PAGTOWEB | COMPARRECADACAO72 | 23/09/2022 | Emitir |
| 056 | 7.3 | PAGTOWEB | CONTACONSDOCARRPG73 | 23/09/2022 | Consultar |

## Integra-Contador-Gerenciador

| Seq | Código | idSistema | idServico | Situação e Data da Implantação | Tipo |
|---|---|---|---|---|---|
| 057 | 8.1 | AUTENTICAPROCURADOR | ENVIOXMLASSINADO81 | 23/09/2022 | Apoiar |
| 058 | 8.2 | AUTENTICAPROCURADOR | EXPIRARAUTORIZACAO82 | Descontinuado em 01/04/2024 | Apoiar |
| 059 | 13.1 | EVENTOSATUALIZACAO | SOLICEVENTOSPF131 | 01/04/2024 | Monitorar |
| 060 | 13.2 | EVENTOSATUALIZACAO | SOLICEVENTOSPJ132 | 01/04/2024 | Monitorar |
| 061 | 13.3 | EVENTOSATUALIZACAO | OBTEREVENTOSPF133 | 01/04/2024 | Monitorar |
| 062 | 13.4 | EVENTOSATUALIZACAO | OBTEREVENTOSPJ134 | 01/04/2024 | Monitorar |

## Integra-SITFIS

| Seq | Código | idSistema | idServico | Situação e Data da Implantação | Tipo |
|---|---|---|---|---|---|
| 063 | 9.1 | SITFIS | SOLICITARPROTOCOLO91 | 01/09/2023 | Apoiar |
| 064 | 9.2 | SITFIS | RELATORIOSITFIS92 | 01/09/2023 | Emitir |

## Integra-Parcelamentos

| Seq | Código | idSistema | idServico | Situação e Data da Implantação | Tipo |
|---|---|---|---|---|---|
| 065 | 16.1 | PARCSN | GERARDAS161 | 11/11/2024 | Emitir |
| 066 | 16.2 | PARCSN | PARCELASPARAGERAR162 | 11/11/2024 | Consultar |
| 067 | 16.3 | PARCSN | PEDIDOSPARC163 | 11/11/2024 | Consultar |
| 068 | 16.4 | PARCSN | OBTERPARC164 | 11/11/2024 | Consultar |
| 069 | 16.5 | PARCSN | DETPAGTOPARC165 | 11/11/2024 | Consultar |
| 070 | 17.1 | PARCSN-ESP | GERARDAS171 | 11/11/2024 | Emitir |
| 071 | 17.2 | PARCSN-ESP | PARCELASPARAGERAR172 | 11/11/2024 | Consultar |
| 072 | 17.3 | PARCSN-ESP | PEDIDOSPARC173 | 11/11/2024 | Consultar |
| 073 | 17.4 | PARCSN-ESP | OBTERPARC174 | 11/11/2024 | Consultar <!-- conferir no PDF, p.17 (código aparece como "7.4"; validado como 17.4 via servicos-x-procuracoes.md) --> |
| 074 | 17.5 | PARCSN-ESP | DETPAGTOPARC175 | 11/11/2024 | Consultar |
| 075 | 18.1 | PERTSN | GERARDAS181 | 11/11/2024 | Emitir |
| 076 | 18.2 | PERTSN | PARCELASPARAGERAR182 | 11/11/2024 | Consultar |
| 077 | 18.3 | PERTSN | PEDIDOSPARC183 | 11/11/2024 | Consultar |
| 078 | 18.4 | PERTSN | OBTERPARC184 | 11/11/2024 | Consultar |
| 079 | 18.5 | PERTSN | DETPAGTOPARC185 | 11/11/2024 | Consultar |
| 080 | 19.1 | RELPSN | GERARDAS191 | 11/11/2024 | Emitir |
| 081 | 19.2 | RELPSN | PARCELASPARAGERAR192 | 11/11/2024 | Consultar |
| 082 | 19.3 | RELPSN | PEDIDOSPARC193 | 11/11/2024 | Consultar |
| 083 | 19.4 | RELPSN | OBTERPARC194 | 11/11/2024 | Consultar |
| 084 | 19.5 | RELPSN | DETPAGTOPARC195 | 11/11/2024 | Consultar |
| 085 | 20.1 | PARCMEI | GERARDAS201 | 11/11/2024 | Emitir |
| 086 | 20.2 | PARCMEI | PARCELASPARAGERAR202 | 11/11/2024 | Consultar |
| 087 | 20.3 | PARCMEI | PEDIDOSPARC203 | 11/11/2024 | Consultar |
| 088 | 20.4 | PARCMEI | OBTERPARC204 | 11/11/2024 | Consultar |
| 089 | 20.5 | PARCMEI | DETPAGTOPARC205 | 11/11/2024 | Consultar |
| 090 | 21.1 | PARCMEI-ESP | GERARDAS211 | 11/11/2024 | Emitir |
| 091 | 21.2 | PARCMEI-ESP | PARCELASPARAGERAR212 | 11/11/2024 | Consultar |
| 092 | 21.3 | PARCMEI-ESP | PEDIDOSPARC213 | 11/11/2024 | Consultar |
| 093 | 21.4 | PARCMEI-ESP | OBTERPARC214 | 11/11/2024 | Consultar |
| 094 | 21.5 | PARCMEI-ESP | DETPAGTOPARC215 | 11/11/2024 | Consultar |
| 095 | 22.1 | PERTMEI | GERARDAS221 | 11/11/2024 | Emitir |
| 096 | 22.2 | PERTMEI | PARCELASPARAGERAR222 | 11/11/2024 | Consultar |
| 097 | 22.3 | PERTMEI | PEDIDOSPARC223 | 11/11/2024 | Consultar |
| 098 | 22.4 | PERTMEI | OBTERPARC224 | 11/11/2024 | Consultar |
| 099 | 22.5 | PERTMEI | DETPAGTOPARC225 | 11/11/2024 | Consultar |
| 100 | 23.1 | RELPMEI | GERARDAS231 | 11/11/2024 | Emitir |
| 101 | 23.2 | RELPMEI | PARCELASPARAGERAR232 | 11/11/2024 | Consultar |
| 102 | 23.3 | RELPMEI | PEDIDOSPARC233 | 11/11/2024 | Consultar |
| 103 | 23.4 | RELPMEI | OBTERPARC234 | 11/11/2024 | Consultar |
| 104 | 23.5 | RELPMEI | DETPAGTOPARC235 | 11/11/2024 | Consultar |
| 105 | 24.1 | PARC-PAEX | OBTEREXTRATOPDF171 | — | Consultar |
| 106 | 24.2 | PARC-PAEX | OBTEREXTRATOJSON172 | — | Consultar |
| 107 | 24.3 | PARC-PAEX | EMITIRDOCARRECADACAO173 | — | Emitir |
| 108 | 25.1 | PARC-SIPADE | OBTEREXTRATOPDF181 | — | Consultar |
| 109 | 25.2 | PARC-SIPADE | OBTEREXTRATOJSON182 | — | Consultar |
| 110 | 25.3 | PARC-SIPADE | EMITIRDOCARRECADACAO183 | — | Emitir |

## Integra-Redesim

| Seq | Código | idSistema | idServico | Situação e Data da Implantação | Tipo |
|---|---|---|---|---|---|
| 111 | 26.1 | PNRCONTADOR | CONSVINCULOS261 | 2/12/2025 | Consultar |
| 112 | 26.2 | PNRCONTADOR | SOLICRENUNCIA262 | 2/12/2025 | Declarar |
| 113 | 26.3 | PNRCONTADOR | CONSRENUNCIA263 | 2/12/2025 | Consultar |
| 114 | 26.4 | PNRCONTADOR | COMPRENUNCIA264 | 2/12/2025 | Emitir |
| 115 | 26.5 | PNRCONTADOR | SITSOLICRENUNCIA265 | 2/12/2025 | Consultar |

## Integra-e-Processo

| Seq | Código | idSistema | idServico | Situação e Data da Implantação | Tipo |
|---|---|---|---|---|---|
| 116 | 27.1 | EPROCESSO | CONSPROCPORINTER271 | 10/11/2025 | Consultar |
| 117 | 27.2 | EPROCESSO | OBTLISTDOCSPROC272 | — | Consultar |
| 118 | 27.3 | EPROCESSO | OBTDOCPROC273 | — | Consultar |
| 119 | 27.4 | EPROCESSO | CONSCOMUNINTIM274 | — | Consultar |

## Legenda

- Em produção
- Em demonstração
- Em construção
- Em prospecção
- Cancelado
- Disponível em produção
- Disponível em demonstração
- Em desenvolvimento
- Em análise e planejamento.
- Serviço Descontinuado.

---

Documento datado de: **15 de outubro de 2026**
