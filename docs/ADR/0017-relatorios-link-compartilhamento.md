# ADR-0017 — Consolidado em ADR-0016 (Fase F)

Status: consolidado

## Contexto

Esta decisão foi consolidada em **ADR-0016 — Relatórios e link seguro de compartilhamento (Fase F)**. Não há conteúdo duplicativo independente.

## Decisão

Consulte a **ADR-0016** para o registro completo sobre:
- Armazenamento de relatórios em `relatorios` table com conteúdo BYTEA.
- Links seguros e expiráveis via `links_compartilhamento_relatorios` table.
- Isolamento por `escritorio_id` em todas as operações.

## Referências

- [ADR-0016](0016-relatorios-link-compartilhamento.md) — Decisão unificada sobre relatórios e links compartilhados.
- [SPEC §8.9](SPEC.md) — Relatórios e link seguro de compartilhamento (Fase F).
- [SPEC §9.20–9.21](SPEC.md) — Tabelas `relatorios` e `links_compartilhamento_relatorios`.
- [SPEC §1.3](SPEC.md) — Fora de escopo: portal autenticado para o cliente PF/PJ.