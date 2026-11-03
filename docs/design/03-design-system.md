# Design system Accurit

> **Fase 0 — proposta para aprovação.** Tokens e componentes serão implementados depois da aprovação. Os valores são uma base inicial; validar contrastes automatizados antes de publicar.

## Fundamentos visuais

- **Direção:** produto fiscal confiável, discreto e legível. Neutros sustentam superfícies; cor é reservada a ação, estado e ambiente.
- **Tipografia:** `Inter`, com fallback para `Segoe UI`, `Arial` e sans-serif do sistema. Não buscar fontes remotas na execução.
- **Escala tipográfica:** 12, 14, 16, 20, 24, 32 e 40 px; corpo padrão 16 px, altura de linha entre 1.45 e 1.6.
- **Espaçamento:** múltiplos de 4 px (`1`=4, `2`=8, `3`=12, `4`=16, `6`=24, `8`=32, `12`=48, `16`=64).
- **Raio:** 4 px para controles densos, 8 px para campos/botões, 12 px para cards, 16 px para diálogos; evitar pílulas em todos os elementos.
- **Sombras:** uma elevação sutil para popover/dialog; separar cards pelo contorno antes de adicionar sombra.
- **Z-index:** conteúdo `0`, sticky `10`, menu/popover `20`, backdrop `30`, diálogo `40`, toast `50`.
- **Motion:** transições de estado entre 120 e 200 ms; evitar movimento decorativo. Sob `prefers-reduced-motion: reduce`, remover deslocamento/animação não essencial.
- **Breakpoints:** 320 px (mínimo suportado), 640 px, 768 px (tabelas passam a cards), 1024 px, 1280 px e 1440 px. Mobile-first; conteúdo não deve exigir rolagem horizontal.

### Tokens CSS e Tailwind v4

Os tokens são a fonte única de cor para Admin, Client e Web. O arquivo compartilhado deve definir CSS custom properties e mapeá-las no `@theme` do Tailwind 4; componentes Angular não devem duplicar valores hexadecimais em estilos locais.

```css
@import "tailwindcss";

:root,
:root[data-theme="light"] {
  color-scheme: light;
  --surface-canvas: #f8fafc;
  --surface-raised: #ffffff;
  --text-primary: #0f172a;
  --text-secondary: #475569;
  --border-default: #64748b;
  --action-primary: #1d4ed8;
  --action-on-primary: #ffffff;
  --state-success: #166534;
  --state-success-surface: #dcfce7;
  --state-warning: #92400e;
  --state-warning-surface: #fef3c7;
  --state-danger: #991b1b;
  --state-danger-surface: #fee2e2;
  --state-info: #075985;
  --state-info-surface: #e0f2fe;
  --env-trial: #92400e;
  --env-trial-surface: #fef3c7;
  --env-producao: #166534;
  --env-producao-surface: #dcfce7;
}

:root[data-theme="dark"] {
  color-scheme: dark;
  --surface-canvas: #0b1220;
  --surface-raised: #111827;
  --text-primary: #f8fafc;
  --text-secondary: #cbd5e1;
  --border-default: #64748b;
  --action-primary: #93c5fd;
  --action-on-primary: #0b1220;
  --state-success: #86efac;
  --state-success-surface: #052e16;
  --state-warning: #fcd34d;
  --state-warning-surface: #422006;
  --state-danger: #fca5a5;
  --state-danger-surface: #450a0a;
  --state-info: #7dd3fc;
  --state-info-surface: #082f49;
  --env-trial: #fcd34d;
  --env-trial-surface: #422006;
  --env-producao: #86efac;
  --env-producao-surface: #052e16;
}

@theme inline {
  --color-canvas: var(--surface-canvas);
  --color-surface: var(--surface-raised);
  --color-text-primary: var(--text-primary);
  --color-text-secondary: var(--text-secondary);
  --color-border: var(--border-default);
  --color-primary: var(--action-primary);
  --color-success: var(--state-success);
  --color-warning: var(--state-warning);
  --color-danger: var(--state-danger);
  --color-info: var(--state-info);
  --color-env-trial: var(--env-trial);
  --color-env-producao: var(--env-producao);
  --font-sans: "Inter", "Segoe UI", Arial, sans-serif;
}
```

