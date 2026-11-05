# Telas, wireframes e fluxos

> **Fase 0 — proposta para aprovação.** Os desenhos são estruturais. Estados que dependam de APIs ausentes não podem ser simulados como sucesso.

## Shells e navegação responsiva

```text
DESKTOP
┌────────────────────┬───────────────────────────────────────────────┐
│ Accurit            │ Breadcrumb     Ambiente  Buscar  Perfil       │
│ Início             ├───────────────────────────────────────────────┤
│ Clientes           │ Título da página                 Ação primária│
│ Consultas fiscais  │ Conteúdo                                        │
│ Procurações        │                                                 │
│ Certificados      │                                                 │
│ Equipe             │                                                 │
└────────────────────┴───────────────────────────────────────────────┘

MOBILE
┌─────────────────────────────────────┐
│ Accurit   Ambiente             Perfil│
│ Breadcrumb                            │
│ Título                    Ação        │
│ Conteúdo empilhado                    │
├─────────────────────────────────────┤
│ Início   Clientes   Consultas   Mais  │
└─────────────────────────────────────┘
```

Desktop permite recolher sidebar; mobile usa quatro destinos frequentes e drawer “Mais”. No Admin, os destinos frequentes são Visão geral e Escritórios; Auditoria, Administradores e Saúde ficam no drawer. Manter ambiente visível no `/app` e no contexto do escritório em `/admin`.

## Login e site institucional

### Site `/`

```text
┌ Accurit                                  Fale conosco ─────────────┐
│ Gestão fiscal com clareza para o seu escritório.                  │
│ [Acessar plataforma]                                               │
│ ...                                                                │
└───────────────────────────────────────────────────────────────────┘
  Rodapé: Accurit       Área do gestor da Accurit → /admin/login
```

- CTA principal **“Acessar plataforma”** leva a `/app/login`.
- O acesso do ADMIN é um link discreto no rodapé; não competir com o CTA principal.
- Remover a chamada “Sou cliente” como se a pessoa atendida fosse o usuário do SaaS. O acesso principal é para profissionais do escritório.
- Remover textos “Fase 2 · …” do rodapé.

### Login `/admin/login`

```text
┌ Accurit | Acesso administrativo ────────────────────┐
│ Gestor da Accurit                                    │
│ E-mail [____________________________]               │
│ Senha  [____________________________]               │
│ [Entrar]                                             │
└──────────────────────────────────────────────────────┘
```

Somente ADMIN. Não pedir slug de escritório. Erros de credencial são uniformes; loading bloqueia reenvio; preservar `returnUrl` apenas quando interno e válido. Informar quando o usuário não tem acesso ao backoffice sem revelar se a conta existe.

### Login `/app/login`

```text
┌ Accurit | Plataforma do escritório ──────────────────┐
│ Acesse a plataforma do seu escritório                │
│ Escritório [________________________]                │
│ E-mail     [________________________]                │
│ Senha      [________________________]                │
│ [Entrar]                                             │
└──────────────────────────────────────────────────────┘
```

CLIENTE/OPERADOR. Não rotular como portal para o cliente PF/PJ atendido. Remover o link enganoso “Acesse o Painel do Escritório” para `/admin/login`; acesso de gestão da Accurit permanece apenas no rodapé do site. `/login` continua redirecionando para `/app/login`.

**Estados comuns:** carregando, inválido com associação ao campo, erro de autenticação uniforme, bloqueio temporário, sessão expirada, usuário autenticado e acesso negado. Foco vai para o título da mensagem de erro relevante.

## Início do escritório `/app/inicio`

```text
Início                                      [Demonstração (Trial)]
Bom dia, {nome}
┌ Pendências ───────────────────────────────────────────┐
│ Certificado: vence em X dias / não configurado       │
│ Consultas: resultado indeterminado (se houver dado)  │
│ Procuração: ausente/não verificável (se confirmado)  │
└──────────────────────────────────────────────────────┘
[Cadastrar cliente] [Consultar Dívida Ativa]
```

- Somente exibir pendência sustentada por resposta persistida/endpoint real. Enquanto não houver API, usar estado de recurso não disponível, sem inventar tarefa.
- Certificado próximo do vencimento pode gerar aviso; não mostrar arquivo nem senha.
- Aviso permanente identifica ambiente Trial e ressalva que o dado pode ser demonstrativo.
- Estado loading usa skeleton; indisponibilidade da plataforma fica separada de estado vazio.

## Visão geral ADMIN `/admin/dashboard`

```text
Visão geral
┌ Escritórios ativos ┐ ┌ Trial ┐ ┌ Produção ┐ ┌ Saúde técnica ┐
└───────────────────┘ └───────┘ └──────────┘ └───────────────┘
Atividade recente (se houver endpoint)              [Ver auditoria]
```

Não exibir KPI inventado nem expor dado fiscal. Até endpoints de auditoria/health analítica existirem, manter o bloco ausente ou indicar que não há fonte disponível. O health check técnico não é métrica de negócio.

## Clientes: lista, cadastro e ficha

### Lista `/app/clientes`

