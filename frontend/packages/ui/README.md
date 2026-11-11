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
| `AccInputComponent` | `acc-input` | Texto/e-mail/senha/número/tel/data/mês com CVA, label, hint, erro, prefixo/sufixo, loading, disabled/readonly e toggle de senha. |
| `AccTextareaComponent` | `acc-textarea` | Área de texto com CVA, label, hint/erro e `linhas`. |
| `AccCheckboxComponent` | `acc-checkbox` | Checkbox booleano com CVA, label/hint/erro. |
| `AccSwitchComponent` | `acc-switch` | Toggle acessível (`role="switch"`) com CVA booleano. |
| `AccRadioGroupComponent` | `acc-radio-group` | Grupo de rádio com CVA (valor string) e opções `{ value, label, hint? }`. |
| `AccTableComponent` | `acc-table` | Tabela declarativa: colunas, ordenação, skeleton, vazio, erro com retry, linhas clicáveis e scroll horizontal. |
| `AccPaginationComponent` | `acc-pagination` | Paginação `pagina`/`tamanho`/`total` com `(paginaChange)`. |
| `AccAlertComponent` | `acc-alert` | Alerta info/sucesso/aviso/erro, descartável. |
| `AccConfirmDialogComponent` | `acc-confirm-dialog` | Confirmação sobre `acc-modal` para ações destrutivas/ativar-desativar. |
| `AccFormFieldComponent` | `acc-form-field` | Wrapper label/obrigatório/hint/erro para campos compostos. |
| `AccFileFieldComponent` | `acc-file-field` | Único wrapper permitido para `<input type="file">` (CVA `File \| null`). |
| `AccDateFieldComponent` / `AccMonthFieldComponent` | `acc-date-field` / `acc-month-field` | Data (`yyyy-mm-dd`) e competência (`yyyy-mm`) com CVA. |
| `AccToastService` / `AccToastHostComponent` | `acc-toast-host` | Toasts efêmeros (`sucesso`/`erro`/`aviso`/`info`); o host já vem na `acc-app-shell`. |

Exemplo de formulário com CVA:

```html
<form [formGroup]="form" (ngSubmit)="salvar()">
  <acc-input label="E-mail" type="email" formControlName="email" [erro]="erroEmail()" />
  <acc-input label="Senha" type="password" formControlName="senha" />
  <acc-select label="Perfil" formControlName="papel" [options]="papeis" [placeholder]="null" />
  <acc-checkbox label="Li e aceito" formControlName="aceite" />
  <acc-switch label="Habilitado" formControlName="ativo" />
  <button acc-button variant="primary" type="submit" [loading]="salvando()">Salvar</button>
</form>
```

Exemplo de tabela + paginação:

```html
<acc-table [colunas]="colunas()" [linhas]="clientes()" [carregando]="carregando()"
           [erro]="erro()" vazio="Nenhum cliente encontrado."
           (tentarNovamente)="carregar()" />
<acc-pagination [pagina]="pagina()" [tamanho]="20" [total]="total()"
                (paginaChange)="carregar($event)" />
```

Colunas com célula customizada usam `template` (um `TemplateRef` com contexto `{ $implicit: linha, linha, valor }`):

```html
<ng-template #status let-linha>
  <acc-badge [status]="linha.ativo ? 'success' : 'danger'">{{ linha.ativo ? 'Ativo' : 'Inativo' }}</acc-badge>
</ng-template>
```

Toasts:

```ts
private readonly toasts = inject(AccToastService);
this.toasts.sucesso('Cliente salvo.');
this.toasts.erro(mensagemDeErro(err, 'Falha ao salvar.'));
```

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
