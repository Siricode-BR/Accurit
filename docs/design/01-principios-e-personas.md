# Princípios de produto e personas

> **Fase 0 — proposta para aprovação.** Este documento descreve a experiência desejada; não altera código, rotas ou contratos de API.

## Contexto e limites

A Accurit é uma plataforma SaaS B2B para escritórios de contabilidade. Há três canais distintos:

- `/`: site institucional e captação de leads.
- `/app/`: plataforma autenticada do escritório.
- `/admin/`: backoffice SaaS para gestão de tenants.

O ADMIN é gestor da Accurit, não pertence a um escritório e não é um usuário da plataforma do escritório. CLIENTE é o titular do escritório assinante. OPERADOR é membro da equipe desse escritório. O cliente PF/PJ atendido é um registro de negócio, não uma pessoa com login.

Consulta Dívida Ativa permanece estritamente informativa. A UI não pode oferecer negociação, parcelamento, pagamento, cobrança ou emissão de DARF de dívida ativa. Integra Contador aparece como recurso futuro/em breve até que catálogo, contratos e autorizações estejam confirmados.

## Princípios de experiência

1. **Clareza fiscal antes de velocidade.** Diferenciar resultado, ausência de resultado, falha de transporte, falta de autorização e estado indeterminado. Nunca inferir ausência de pendência a partir de erro ou resposta ambígua.
2. **Sempre responder “o que preciso fazer agora?”.** Cada bloqueio deve dizer o motivo conhecido e a próxima ação segura, sem orientar a contornar autorização.
3. **Segurança visível, segredos invisíveis.** Mostrar o ambiente ativo e os limites de acesso; não mostrar token, certificado, chave privada, senha, CPF/CNPJ completo em logs ou payload técnico.
4. **O backend é a fonte de verdade.** O front-end apresenta o ambiente recebido do servidor; não escolhe Trial/Produção em consultas. A troca para produção no backoffice é pessimista, autorizada, auditada, precedida de pré-voo e confirmada explicitamente.
5. **Honestidade sobre incerteza.** Respostas `202`, `204`, `304`, `504`, falta de confirmação de procuração ou serviço indisponível não devem parecer sucesso concluído. Não iniciar repetição automática de declaração/emissão.
6. **Minimização e privacidade por padrão.** Exibir somente os dados necessários à tarefa. Preservar mascaramento entregue pela API e não persistir credenciais no navegador.
7. **Consistência com o domínio.** Usar “escritório” para o tenant; “cliente atendido” para PF/PJ; “gestor da Accurit” para ADMIN. Evitar chamar o titular do escritório ou a pessoa atendida apenas de “cliente” quando houver ambiguidade.
8. **Acessibilidade como comportamento padrão.** Teclado, foco, semântica, contraste, mensagens anunciadas e redução de movimento fazem parte do componente, não são acabamento opcional.
9. **Densidade útil.** Priorizar leitura rápida de status, próximos passos e listas. Evitar dashboards decorativos, números sem fonte e estados de demonstração tratados como dados reais.
10. **Compatibilidade progressiva.** Não alterar contratos de API nesta etapa. Divergências ficam registradas em [UI-CONTRATOS-DIVERGENTES.md](../UI-CONTRATOS-DIVERGENTES.md) e são isoladas em adaptadores de dados até decisão explícita.

## Personas

### Operador de escritório

- **Quem é:** pessoa da equipe que executa rotinas contábeis e mantém os registros dos clientes atendidos.
- **Objetivos:** localizar rapidamente um cliente, manter cadastro correto e consultar informações fiscais autorizadas.
- **Dores:** listas extensas, respostas externas pouco claras, alternância entre sistemas e risco de interpretar Trial como dado real.
- **Necessidades de UI:** filtros persistentes, busca, lista legível, atalhos contextuais, status com horário/ambiente e erro que sugira uma ação segura.
- **Limites:** nunca deve receber privilégio de ADMIN por trabalhar para um escritório; operações que a API negar permanecem negadas no front.

### Titular do escritório (CLIENTE)

