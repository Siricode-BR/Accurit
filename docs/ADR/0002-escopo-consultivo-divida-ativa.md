# ADR-0002 — Escopo Exclusivamente Consultivo da Dívida Ativa

- **Status:** Aceito
- **Data:** 2026-10-20
- **Decisores:** Arquitetura Accurit
- **Escopo:** Módulo Dívida Ativa da União e limites de produto
- **Relaciona-se com:** [ADR-0001](0001-escolha-stack-tecnologica.md), [ADR-0003](0003-envelope-verbos-integra-contador.md), [ADR-0004](0004-seguranca-cabecalho-assinatura-xml.md)

## Contexto

A Accurit integra a API de Consulta Dívida Ativa da União, disponibilizada pela SERPRO, para que escritórios de contabilidade identifiquem pendências e detalhem inscrições de seus clientes. A API expõe operações de leitura:

| Método | Endpoint (relativo à base) | Finalidade |
|---|---|---|
| `GET` | `/devedor/{cpfCnpj}` | Lista e resume as inscrições/pendências do devedor |
| `GET` | `/inscricao/{numeroInscricao}` | Detalha uma inscrição individual |

A base Trial é `https://gateway.apiserpro.serpro.gov.br/consulta-divida-ativa-trial/api/v1`
(os paths são concatenados a ela; **não** adicionar outro `/v1`). A base de produção é
**indefinida** e a chamada é bloqueada com `SERPRO_PROD_NOT_CONFIGURED` até confirmação
documental da SERPRO (D5). A autenticação usa OAuth2 Client Credentials e token Bearer; o header
`x-cpf-usuario` é operacional e **não consta na documentação oficial** (D3, ADR-0004).

O catálogo do Integra Contador contém serviços de parcelamento e negociação de débitos (por exemplo, `PARCSN`, `PARCSN-ESP`, `PERTSN`, `RELPSN`, `PARCMEI`, `PARCMEI-ESP`, `PERTMEI`, `RELPMEI`, `PARC-PAEX` e `PARC-SIPADE`). A existência desses serviços no catálogo não significa que eles devam ser expostos pela Accurit. O produto é posicionado como plataforma de gestão contábil e consulta fiscal, não como canal de negociação, cobrança ou liquidação de dívida.

Há riscos relevantes em permitir qualquer fluxo mutável nesse domínio:

- risco jurídico e regulatório por intermediar negociação ou pagamento;
- risco de o escritório acreditar que a Accurit quita, parcela ou regulariza débitos;
- risco de duplicidade de pedidos, emissão indevida de guias e efeitos fiscais irreversíveis;
- ampliação desnecessária da superfície de ataque e da complexidade de auditoria;
- confusão entre consulta informativa e aconselhamento fiscal.

## Decisão

O módulo Dívida Ativa da Accurit é **estritamente consultivo e somente leitura**.

1. A plataforma expõe apenas as duas operações de consulta da API SERPRO:
   - `GET {base}/devedor/{cpfCnpj}`;
   - `GET {base}/inscricao/{numeroInscricao}`.
2. Internamente, a API da Accurit expõe somente:
   - `GET /api/v1/divida-ativa/devedores/{cpfCnpj}`;
   - `GET /api/v1/divida-ativa/inscricoes/{numero}`.
3. É proibido criar, no backend, casos de uso, comandos, repositórios de escrita ou endpoints para:
   - parcelamento;
   - negociação;
   - adesão a transação ou programa de regularização;
   - liquidação ou quitação;
   - cobrança;
   - emissão de DARF ou documento de arrecadação de dívida ativa;
   - alteração de situação cadastral ou fiscal do devedor.
4. É proibido criar, no Admin ou no portal do cliente, telas, botões, rotas, formulários ou mensagens que sugiram a prática dessas operações.
5. Os serviços de parcelamento/negociação do catálogo do Integra Contador não serão mapeados, habilitados ou oferecidos pela Accurit, **inclusive os serviços `Consultar`**. O adapter do Integra Contador pode suportar os cinco verbos operacionais, mas o catálogo exposto ao produto passa por uma allowlist que exclui esses sistemas (D4).
6. O domínio `divida_ativa` é composto por **read models/projeções**. Não há agregados responsáveis por negociar, parcelar ou liquidar débitos. O banco de dados pode armazenar histórico de consultas e metadados de auditoria, mas não um estado de negociação.
7. O fallback de CPF/CNPJ para “Código de Contribuinte” é **NÃO CONFIRMADO** e não existe na
   documentação oficial. Ele fica desabilitado por padrão
   (`DIVIDA_ATIVA_FALLBACK_CODIGO_CONTRIBUINTE=false`), atrás da porta
   `ResolvedorChaveAlternativa`, e não altera o caráter consultivo (ADR-0004, D2).
