# GO-LIVE.md — Checklist de Produção (FASE H)

> Documento de referência para lançamento em produção da plataforma Accurit.
> Itens marcados como **pendente de ação externa** dependem de confirmação da
> SERPRO ou de contratos comerciais antes de habilitar em produção.

## 1. Checklist de Go-Live

### D5 — URL de produção da Dívida Ativa

| Item | Status | Detalhes |
| ------ | -------- | ---------- |
| URL de produção configurada | **Pendente** | `SERPRO_DIVIDA_ATIVA_PROD_URL` não confirmado pela SERPRO. A chamada à produção da Dívida Ativa está bloqueada por `DividaAtivaProducaoBloqueada` (endereço `endpoint_resolver`, infra/serpro/endpoints.rs). |
| Comportamento quando URL ausente | ✅ Implementado | Retorna `ErroEndpoint::DividaAtivaProducaoBloqueada` com código `SERPRO_ENDPOINT_INDEFINIDO`. Nunca faz fallback silencioso para Trial. |
| Teste live opt-in | **Pendente** | Requer contrato/credencial de produção da SERPRO para o produto Dívida Ativa. |

### P5 — Produção do Integra Contador (SAPI / mTLS e-CNPJ)

| Item | Status | Detalhes |
| ------ | -------- | ---------- |
| Endpoint de homologação configurado | ✅ Implementado | `SERPRO_INTEGRA_PROD_URL` lido da configuração. Estratégia `Oauth2Producao` com `consumer_key`/`consumer_secret` via OAuth2 client-credentials. |
| mTLS com e-CNPJ | **Pendente** | Estrutura de autenticação preparada (`EstrategiaAutenticacaoSerpro::Oauth2Producao`). A validação do mTLS com certificado A1 em homologação depende de ação externa (certificado válido, cadeia de confiança, formato exato aceito pelo SERPRO). |
| Token JWT (access_token + jwt_token) | **Pendente** | Fluxo SAPI documentado em `docs/Serpro/Integra-Contador/`. Falta validar: formato exato do certificado, necessidade de `x-jwt-assertion`, duração/renovação do `jwt_token`, comportamento de `401`/`403` por produto. |
| Teste live em homologação | **Pendente** | Executar com credencial de homologação da SERPRO e validar end-to-end. |

### P4 — `versaoSistema` dos serviços

| Item | Status | Detalhes |
|------|--------|----------|
| `versaoSistema` confirmado | **Parcial** | `OBTERPROCURACAO41 = "1"` e `ENVIOXMLASSINADO81 = "1.0"` confirmados. Os demais serviços devem ser habilitados um a um, com `versaoSistema` extraída do catálogo/PDF e coberta por fixture. |
| Habilitação serviço a serviço | **Pendente** | Não usar valor global. Cada linha do catálogo (`catalogo_servicos_integra`) deve ter `versao_confirmada = TRUE` antes de ser exposta. |

### D2 — Fallback “Código de Contribuinte”

| Item | Status | Detalhes |
|------|--------|----------|
| Habilitado por padrão | ✅ Não | `DIVIDA_ATIVA_FALLBACK_CODIGO_CONTRIBUINTE=false`. Porta `ResolvedorChaveAlternativa` sem implementação configurada. |
| Teste live opt-in | **Pendente** | Executar com `RUN_LIVE_TRIAL=1` e chave alternativa real fornecida pela SERPRO. Verificar se o path aceita o valor e qual o comportamento de erro. |

### P2 — Formato de “sem pendências” (200 vazio vs 404)

| Item | Status | Detalhes |
|------|--------|----------|
| Comportamento documentado | ✅ Implementado | DTO `RespostaDevedorSerpro` usa `#[serde(untagged)]` para aceitar objeto ou lista. `200` com corpo vazio/lista vazia = SEM_PENDENCIAS. `404` não é convertido automaticamente em “sem pendências”. |
| Teste live opt-in | **Pendente** | Executar com CPF/CNPJ sem pendências conhecido. Validar se `200` vazio ou `404` é retornado. |

### P3 — Formato multi-inscrição de `/devedor`

| Item | Status | Detalhes |
|------|--------|----------|
| DTO tolerante | ✅ Implementado | `RespostaDevedorSerpro` aceita objeto plano ou lista via `#[serde(untagged)]`. |
| Teste live | **Pendente** | Validar com devedor que tenha múltiplas inscrições. Confirmar se o endpoint retorna lista. |

## 2. Variáveis de Ambiente Obrigatórias (Produção)

