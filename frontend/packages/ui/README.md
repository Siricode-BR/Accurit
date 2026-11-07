# @accurit/ui

Design system Angular do Accurit — biblioteca ng-packagr consumida pelos apps `apps/admin` e `apps/client`.

## Componentes

| Export | Uso |
| --- | --- |
| `AccButtonComponent` | botões (`variant` primary/secondary/outline/ghost, `loading`) |
| `AccCardComponent` | cards de conteúdo |
| `AccBadgeComponent` | badges de status (`status` info/neutral/success…) |
| `AccSpinnerComponent` | spinner de carregamento |
| `AccDocumentMaskDirective` | máscara de CPF/CNPJ em inputs (`accDocMask`) |

## Tokens

`tokens.css` (exportado como `@accurit/ui/tokens.css`) define `--acc-color-*`, `--acc-radius-*`, `--acc-shadow-*` e `--acc-font-sans`. Importado em `styles.css`/`globals.css` dos apps — a landing do Next também consome os mesmos tokens.

Consumo via `"@accurit/ui": "workspace:*"`. Este é o único package que exige `build` (`ng build` → FESM em `dist/ui`); o Turborepo garante a ordem via `dependsOn: ["^build"]`.

## Componentes de layout e formulário (compartilháveis entre os apps Angular)

Cada componente novo mantém `.ts`, `.html`, `.css` e `.types.ts` separados.

| Componente | Seletor | Uso |
| --- | --- | --- |
| `AccAppShellComponent` | `acc-app-shell` | Casca da área autenticada: sidebar recolhível (desktop), drawer (mobile) e topbar. Recebe `groups` (`AccNavGroup[]`), `usuario` e emite `sair`. Slot `[acc-shell-topbar]`. |
| `AccSidebarComponent` | `acc-sidebar` | Navegação primária agrupada com item ativo (`routerLinkActive`) e item "Em breve". |
| `AccSelectComponent` | `acc-select` | Select controlado por signal (`[(value)]`), sem `ngModel` — seguro dentro de `<form>`. |
| `AccDocumentFieldComponent` | `acc-document-field` | CPF/CNPJ com máscara e validação; `bloqueado` torna o campo somente leitura. |
| `AccStatCardComponent` | `acc-stat-card` | Indicador (rótulo + valor) para dashboards. |
| `AccPageHeaderComponent` | `acc-page-header` | Título, subtítulo e área de ações da página. |
| `AccIconComponent` | `acc-icon` | Ícones do design system (`AccIconName`). |

Exemplo (app admin reutilizando o shell):

```ts
const NAV: AccNavGroup[] = [
  { id: 'principal', items: [{ id: 'visao-geral', label: 'Visão geral', icon: 'dashboard', route: '/dashboard', exact: true }] },
];
```

```html
<acc-app-shell brand="Accurit" portal="Administração" [groups]="nav" [usuario]="usuario()" (sair)="sair()">
  <router-outlet />
</acc-app-shell>
```

Depois de alterar a lib, recompile com `pnpm --filter @accurit/ui build` (os apps consomem `dist/`).