O snippet é uma proposta de nomenclatura, não um tema já validado. O contraste de cada par foreground/background, estados de foco e variantes de tema deve ser medido em teste e atingir WCAG 2.2 AA: 4.5:1 para texto normal, 3:1 para texto grande e componentes gráficos/foco.

### Tema e densidade

- Tema: `sistema` como padrão; opções explícitas claro/escuro. Persistir somente preferência visual não sensível; respeitar `prefers-color-scheme`.
- Densidade: `confortável` padrão; `compacta` para tabelas e usuários que a selecionarem. A densidade compacta não reduz alvo interativo abaixo de 44 × 44 px no mobile.
- Estados de foco devem ter contorno visível independente de cor; não remover `outline` sem substituto equivalente.

## Acessibilidade WCAG 2.2 AA

- Navegação completa por teclado e ordem de foco previsível; foco retorna ao controle que abriu diálogo/menu.
- Foco visível com contraste mínimo 3:1 em relação ao estado adjacente; não pode ficar encoberto por cabeçalhos, banners ou diálogos.
- Alvos móveis com pelo menos 44 × 44 px; espaçamento suficiente para evitar acionamento adjacente.
- Todo campo tem label persistente; erro fica associado por `aria-describedby`; não comunicar estado apenas por cor.
- `aria-live="polite"` para progresso e resultado; `role="alert"` reservado a erro que exige atenção imediata.
- Tabelas têm caption/headers semânticos; no mobile, cards preservam a relação campo/valor.
- `Dialog` prende foco, suporta Escape quando seguro e fornece título/descritivo acessíveis.
- Respeitar zoom/reflow a 320 CSS px, alto contraste do sistema e `prefers-reduced-motion`.
- Realizar auditoria automatizada e verificação manual com teclado/leitor de tela nos fluxos principais.

## Catálogo de componentes

