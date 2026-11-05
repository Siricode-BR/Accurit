# @accurit/core

DTOs de API e tipos de domínio comuns do Accurit, **espelhados manualmente do OpenAPI do backend externo** (sem dependência de build entre frontend e backend).

## Conteúdo

- `auth/`: `LoginRequestDto`, `LoginResponseDto`, `UsuarioSessaoDto`, `AuthSessionState`
- `cliente/`: `ClienteDto`, `ListagemClientesDto`, `CreateClienteRequestDto`, `FiltroClientes`
- `tenant/`: `TenantDto`, `TenantAlteradoDto`, `CriarTenantRequestDto/ResponseDto`
- `serpro/`: `AmbienteSerproInfoDto`, `EstadoAmbiente`
- `divida-ativa/`: tipos da consulta de Dívida Ativa (RS)
- `common/api.ts`: contratos e helpers de paginação/envelope

## Convenções

- Tipagem estrita: `type` + `interface` de payload somente leitura nos DTOs usados em state.
- Mudou o contrato do backend? Atualize aqui primeiro — os `data/services` e features tipam contra `@accurit/core`.

Consumo via `"@accurit/core": "workspace:*"` — TypeScript fonte ESM, sem build.
