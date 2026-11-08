# ADR-0012 — Gestão de usuários do escritório

Status: aceito

## Contexto
Usuários (`CLIENTE` titular e `OPERADOR`) só existiam via provisionamento. É preciso cadastrar e administrar usuários no portal e no backoffice.

## Decisão
- `usuarios.nome` (`TEXT NOT NULL DEFAULT ''`) adicionado na migration original (sem ALTER, fase de desenvolvimento); exposto em `usuario` de login e `/me`.
- Portal: `/api/v1/usuarios` restrito ao titular `CLIENTE`, sempre escopado ao escritório do token (outro tenant → `404`). Backoffice: `/api/v1/admin/usuarios`, global, com `escritorioId` na criação.
- Mesmo caso de uso para ambos; o `Ator` define o escopo.
- Regras: e-mail único na plataforma (ADR-0011), CPF único por escritório, senha mínima de 8 caracteres (Argon2), ninguém desativa a si mesmo, o último titular ativo não pode ser desativado nem rebaixado (`409`).
- Desativar, trocar papel ou redefinir senha revoga os refresh tokens do usuário; redefinir/reativar zera falhas e bloqueio.
- Auditoria em `auditoria_eventos` (`USUARIO_CRIADO/ATUALIZADO/DESATIVADO/REATIVADO/SENHA_REDEFINIDA`); senhas nunca são registradas.
