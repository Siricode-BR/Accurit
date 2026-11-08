# ADR-0013 — Catálogo configurável de serviços do Integra Contador e persistência de procurações

Status: aceito

## Contexto

A Fase 5A introduz a persistência de procurações e-CAC (`procuracoes`) e um catálogo de serviços do Integra Contador configurável pelo administrador (`catalogo_servicos_integra`). Antes, `ProcuracaoNaoVerificavel` era um stub que sempre retornava `Ok(false)`, bloqueando qualquer serviço que exigisse verificação de procuração quando `autorPedidoDados != contribuinte`. O catálogo era estático (referência em `domain/integra_contador/servicos.rs`) e não podia ser ajustado por operação sem deploy.

Precisávamos de:

1. Um catálogo **autoritativo no banco de dados**, para que o administrador possa habilitar/desabilitar serviços sem redeploy e sem que um serviço desabilitado no banco volte a ser acessível por fallback estático.
2. Uma tabela `procuracoes` persistente, escritório-scoped, com cache de validade curta (TTL) e persistência de tentativas (inclusive falhas), para que o status de procuração sobreviva a reinicializações e seja consultável pela UI.
3. Status de procuração por serviço na UI, com mapeamento dos estados do SERPRO para status persistidos.
4. Regras de bloqueio por verbo (ADR-0013 / D4): `Consultar` de parcelamentos agora permitido (decisão do usuário sobrescreve o bloqueio geral do ADR-0002/D4); `Emitir` permanece bloqueado; tudo ainda desabilitado pendente de confirmação de `versaoSistema` (P4).

## Decisão

### 1. Catálogo no banco de dados (`catalogo_servicos_integra`)

- Tabela `catalogo_servicos_integra` com colunas: `id_sistema`, `id_servico`, `versao_sistema`, `verbo`, `habilitado BOOLEAN NOT NULL`, `versao_confirmada BOOLEAN NOT NULL`, `regra_procuracao`, `criado_em`, `atualizado_em`.
- O banco é **autoritativo** quando um provedor de catálogo está configurado: não há fallback para o catálogo estático. Um serviço desabilitado no banco permanece inacessível mesmo que esteja habilitado estaticamente.
- Fallback estático somente quando o provedor é `None` (testes antigos).
- `resolver`: linha ausente/desabilitada → `ServicoNaoHabilitado`; `versao_sistema` ausente ou divergente do informado → `VersaoNaoConfirmada`.
- `definir_habilitado` é fail-closed: não habilita quando `versao_confirmada` é `false`.
- Apenas `OBTERPROCURACAO41` (`versao "1"`, verbo `Consultar`) e `ENVIOXMLASSINADO81` (`versao "1.0"`, verbo `Apoiar`) estão confirmados. `ENVIOXMLASSINADO81` fica com `habilitado=false` no catálogo (uso interno exclusivo via fluxo `AUTENTICAPROCURADOR`).

### 2. Persistência de procurações (`procuracoes`)

- Tabela `procuracoes` ganha `cliente_id UUID REFERENCES clientes(id) ON DELETE SET NULL` (além de `escritorio_id`, `ambiente`, `contribuinte_cpf_cnpj`, `procurador_cpf_cnpj`, `codigo_servico`, `status`, `valido_de`, `valido_ate`, `verificado_em`, `origem`).
- `ProcuracaoNaoVerificavel` é substituído por `ValidadorProcuracaoDiesel`, implementação Diesel que consulta a tabela `procuracoes` e/ou dispara `consultar_procuracoes` (`ExecutarIntegraContadorUseCase::consultar_procuracoes`), gravando o resultado com `ValidacaoProcuracao::{e_valida_agora, cobre_regra}` (fail-closed).
- Status persistidos: `CONCLUIDO → VALIDA`, `FALHA_AUTORIZACAO → NAO_AUTORIZADA`, `FALHA_TRANSPORTE/INDETERMINADO → INDETERMINADA` (tentativa persistida). `ConfiguracaoIncompleta` propaga sem persistir.
- `codigo_servico` pode ser `NULL` para serviços sem regra de procuração (linhas agregadas); o formato de resposta de `OBTERPROCURACAO41` não está documentado em `docs/Serpro`, então aquelas linhas são persistidas como agregadas com `payload_redigido` JSONB bruto (ver P7).
- Chave única: `(escritorio_id, ambiente, contribuinte_cpf_cnpj, procurador_cpf_cnpj, codigo_servico)` — quando `codigo_servico` é NULL, a unicidade vale para a combinação sem ele.

