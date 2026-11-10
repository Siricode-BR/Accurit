# LGPD — retenção, portabilidade e anonimização

## Base e escopo

O escritório é o controlador dos dados dos seus clientes PF/PJ; a Accurit atua como operadora.
Todo acesso é isolado por `escritorio_id` derivado do token, nunca do corpo da requisição.

## Direitos atendidos

| Direito | Endpoint | Observação |
| --- | --- | --- |
| Portabilidade | `GET /api/v1/lgpd/clientes/{id}/export` | JSON com cliente, consultas, guias, relatórios e consentimentos |
| Eliminação/anonimização | `DELETE /api/v1/lgpd/clientes/{id}` | Substitui PII por valores anonimizados e registra `LGPD_CLIENTE_ANONIMIZADO` em auditoria |
| Consentimento | `GET/POST /api/v1/lgpd/clientes/{id}/consentimentos` | Tipos `tratamento_dados`, `compartilhamento_serpro`, `comunicacoes`; versão obrigatória |
| Retenção | `GET/PUT /api/v1/lgpd/retencao` | Prazos em dias para consultas, relatórios e auditoria (1–3650) |

Perfis `CLIENTE` não podem anonimizar nem alterar retenção (403). O histórico de auditoria
preserva o evento da anonimização, sem reter a PII substituída.

## Retenção

Os prazos ficam em `politicas_retencao_escritorio` (padrões: consultas 90 dias, relatórios 30,
auditoria 365). A configuração é por escritório e deve ser revisada antes do go-live. O expurgo
dos brutos de medição e do histórico fiscal usa as rotinas existentes/configuráveis; o prazo de
`chamadas_serpro` deve seguir a política comercial e ser validado com o jurídico.

## Logs e PII

Nenhum payload de cliente, documento, token ou certificado é escrito em log. Erros retornam
apenas código/mensagem amigável e `requestId`; o detalhe técnico fica restrito ao log redigido.
