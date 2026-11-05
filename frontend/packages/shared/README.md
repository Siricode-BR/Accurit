# @accurit/shared

Validações e formatadores puros do Accurit — sem dependências externas e sem tocar em APIs de browser, por isso servem tanto a apps Angular quanto Server Components/route handler do Next.

## Conteúdo

- `validators/`: `cpf.ts`, `cnpj.ts`, `document.ts`, `email.ts`
- `formatters/`: `document.ts`, `phone.ts`, `currency.ts`

## Uso

```ts
import {
  isValidDocument,   // valida CPF ou CNPJ (com dígito verificador)
  formatDocument,    // '52310175098' -> '523.101.750-98'
  sanitizeDocument,  // remove máscara e não-dígitos
  isValidEmail,
  formatPhone,
} from '@accurit/shared';

sanitizeDocument('523.101.750-98'); // '52310175098'
```

## Instalação

Consumo via workspaces do pnpm: `"@accurit/shared": "workspace:*"`.

Exporta TypeScript fonte ESM (`exports -> src/index.ts`) — nenhum step de build; o bundler de quem consome (esbuild do Angular CLI, Next/Turbopack) transpila.
