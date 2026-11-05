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