- **Quem é:** responsável pelo escritório assinante e pelo acesso da equipe à plataforma.
- **Objetivos:** acompanhar a operação, os ambientes usados e a situação de sua equipe e clientes atendidos.
- **Dores:** falta de visão consolidada, risco de ações realizadas sem autorização e confusão sobre quem pode mudar o ambiente SERPRO.
- **Necessidades de UI:** painel de pendências, acesso a equipe conforme permissões existentes, visibilidade de ambiente e trilha de atividades.
- **Limites:** não alterna Trial/Produção; essa ação pertence ao ADMIN da Accurit, conforme SPEC e ADR-0010.

### Gestor SaaS (ADMIN)

- **Quem é:** operador interno da Accurit que administra tenants e saúde da plataforma.
- **Objetivos:** provisionar escritórios, inspecionar configuração, prevenir ativação de produção sem pré-requisitos e investigar eventos.
- **Dores:** alterações de alto impacto, contextos de múltiplos tenants e risco de trocar o ambiente errado.
- **Necessidades de UI:** seleção explícita do escritório, resumo e checklist antes de qualquer mudança, confirmação com slug, atualização pessimista, auditoria e tratamento de conflito `409`.
- **Limites:** seu token não carrega `escritorio_id`; o alvo deve ser explícito na requisição e validado no backend. A UI não substitui controles de autorização.

## Jornadas críticas

1. **Entrar no canal correto:** lead visita `/`; profissional do escritório entra em `/app/login`; gestor SaaS usa `/admin/login`. Nenhum link deve encaminhar o titular/operador ao login de ADMIN.
2. **Localizar e consultar um cliente:** abrir Clientes, filtrar/buscar, conferir documento mascarado, abrir ficha e iniciar uma consulta somente de leitura. O resultado deve identificar ambiente e horário.
3. **Interpretar Dívida Ativa:** distinguir “nenhuma inscrição retornada” de não encontrado, erro SERPRO, não autorizado e indeterminado; manter aviso de escopo consultivo.
4. **Avaliar uma procuração:** exibir autorização como válida, ausente, vencida, cancelada ou não verificável somente quando o backend fornecer essa conclusão. Resposta de consulta bruta não equivale a autorização concedida.
5. **Criar ou renovar um certificado:** orientar seleção, upload, senha transitória e validação; mostrar metadados de validade sem expor o arquivo ou a senha.
6. **Trocar ambiente no backoffice:** selecionar escritório, entender estado e pré-requisitos, confirmar impacto e slug, aguardar resposta; em conflito, recarregar e explicar que o valor foi alterado por outra operação.
7. **Investigar indisponibilidade:** correlacionar mensagem segura e `requestId`, sem exigir que o usuário copie tokens ou envie payload fiscal.

## Métricas propostas

As metas abaixo são hipóteses para aprovação e validação com usuários; não são SLAs SERPRO.

| Métrica | Definição | Meta inicial proposta |
| --- | --- | --- |
| Tempo para cadastrar cliente atendido | Mediana entre abertura do formulário e confirmação do cadastro, em cenário sem erro externo | Até 2 minutos |
| Tempo para localizar cliente | Mediana entre abrir Clientes e abrir a ficha correta, usando busca/filtro | Até 30 segundos |
| Tempo de resposta percebido da consulta | Mediana e p90 entre envio e resultado; medir separadamente tempo do backend/SERPRO | Apresentar feedback imediato; medir p90 contra timeout configurado, sem ocultar estado indeterminado |
| Erro de troca de ambiente | PATCH com erro, `409` ou pré-requisito não atendido / tentativas totais de troca; separar falha esperada de concorrência | Acompanhar tendência e reduzir erros evitáveis; zero troca silenciosa ou não confirmada |
| Uso de confirmação de risco | Proporção de tentativas de produção que viram confirmação explícita, cancelamento e sucesso | 100% das trocas para produção passam por confirmação |
| Acessibilidade de fluxo | Barreiras bloqueantes encontradas em navegação por teclado, leitor de tela e contraste | Zero barreira bloqueante nos fluxos principais |

Instrumentação deve registrar somente métricas agregadas e eventos mínimos, sem documento fiscal, conteúdo do certificado, senha, token ou payload SERPRO.