### 3. Fluxo `validar` / `revalidar`

- `validar` = fresh-check com TTL (`SERPRO_PROCURACAO_CACHE_TTL_SECS`, padrão 300 s): se há registro válido dentro do TTL, retorna fonte `"CACHE"`; caso contrário chama o SERPRO, persiste (fonte `"SERPRO"`) e retorna o estado.
- `revalidar` = força nova chamada ao SERPRO, ignorando o TTL.
- `salvar` = UPDATE-then-INSERT com `RETURNING` via `COLUNAS_PROCURACAO` const + `IS NOT DISTINCT FROM`, evitando `ON CONFLICT`.
- `existe_valida` conta via `TotalRow`, com cutoff de TTL computado em Rust (`Nullable<Timestamptz>`).

### 4. UI — status de procuração por serviço

Status da UI: `NAO_EXIGIDA` / `AUTORIZADA` / `CONSULTADA` / `PENDENTE`.
Mapeamento persistido → UI: `VALIDA → AUTORIZADA`, `NAO_AUTORIZADA → PENDENTE`, `INDETERMINADA → PENDENTE`, ausente → `PENDENTE`.

### 5. `ContextoIntegraContador` com `escritorio_id`

Ganha campo `escritorio_id: Uuid` (apenas dois locais de construção: definição em `ports.rs` e resolver Diesel), para que o validador seja escritório-scoped.

### 6. `ResultadoIntegraContador` com `dados_resposta`

Ganha `dados_resposta: Option<String>`; `mapear_resultado` agora recebe `&str` (referências).

### 7. Caso de uso com catálogo injetado

`ExecutarIntegraContadorUseCase::com_catalogo(self, provedor: Option<Arc<dyn ProvedorCatalogoIntegra>>) -> Self` consome `self` pré-`Arc` (call site: `Arc::new(UseCase::novo(...).com_catalogo(Some(...)))`).

### 8. Rotas administrativas

`construir_admin` sem middleware `resolver_ambiente` (apenas `ExigirAudiencia::Admin`), aninhado como `/admin/integra-contador`. O handler valida existência contra o catálogo de referência do domínio → `404 NAO_ENCONTRADO` para sistemas desconhecidos.

### 9. Erros

- `AutenticarProcuradorUseCase`: erro de repositório → `Persistencia`; cliente ausente → `ClienteNaoEncontrado` (→ HTTP 404 `NAO_ENCONTRADO`).
- `mapear_erro` mapeia `ClienteNaoEncontrado` → 404 `NAO_ENCONTRADO`.

## Consequências

- Positivo: o administrador pode desabilitar serviços no banco sem deploy; o bloqueio fail-closed impede bypass via catálogo estático; o status de procuração persiste e é consultável pela UI; o isolamento por `escritorio_id` é mantido.
- Negativo: o formato de resposta de `OBTERPROCURACAO41` não está documentado oficialmente (P7), então linhas agregadas com `codigo_servico NULL` e `payload_redigido` JSONB bruto precisam de revisão quando o documento sair.
- Serviços adicionais (além de `OBTERPROCURACAO41` e `ENVIOXMLASSINADO81`) permanecem `AguardandoConfirmacao` até confirmação de `versaoSistema` (P4).

## Referências

- [ADR-0002](ADR/0002-escopo-consultivo-divida-ativa.md) — D4, bloqueio de parcelamentos.
- [ADR-0007](ADR/0007-modelo-contratacao-serpro.md) — modelo de contratação A (contratante = autor).
- [ADR-0010](ADR/0010-revisao-atores-papeis-e-provisionamento.md) — revisão de atores e papéis.
- [SPEC §5.4](SPEC.md) — catálogo e payloads do Integra Contador.
- [SPEC §6](SPEC.md) — procurações e-CAC.
- [SPEC §9.7](SPEC.md) — tabela `procuracoes`.
- [DECISOES-PENDENTES.md](DECISOES-PENDENTES.md) — P4 (`versaoSistema`), P7 (formato `OBTERPROCURACAO41`).
