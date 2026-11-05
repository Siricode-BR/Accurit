# Contratos divergentes entre UI e API

> Inventário para a Fase 0 do design system. A interface de dados deve isolar estas diferenças. **Nenhum contrato de API foi alterado por este documento.**

## Divergências verificadas

| ID | Área | Contrato observado | Expectativa/problema na UI | Proposta sem mudança de API | Owner |
| --- | --- | --- | --- | --- | --- |
| UI-API-01 | Tipo de cliente | `backend/src/domain/clientes/cliente.rs`: enum `TipoCliente::{Pf,Pj}` deriva `Serialize/Deserialize` sem rename explícito. Em JSON, os valores wire são `"Pf"`/`"Pj"`. | `client/src/app/features/clientes/cliente.service.ts` define e envia `TipoCliente = 'PF' | 'PJ'`; resposta também é tratada como maiúscula. O POST/PUT com`"PF"`/`"PJ"` pode falhar na desserialização e a resposta não corresponde ao tipo TS. | Criar DTO wire e mapper em `client/.../clientes/data/`: `PF`/`PJ` no domínio de UI ↔ `Pf`/`Pj` no wire; cobrir request e response com fixtures. Correção backend para `PF`/`PJ` exige decisão e mudança coordenada de contrato. | Frontend Client; Backend para alinhamento futuro |
| UI-API-02 | Paginação de Clientes | `backend/src/presentation/clientes/dto.rs` aceita `pagina`/`tamanho` e aliases `page`/`pageSize`, mas serializa listagem como `pagina` e `tamanho`. `SPEC.md` §10.3 padroniza paginação em `page`/`pageSize`. | O Client envia `pagina`/`tamanho` e espera `pagina`/`tamanho`; funciona contra o wire atual, mas diverge do contrato comum documentado e do contrato Admin (`page`/`pageSize`). | Expor modelo interno `page`/`pageSize`; mapper converte para campos wire `pagina`/`tamanho` e normaliza a resposta. Preservar o endpoint atual; qualquer padronização do backend deve manter compatibilidade ou anunciar breaking change. | Frontend Client; Backend/Arquitetura para revisão de SPEC |
| UI-API-03 | Certificados A1 | `backend/src/presentation/certificados/dto.rs` não usa `rename_all = "camelCase"` nos DTOs. Resposta inclui chaves snake_case como `escritorio_id`, `titular_cpf_cnpj`, `valido_de`, `valido_ate`, `criado_em`, `dias_para_expirar`; request usa campos como `titular_cpf_cnpj`, `conteudo_base64`, `valido_de`. | Não existe feature Angular de certificados ainda; uma futura UI que presuma camelCase não desserializa corretamente. | Criar DTOs wire snake_case e mapper em `client/.../certificados/data/`, mantendo modelo de apresentação camelCase. Enviar a senha apenas no request autenticado e não persistir arquivo/senha no browser. | Frontend Client; Backend se houver correção futura |
| UI-API-04 | Erros e correlação | `backend/src/presentation/error.rs` serializa envelope camelCase com `error.code`, `message`, `requestId` e `details`. | A UI de Dívida Ativa traduz vários códigos, mas não apresenta `requestId` copiável; outros fluxos ainda não têm componente de erro comum. O dado de correlação existe no contrato e está sendo descartado visualmente. | Criar mapper comum para `ApiError` e `ErrorState`; renderizar código seguro/categoria e botão de copiar apenas `requestId`. Nunca exibir `details` sem revisão de segurança. | Frontend Admin/Client |

## Lacunas de capacidade (não inventar endpoint)

As telas a seguir estão no mapa de navegação, mas não foram encontrados endpoints para sustentá-las na API atual. Elas não são divergências de serialização: exigem contrato, autorização e decisão próprios antes de desenvolvimento funcional.

| Capacidade | Situação observada | Recomendação | Owner |
| --- | --- | --- | --- |
| Equipe do escritório | Sem rotas de gestão/listagem de usuários em `backend/src/presentation/routes/mod.rs` | Definir CRUD, convite/credenciais, papel e auditoria; CLIENTE é o único papel com acesso visual proposto | Product + Backend |
| Administradores SaaS | Sem rotas para gerir `administradores` observadas no router principal | Definir provisionamento/recuperação, RBAC, lockout e auditoria antes da UI | Product + Backend |
| Auditoria tenant e global | Persistência existe, mas não há rota de consulta no router principal | Definir filtros, paginação, redação, retenção e escopo antes de expor eventos | Product + Backend + Segurança |
| Detalhe/abas de escritório | API expõe listagem, provisionamento e PATCH do modo Trial; não foi encontrado GET de detalhe/credenciais/usuários/auditoria | Definir leitura por aba e política de exposição; não compor detalhe a partir de campos não listados | Product + Backend |
| Histórico geral de consultas | Dívida Ativa persiste histórico, mas não foi encontrada rota HTTP de listagem no router | Definir filtros e DTO redigido; não consultar diretamente o banco pelo frontend | Product + Backend |
| Procuracoes como área de gestão | Integra Contador tem consulta específica de procurações; não há contrato completo para listar/interpretar estado atual de todos os vínculos | Confirmar contrato oficial, payload, códigos, validade e autoridade do status antes de exibir “válida” | Product + Backend + SERPRO |
| Selo de saúde de negócio | Existe `/api/v1/health` técnico para API/banco; não há agregação de saúde fiscal | Não inferir situação fiscal a partir do health check técnico | Backend |

## Convenções de implementação do mapper

1. Nomear tipos wire explicitamente (`ClienteWire`, `ListagemClientesWire`, `CertificadoWire`) e mantê-los na camada `data`.
2. Modelos de domínio/apresentação usam o vocabulário de UI (`PF`/`PJ`, datas tipadas para apresentação, paginação `page`/`pageSize`).
3. Testar conversão nos dois sentidos com fixtures capturadas de contratos, incluindo `null`, propriedades opcionais e enum desconhecido.
4. Enum desconhecido não recebe default permissivo; mostrar erro de contrato seguro e `requestId`.
5. Não alterar nem normalizar moeda para `float`; preservar strings pt-BR.
6. Tratar campos extras wire de forma compatível sem exibi-los automaticamente. Nunca mapear `escritorioId` em UI onde ele não é necessário.
7. Qualquer ajuste backend que elimine uma divergência precisa ser comunicado, versionado e acompanhado de atualização deste inventário e dos testes.

## Observações de navegação (fora do contrato de API)

- `web/src/app/page.tsx` direciona “Acessar plataforma” e “Sou do escritório” para `/admin/login`, embora esse canal seja exclusivo do papel ADMIN. Proposta: CTA de plataforma para `/app/login`; link discreto de gestor no rodapé para `/admin/login`.
- `web/src/app/page.tsx` chama o usuário do escritório de “Sou cliente”, conflitando com ADR-0010, em que CLIENTE significa o escritório e o cliente PF/PJ atendido não autentica.
- `client/src/app/features/auth/login/login.html` chama `/admin/login` de “Painel do Escritório”; essa descrição confunde gestor SaaS com titular/operador.
- `admin/src/app/features/dashboard/dashboard.ts` existe, mas não está registrado em `admin/src/app/app.routes.ts`; decidir Visão geral como destino do shell e corrigir rota na fase de navegação.
- Rodapés de login/Início exibem rótulos internos de fase; removê-los durante migração visual.