| Componente | Anatomia e variantes | Estados | Usar / evitar |
|---|---|---|---|
| `AppShell` | Sidebar, topbar, conteúdo, nav mobile; variações `admin`/`client` | Carregando sessão, autenticado, acesso negado | Base de toda área autenticada; não duplicar shell por tela |
| `Sidebar` | Marca, grupos, item ativo, recolher | Expandida, compacta, mobile drawer | Navegação primária; não colocar ações destrutivas |
| `Topbar` | Breadcrumb, `EnvBadge`, busca, tema, perfil/sair | Busca fechada/aberta, sessão expirando | Contexto e ações globais; nunca mostrar documento/token |
| `Breadcrumb` | Hierarquia clicável + página atual | Compacto/mobile | Contextualizar rota; não substituir título |
| `CommandPalette` | Campo de busca + grupos de destinos/ações | Aberta, sem resultados, sem permissão | Navegação `Ctrl+K`/`⌘K`; mutações exigem diálogo separado |
| `PageHeader` | Título, descrição curta, ações primárias | Normal, ação indisponível | Cabeçalho consistente; evitar títulos internos de roadmap |
| `Button` | Texto, ícone opcional; primary/secondary/tertiary/danger | Default, hover, focus, busy, disabled | Uma ação primária por contexto; não usar para links simples |
| `IconButton` | Ícone + nome acessível/tooltip | Estados padrão e desabilitado | Ação curta em tabela; evitar ícone sem nome acessível |
| `Input` / `Mask` | Label, controle, hint, erro; CPF/CNPJ/telefone/documento | Pristine, inválido, disabled, busy | Validar e formatar sem mudar dado fonte; não revelar documento inteiro sem necessidade |
| `Select` / `Combobox` | Label, opção selecionada, lista pesquisável | Aberto, vazio, inválido, loading | Select curto; combobox para catálogo grande |
| `Checkbox` / `Switch` | Label, descrição, controle | Marcado, indeterminado, disabled | Switch para configuração binária não destrutiva; não trocar ambiente sem confirmação |
| `FormField` | Label, control, hint, erro e estado obrigatório | Default, erro, success quando validado | Único padrão de formulários; mensagem específica, sem erro só em toast |
| `DataTable` | Caption, cabeçalhos, linhas, ações, ordenação/paginação | Skeleton, vazio, erro, seleção, sem resultados | Desktop para comparação; evitar overflow e colunas sem prioridade |
| `ResponsiveList` | Mesmo dado da tabela como cards semânticos | Skeleton, vazio, erro | Substituir DataTable abaixo de 768 px |
| `FilterChips` | Chips selecionáveis + limpar filtros | Ativo, foco, nenhum filtro | Filtros enumerados e refletidos na URL; não ocultar filtros ativos |
| `Tabs` | Lista de abas + painel | Ativa, foco, conteúdo vazio | Subáreas da ficha/detalhe; query/path opcional para deep link |
| `Card` | Título opcional, corpo, rodapé | Default, selecionável, aviso | Agrupar tarefa relacionada; não empilhar cartões sem hierarquia |
| `StatCard` | Métrica, unidade, período/fonte opcional | Normal, indisponível, sem dados | Apenas métrica com fonte; evitar números fictícios ou sem contexto |
| `Badge` / `Chip` | Rótulo textual com estado | Sucesso, aviso, perigo, neutro, info | Classificação complementar; nunca usar cor sem texto |
| `EnvBadge` | Ícone/label explícito “Demonstração (Trial)” ou “Produção” | Trial, Produção, carregando, desconhecido | Visível em todas as telas `/app` e telas de tenant `/admin`; não deduzir estado local |
| `StatusIntegracao` | Estado + frase acionável e detalhe opcional | Sem pendências, com pendências, não autorizado, limite excedido, indisponível, indeterminado, produção não configurada | Resultado de integração; nunca mapear falha para sucesso/zero pendência |
| `Dialog` / `ConfirmDialog` | Título, impacto, decisão, cancelar/confirmar | Aberto, busy, erro | Ações de risco; produção exige digitar slug e não fecha antes de resposta |
| `Toast` | Mensagem curta temporária | Sucesso não crítico, aviso | Confirmações rápidas; erro importante deve ficar na página |
| `Banner` | Ícone, título, mensagem, ação opcional | Trial, risco, bloqueio, informação | Aviso persistente (ex.: modo demonstração); não ocultar o caráter Trial |
| `Skeleton` | Placeholder com dimensões da estrutura real | Carregando | Carregamento inicial; não usar spinner sem contexto para página inteira |
| `EmptyState` | Ícone, título, explicação, CTA permitido | Sem cadastros, sem resultados | Distinguir vazio válido de falha |
| `ErrorState` | Título, mensagem, ação segura, `requestId` copiável | Rede, autorização, negócio, indeterminado | Erro recuperável; não mostrar stack/response bruta |
| `Stepper` | Etapas nomeadas e progresso | Atual, concluída, bloqueada | Upload/validação de certificado e pré-voo; não apresentar processo não suportado |
| `Timeline` | Horário, ator, ação e metadado minimizado | Ordenada, vazia, indisponível | Auditoria/histórico; não gravar/exibir segredo ou payload fiscal |
| `Checklist` | Pré-requisito, estado, evidência segura, explicação | Aprovado, pendente, bloqueado, não verificável | Pré-voo de produção; nenhum bloqueio pode ser resolvido apenas no front |
| `CopyField` | Valor mascarado ou identificador + copiar | Copiado, erro | `requestId` e IDs não sensíveis; nunca oferecer copiar segredo |
| `Tooltip` | Texto de ajuda associado | Visível por hover/foco | Informação complementar; não conter instrução essencial exclusiva |
| `Avatar` / `UserMenu` | Identidade mínima, papel, preferências, sair | Normal, carregando, menu aberto | Perfil do usuário autenticado; não carregar PII não necessária |

**Stack proposta:** Tailwind CSS v4 para tokens, composição visual e responsividade; Angular CDK para overlays, focus trap, acessibilidade e interações; Angular Material somente atrás de wrappers de `libs/ui` onde ainda não houver equivalente aprovado. Material já está presente e usado diretamente nos apps; a migração será incremental, sem reescrita simultânea. Não adicionar outro framework de componentes.

## Catálogo de mensagens