8. Resultado vazio, nulo ou sem inscrições não é erro: a resposta deve indicar “sem pendências” quando a chave consultada retornar vazia de forma inequívoca. O teste live deve distinguir `200`
   vazio de `404`. Dados nunca são inventados ou inferidos.
9. Toda consulta registra metadados mínimos: tenant, cliente, ambiente, chave/fonte utilizada, status, data e protocolo/referência. Payloads fiscais completos não são persistidos sem necessidade e política explícita.
10. Qualquer proposta de adicionar negociação, parcelamento ou liquidação exige **nova ADR**, revisão jurídica/compliance e aprovação explícita de arquitetura. Não é uma mudança de código ordinária.

### Limites de implementação

| Artefato | Permitido | Proibido |
|---|---|---|
| Rotas HTTP | `GET /devedores/{cpfCnpj}`, `GET /inscricoes/{numero}` | `POST`, `PUT`, `PATCH` ou `DELETE` de negociação/parcelamento |
| Casos de uso | Consultar devedor, consultar inscrição, registrar auditoria | Negociar, parcelar, liquidar, cobrar, emitir DARF de dívida ativa |
| Repositórios | Leitura de histórico de consultas e configuração do tenant | Persistir acordos, parcelas, pagamentos ou quitações |
| UI Admin | Busca, filtros, detalhe, estados de erro e ambiente | Ações de negociação, parcelamento, pagamento ou emissão de guia |
| UI Cliente | Consulta e visualização | Qualquer ação transacional sobre a dívida |
| Integra Contador | Serviços autorizados compatíveis com o escopo do produto; `SITFIS RELATORIOSITFIS92` é **Emitir** | Todos os sistemas de parcelamento (`PARCSN`, `PARCSN-ESP`, `PERTSN`, `RELPSN`, `PARCMEI`, `PARCMEI-ESP`, `PERTMEI`, `RELPMEI`, `PARC-PAEX`, `PARC-SIPADE`); `SICALC` `Emitir` (`CONSOLIDARGERARDARF51`, `GERARDARFCODBARRA53`); `PGMEI / DIVIDAATIVA24` fica `AguardandoDecisao` |
| Banco | `consultas_fiscais`, `auditoria_eventos` e dados cadastrais | Tabelas de acordos, parcelas, pagamentos ou liquidação |

### Alternativas consideradas

| Alternativa | Motivo da não adoção |
|---|---|
| Expor também o catálogo de parcelamentos do Integra Contador | Ampliaria o escopo jurídico e operacional, contrariaria o posicionamento do produto e criaria risco de efeitos fiscais irreversíveis. |
| Criar fluxo de “pré-negociação” apenas informativo | Ainda criaria expectativa de negociação e exigiria regras, consentimentos e auditoria específicos; o produto não é canal de regularização. |
| Delegar negociação ao e-CAC por links externos | Links informativos são permitidos, mas não podem ser apresentados como funcionalidade da Accurit nem simular uma transação interna. |
| Manter endpoints mutáveis ocultos atrás de feature flag | Aumentaria a superfície de ataque e o risco de ativação acidental; a proibição deve ser estrutural. |

## Consequências

### Positivas

- **Aderência ao escopo:** o produto permanece uma plataforma de consulta e gestão contábil, sem assumir responsabilidades de negociação ou cobrança.
- **Redução de risco jurídico e fiscal:** não há intermediação de pagamento, parcelamento ou quitação.
- **Superfície de ataque menor:** menos endpoints mutáveis, menos estados transacionais e menos dados sensíveis persistidos.
- **Auditoria mais simples:** o histórico registra consultas e metadados, não acordos ou pagamentos.
- **UX honesta:** o usuário entende que a Accurit informa pendências, mas a regularização ocorre nos canais oficiais.
- **Menor custo de manutenção:** não há regras de parcelamento, cálculo de guias, idempotência de negociação ou conciliação de pagamentos.

### Negativas

- **Escopo comercial limitado:** escritórios que desejam negociar débitos precisarão usar o e-CAC ou outro canal.
- **Possível perda de percepção de valor:** usuários podem esperar uma solução “fim a fim” e precisarão ser educados sobre o limite.
- **Pressão por expansão de escopo:** vendas ou clientes podem solicitar parcelamento; a decisão precisa ser defendida continuamente.
- **Necessidade de mensagens claras:** a interface deve explicar o que é consultivo e o que deve ser feito fora da plataforma, sem sugerir que a Accurit realiza a regularização.
- **Dependência de canais externos:** a jornada do usuário continua no e-CAC ou no SERPRO após a consulta.