```text
Clientes atendidos                         [Novo cliente]
[Buscar por nome ou documento.........................]
[Todos] [CPF] [CNPJ · MEI] [Simples] [Presumido] [Real] [Limpar]
┌ Nome                 Tipo  Documento     Regime        Ações ┐
│ ... (tabela desktop; cards abaixo de 768 px)                 │
└──────────────────────────────────────────────────────────────┘
 [Anterior] Página N de M [Próxima]
```

- Os chips refletem `todos`, `cpf`, `cnpj_mei`, `cnpj_simples`, `cnpj_presumido`, `cnpj_real`; filtro, busca e página ficam sincronizados com query params internos e podem ser restaurados ao voltar.
- Documento fiscal deve usar o formato recebido, preservando mascaramento da API; não incluir documento em URL, telemetria ou log.
- Ordenação e paginação têm feedback claro; manter filtro ao navegar de página.
- Loading: skeleton com a forma da tabela. Vazio: CTA de cadastro se permitido. Erro: ação Recarregar + `requestId` copiável. Busca sem resultados: botão limpar filtros.
- Ações de cadastro/edição respeitam o papel que o backend realmente autoriza; divergência atual está documentada separadamente.

### Cadastro/edição `/app/clientes/novo` e `/app/clientes/:id/editar`

```text
Novo cliente atendido
Tipo: [Pessoa física | Pessoa jurídica]
Documento [________________]   Porte/regime [Opcional para PJ]
Nome/Razão social [________________________]
Nome fantasia [___________] E-mail [________] Telefone [_____]
[Cancelar]                                      [Salvar cliente]
```

- Porte/regime é opcional para PJ; para PF, não se aplica.
- Máscara não substitui validação de dígitos; erro fica junto ao campo.
- Não limpar dados no erro de rede; bloquear envio duplicado; em sucesso informar que o cadastro foi salvo.
- Atualização parcial e semântica de `null` exigem mapper compatível com o contrato vigente.

### Ficha `/app/clientes/:id`

```text
Clientes / {nome}
{Nome ou razão social}     Documento mascarado    [Editar, se permitido]
[Resumo] [Consultas fiscais] [Procurações] [Histórico]
```

- Resumo: dados cadastrais mínimos e status; ocultar dados não necessários.
- Consultas fiscais: histórico por serviço e ambiente; consulta Dívida Ativa somente leitura.
- Procurações: status e razão de bloqueio retornados pelo backend; “não verificável” não vira “válida”.
- Histórico: linha do tempo de consulta/auditoria disponível.
- Ações rápidas: “Consultar Dívida Ativa” e “Verificar procuração” apenas quando endpoint e permissão suportarem o contexto do cliente.
- Estado sem cliente/ID não autorizado não revela a existência de outro tenant.

## Dívida Ativa: consulta, detalhes e histórico

### Consulta `/app/consultas/divida-ativa`

```text
Consulta Dívida Ativa                         [Demonstração (Trial)]
Consulta exclusivamente informativa.
Esta tela não negocia, parcela, cobra, paga nem emite DARF.
CPF/CNPJ [________________________] [Consultar]
```

Estados:

- **Carregando:** texto de progresso e botão bloqueado; evitar tentativa dupla.
- **Com pendências:** lista com situação, número mascarado conforme API e valores como string pt-BR.
- **Sem pendências:** somente quando a resposta válida e inequívoca indicar lista vazia.
- **Não localizado:** estado diferente de lista vazia; explicar que não confirma ausência.
- **Não autorizado / indisponível / limite / resposta inválida:** `ErrorState`, ação segura e `requestId`.
- **Indeterminado:** explicar que o status ainda não foi confirmado; não repetir automaticamente.
- **Trial:** banner persistente informa caráter demonstrativo.

### Detalhe `/app/consultas/divida-ativa/inscricoes/:numero`

Mostrar ambiente, hora, chave mascarada, fonte e campos da inscrição retornados pela API. Não criar ação de cobrança, regularização, pagamento, parcelamento ou emissão de documento. Moeda continua string formatada conforme contrato, sem conversão para ponto flutuante.

### Histórico

Filtrar por cliente, serviço, ambiente e período somente quando API fornecer esses dados. Para ausência de histórico, mostrar EmptyState; erro não deve ser apresentado como “sem consultas”.

## Integra Contador `/app/consultas/integra-contador`

```text
Integra Contador
Serviços SERPRO em preparação
Catálogo e autorização ainda não confirmados para uso.
Nenhuma declaração ou documento será transmitido.
[Consultar status disponível] (somente se o backend confirmar operação permitida)
```

A feature deve ficar em estado informativo/“em breve” até a confirmação de contrato e allowlist. Não exibir CTA mutável nem tratar consulta de procuração como prova de autorização. Se uma consulta permitida existir, separar autorização, transporte, resposta de negócio e processamento assíncrono; polling/repetição só quando contrato e backend autorizarem.

## Procurações `/app/procuracoes`