O envelope interno usa `error.code`, `error.message`, `error.requestId` e `error.details`. O código determina categoria e próximo passo; a mensagem do backend pode ser apresentada se for segura, mas detalhes técnicos nunca são renderizados.

| Código/categoria | Título sugerido | Texto e ação | Severidade |
|---|---|---|---|
| `VALIDACAO` | Revise os dados | Indicar o campo ou regra inválida; corrigir e tentar novamente | Aviso |
| `CONFLITO` / HTTP `409` | Informação desatualizada | Recarregar a tela; em troca de ambiente, mostrar o estado atual do servidor | Aviso |
| `MUITAS_REQUISICOES` / HTTP `429` | Muitas tentativas em pouco tempo | Aguardar o período indicado; não oferecer repetição em loop | Aviso |
| `AUTH_INVALID_CREDENTIALS` / `AUTH_*` | Não foi possível entrar | Mensagem uniforme; conferir dados ou recuperar acesso pelo fluxo disponível, sem indicar qual campo está correto | Perigo |
| `PROIBIDO` / HTTP `403` | Acesso não autorizado | Explicar que o perfil não pode executar a ação; orientar contato com titular/gestor apropriado | Perigo |
| `SERPRO_PROD_NOT_CONFIGURED` | Produção não configurada | Produção indisponível; não sugerir mudar para Trial como fallback | Bloqueio |
| `SERPRO_TRIAL_NOT_CONFIGURED` | Demonstração indisponível | Informar indisponibilidade do ambiente Trial e oferecer tentar novamente mais tarde | Aviso |
| `SERPRO_UNAUTHORIZED` | Integração não autorizada | Falha de autenticação SERPRO; encaminhar ao administrador, sem expor credencial | Perigo |
| `SERPRO_PROXY_DENIED` | Procuração não autorizada | Não executar a operação; revisar autorização no canal oficial | Perigo |
| `SERPRO_RATE_LIMITED` | Limite de consultas atingido | Aguardar `Retry-After`, se presente; não reenviar mutação | Aviso |
| `SERPRO_NOT_FOUND` | Informação não localizada | Não interpretar como ausência de pendências; revisar identificador/fonte | Informação |
| `SERPRO_BAD_RESPONSE` | Não foi possível validar a resposta | Resultado não confirmado; tentar mais tarde ou encaminhar `requestId` | Perigo |
| `SERPRO_INDETERMINATE` | Resultado ainda não confirmado | Consultar status antes de repetir; nunca repetir `Declarar`/`Emitir` automaticamente | Bloqueio |
| `SERPRO_UNAVAILABLE` | Serviço temporariamente indisponível | Aguardar e tentar novamente apenas quando seguro | Aviso |
| `BANCO_INDISPONIVEL` / erro de rede | Plataforma temporariamente indisponível | Recarregar ou tentar mais tarde | Perigo |
| Código desconhecido | Não foi possível concluir | Mostrar mensagem segura genérica e `requestId` copiável | Perigo |

Mapear também o código técnico da resposta SERPRO para categorias internas antes de apresentar mensagem. Texto e ação devem respeitar o estado concreto do serviço; não transformar HTTP `404` externo em “sem pendências”.

## Escrita de UI e glossário

- Tom: direto, respeitoso e neutro; explicar consequência, não culpar usuário.
- Rótulos: “Escritório”, “Cliente atendido”, “Titular do escritório”, “Operador”, “Gestor da Accurit”, “Demonstração (Trial)”, “Produção”.
- Usar “procuração” para autorização de representação; “certificado A1” para credencial criptográfica.
- “Em breve” significa indisponível para operação, não uma autorização pendente do usuário.
- Evitar em interfaces: “Fase 2A”, “Fase 2 · …”, “tenant”, “payload”, “gateway”, “mock”, “SERPRO falhou” sem contexto.
- Nunca dizer “sem pendências” quando a consulta está em erro, indisponível, não autorizada ou indeterminada.
- Valores monetários são strings pt-BR da API; formatar apenas conforme contrato, nunca converter para `float`.
- Datas são apresentadas em `dd/MM/yyyy HH:mm`, com indicação do fuso quando necessário.