### Mitigações

- Incluir aviso permanente de que a Accurit é consultiva e não realiza negociação, parcelamento, liquidação ou emissão de DARF de dívida ativa.
- Oferecer links para os canais oficiais (e-CAC/SERPRO) quando apropriado, deixando claro que são externos.
- Manter o escopo documentado no README, SPEC e roadmap.
- Adicionar testes de arquitetura que falhem se rotas ou casos de uso proibidos forem criados.
- Revisar o catálogo exposto ao produto em cada atualização de serviços SERPRO.

## Conformidade e Regras de Negócio

1. **Somente leitura:** o módulo Dívida Ativa não pode executar métodos HTTP mutáveis contra a SERPRO. O adapter deve permitir apenas `GET`.
2. **Allowlist de rotas:** o router de Dívida Ativa deve conter somente as duas rotas `GET` internas. Rotas adicionais exigem nova ADR.
3. **Allowlist de serviços:** o catálogo do Integra Contador exposto pela Accurit não pode incluir sistemas de parcelamento, negociação ou liquidação. A lista de exclusão deve ser mantida em código/configuração e coberta por testes.
4. **Domínio read-only:** não podem existir comandos, agregados ou repositórios de escrita para negociação, parcelamento ou liquidação.
5. **UI consultiva:** nenhuma tela pode exibir botões, formulários ou chamadas de ação para negociar, parcelar, pagar ou emitir DARF de dívida ativa.
6. **Fallback consultivo:** o fallback por Código de Contribuinte é **NÃO CONFIRMADO** e fica
   desabilitado por padrão. Quando habilitado por `DIVIDA_ATIVA_FALLBACK_CODIGO_CONTRIBUINTE=true`,
   ele apenas repete uma consulta de leitura e deve ser rastreado em
   `consultas_fiscais.fonte_consulta` (D2).
7. **Ausência de dados:** se a chave consultada retornar vazio de forma inequívoca, a resposta é
   “sem pendências”, não erro e não dado inventado.
8. **Auditoria:** toda consulta registra tenant, cliente, ambiente, fonte, status e data; não registra payload fiscal completo sem política explícita.
9. **Isolamento:** toda consulta é filtrada por `escritorio_id` derivado do contexto autenticado; o usuário não pode consultar cliente de outro tenant.
10. **Segurança:** o header `x-cpf-usuario` e a autenticação SERPRO seguem a [ADR-0004](0004-seguranca-cabecalho-assinatura-xml.md). Nenhuma credencial é enviada ao frontend.
11. **Governança:** qualquer solicitação de negociação/parcelamento deve ser tratada como mudança de escopo de produto, com nova ADR, análise jurídica e aprovação explícita.
12. **DARF e SICALC:** a proibição de “emissão de DARF” refere-se à **dívida ativa**. `SICALC`
    `Emitir` (`CONSOLIDARGERARDARF51`, `GERARDARFCODBARRA53`) e `PGDASD / GERARDAS12` não são
    dívida ativa, mas continuam sujeitos à allowlist: SICALC fica bloqueado (D4) e PGDASD depende de
    decisão explícita antes de ser habilitado.

## Critérios de verificação

- O OpenAPI/roteador da Accurit contém apenas `GET` para Dívida Ativa.
- Não existem casos de uso, tabelas ou telas de negociação/parcelamento/liquidação.
- O catálogo exposto não contém os sistemas de parcelamento listados nesta ADR.
- Consulta por devedor e por inscrição funciona no ambiente resolvido pelo backend.
- O fallback por Código de Contribuinte permanece desabilitado por padrão e, quando habilitado, é
  registrado e não altera o caráter consultivo.
- Testes de arquitetura e revisão de código bloqueiam a introdução de rotas mutáveis.
- A UI exibe aviso de escopo consultivo e não oferece ações de regularização.

## Referências

- [Quick start — Consulta Dívida Ativa](../Serpro/Divida-Ativa/quick-start-consulta-divida-ativa.md)
- [Catálogo de Serviços — Integra Contador](../Serpro/Integra-Contador/catalogo-de-servicos.md)
- [Serviços x Procurações — Integra Contador](../Serpro/Integra-Contador/servicos-x-procuracoes.md)
- [SPEC — Módulo Dívida Ativa](../../SPEC.md)
- [ADR-0001 — Escolha da stack tecnológica](0001-escolha-stack-tecnologica.md)
- [ADR-0003 — Envelope e verbos do Integra Contador](0003-envelope-verbos-integra-contador.md)
- [ADR-0004 — Segurança, cabeçalho e assinatura XML](0004-seguranca-cabecalho-assinatura-xml.md)