| Variável | Obrigatória | Exemplo | Descrição |
| ---------- | ------------- | --------- | ----------- |
| `APP_ENV` | Sim | `production` | Deve ser `production` para habilitar cookies seguros e desabilitar fallback de trial. |
| `DATABASE_URL` | Sim | `postgres://app:senha@host:5432/accurit` | Conexão PostgreSQL 16. |
| `JWT_SECRET` | Sim | 32+ bytes | Segredo para assinatura de JWT. |
| `CERTIFICATE_MASTER_KEY` | Sim | base64(32 bytes) | Chave mestra para cifrar certificados A1 e credenciais SERPRO. |
| `SERPRO_TOKEN_URL` | Sim | `https://...` | URL do OAuth2 token (produção). |
| `SERPRO_SAPI_AUTH_URL` | Sim | `https://...` | URL de autenticação SAPI. |
| `SERPRO_INTEGRA_TRIAL_URL` | Sim | `https://...` | URL trial do Integra Contador. |
| `SERPRO_INTEGRA_PROD_URL` | Sim | `https://...` | URL produção do Integra Contador. |
| `SERPRO_DIVIDA_ATIVA_TRIAL_URL` | Sim | `https://...` | URL trial da Dívida Ativa. |
| `SERPRO_DIVIDA_ATIVA_PROD_URL` | Não | `https://...` | URL produção da Dívida Ativa (pendente de confirmação SERPRO). |
| `SERPRO_TRIAL_BEARER_TOKEN` | Não | `Bearer ...` | Token bearer do trial (opcional; sem ele, trial falha fechado). |
| `SERPRO_CONSUMER_KEY` | Não | `...` | Consumer key para OAuth2 produção. |
| `SERPRO_CONSUMER_SECRET` | Não | `...` | Consumer secret para OAuth2 produção. |
| `SERPRO_CONTRATANTE_CNPJ` | Não | `12345678000199` | CNPJ da Software House (Contratante no modelo C). |
| `SERPRO_CONTRATANTE_NOME` | Não | `Nome da Software House` | Razão social da Software House. |
| `SERPRO_OPERATOR_HEADER_ENABLED` | Não | `true` | Envia header `x-cpf-usuario` quando habilitado. |
| `DIVIDA_ATIVA_FALLBACK_CODIGO_CONTRIBUINTE` | Não | `false` | Habilita fallback de Código de Contribuinte (D2). |
| `AUTH_COOKIE_SECURE` | Sim | `true` | Deve ser `true` em produção. |

## 3. Scripts e Documentação para o Escritório

Consulte a pasta [`scripts/`](../scripts/) para:

- **`cadastro-certificado-a1.md`** — Passo a passo para o escritório cadastrar o certificado A1 no portal.
- **`credenciais-serpro.md`** — Como o escritório insere as credenciais SERPRO (consumer key/secret, certificado A1).
- **`validacao-procuracao.md`** — Como validar uma procuração junto à SERPRO (trial e produção).
- **`troca-trial-producao.md`** — Procedimento seguro de troca de Trial para Produção, com confirmação e auditoria.

## 4. Contagem de Chamadas e Cobrança

A contagem de chamadas SERPRO distingue automaticamente `TRIAL` de `PRODUCAO` no campo `ambiente` da tabela `chamadas_serpro` e nos agregados `agregados_consumo_serpro`. Isso permite:

- Cobrança separada por ambiente (trial gratuito vs produção paga).
- Limites de chamadas/mês por plano de escritório (`assinaturas_escritorio.chamadas_mes_limite`).
- Bloqueio configurável ao exceder o limite (`assinaturas_escritorio.bloqueio_ao_exceder`).
- Aviso automático em 80% e 100% de utilização (via `percentual_utilizacao` no retorno de `registrar_chamada`).

Consulte [`docs/ADR/0018-medicao-uso-serpro.md`](ADR/0018-medicao-uso-serpro.md) para a decisão arquitetural de medição.

## 5. Auditoria

Toda troca de Trial → Produção é auditada na tabela `auditoria_eventos` com ação `INTEGRA_MODO_TRIAL_ALTERADO` (ou equivalente), registrando:

- `escritorio_id` alvo
- `administrador_id` que executou a troca
- `versao_esperada` (concorrência otimista)
- `ip` de origem
- `atualizado_em`

O endpoint `PATCH /api/v1/admin/escritorios/{id}/serpro/trial` exige audiência `ADMIN` (gestor de tenants SaaS).

---

**Próximos passos:** Após confirmação dos itens pendentes (D5, P5, P4, D2/P2/P3), atualizar este documento, remover os bloqueios de produção e habilitar o ambiente de produção para os escritórios contratantes.