Listar contribuinte mascarado, procurador mascarado, códigos cobertos, validade, situação, ambiente e última verificação, se confirmados pela API. Estados possíveis: válida, ausente, vencida, cancelada, não verificada, não verificável e erro de transporte. Nunca inferir validade de cache expirado ou resposta não interpretada. Não há endpoint de consulta genérica de procurações na API base atual; não liberar uma tela operacional até contrato estar definido.

## Certificados A1 `/app/certificados`

```text
Certificados A1                            [Adicionar certificado]
┌ Ativo  Válido até dd/MM/yyyy  Faltam N dias   [Detalhes] ┐
└──────────────────────────────────────────────────────────┘
```

- Upload: selecionar arquivo, informar senha em campo temporário, revisão de titular/validade, enviar, aguardar validação do backend, descartar arquivo/senha da memória de UI após conclusão.
- Detalhe: titular mascarado, fingerprint apenas quando necessário, período de validade e estado de validação; nunca mostrar conteúdo, senha ou chave.
- Erros de arquivo, senha, criptografia e certificado vencido são distintos e acionáveis.
- Revogação exige `ConfirmDialog` com impacto, papel autorizado e confirmação clara; não confundir revogação com exclusão de registro.
- Tela pode usar a API existente por adaptador snake_case descrito em [UI-CONTRATOS-DIVERGENTES.md](../UI-CONTRATOS-DIVERGENTES.md).

## Equipe `/app/equipe`

Visível somente ao CLIENTE. Lista de usuário, papel, status e última atividade, com criação/convite, edição e desativação apenas após endpoint e política de autorização serem definidos. Não coletar nem exibir senha; convite deve ser fluxo de credencial temporária apenas se a API suportar. Sem API de equipe hoje, manter a feature fora do menu até existir contrato.

## Backoffice: escritórios

### Lista `/admin/escritorios`

Tabela/card com nome, slug, ambiente, versão e atualização; paginação; botão “Novo escritório”. O toggle atual permanece pessimista. Falha de lista mostra ação Recarregar. Em telas pequenas, cada escritório é card com status e ação claramente separada.

### Detalhe `/admin/escritorios/:id`

```text
Escritórios / {nome}                             [Demonstração (Trial)]
[Resumo] [Ambiente SERPRO] [Credenciais] [Certificado] [Usuários] [Auditoria]

Pré-voo para produção
[ ] Credencial INTEGRA_CONTADOR / PRODUÇÃO presente e válida
[ ] Certificado A1/e-CNPJ disponível e dentro da validade
[ ] Contrato/ambiente de produção confirmado e liberado
[ ] Política de auditoria e cache pronta
Produção indisponível — itens pendentes explicam o bloqueio.
```

- Checklist é somente visualização derivada de API; não validar credencial no browser.
- Enquanto a URL/contrato SERPRO ou D5 estiver pendente, Produção fica indisponível. Não oferecer clique que falhará inevitavelmente.
- Quando todos os pré-requisitos forem confirmados, alternar com `ConfirmDialog`: explicar acesso a dados reais, invalidação de cache e requisições em andamento; exigir digitação exata do slug.
- Toggle não muda visualmente antes de resposta de sucesso. Em `409`, recarregar o estado e avisar que o escritório foi alterado por outra operação; em erro, manter valor anterior. Nunca fazer fallback silencioso.
- Credenciais e certificado mostram somente status e metadados seguros; nenhum segredo copiável.
- Abas sem endpoint atual ficam desabilitadas/ocultas com mensagem clara; não preencher com dados demonstrativos.

### Provisionamento `/admin/escritorios/novo`

Formulário: slug, nome do escritório, e-mail/CPF/senha inicial do titular, conforme contrato existente. Validar formato, força mínima de senha e estados de duplicidade sem enumerar dados. Não armazenar senha em URL/telemetria. Confirmar sucesso sem ecoar senha ou documento completo.

### Administradores, auditoria global e saúde

- Administradores SaaS: lista e gestão somente quando API definir operações, confirmação e auditoria.
- Auditoria: filtros por tenant, ator, ação e período; mostrar antes/depois redigidos sem tokens, certificados, payload fiscal ou documentos integrais.
- Saúde: separar disponibilidade de API/banco de estado dos produtos SERPRO. Exibir timestamp e fonte; “saúde OK” não significa que consultas fiscais estão autorizadas.
- Enquanto endpoints não existirem, manter as páginas planejadas no mapa, mas não apresentar conteúdo fictício.

## Estados transversais obrigatórios

| Estado | Padrão de apresentação |
| --- | --- |
| Loading | Skeleton estrutural; controles desabilitados somente durante a operação correspondente |
| Vazio | Explicar por que não há itens e oferecer CTA permitido |
| Erro | Texto seguro, categoria, ação de recuperação e `requestId` copiável |
| Sem permissão | Página/estado explícito; não redirecionar silenciosamente nem revelar dado |
| Offline/indisponível | Preservar formulário/dados em memória quando seguro; oferecer recarregar |
| Indeterminado | Dizer que resultado não foi confirmado; consultar status antes de nova ação |
| Trial | Badge e aviso persistentes; não usar palavras ou cores que sugiram dado real |
| Resposta vazia válida | “Sem pendências/sem registros” somente quando resposta contratual provar que é vazio |
