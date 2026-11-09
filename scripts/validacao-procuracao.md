# Validação de Procuração (scripts/validacao-procuracao.md)

> Procedimento para o escritório validar uma procuração junto à SERPRO,
> permitindo consultas e operações autorizadas em nome do cliente.

## Pré-requisitos

1. Procuração outorgada pelo cliente ao escritório (via e-CAC ou outro canal oficial).
2. Credenciais SERPRO configuradas (consulte `scripts/credenciais-serpro.md`).
3. Acesso ao portal como usuário com papel `GESTOR` ou `OPERADOR` do escritório.

## Passo a passo

### 1. Registrar a procuração

No portal, navegue para:
```
Clientes > {cliente} > Procurações > Nova procuração
```

Informe:
- **Tipo de documento** (CPF ou CNPJ) e número do outorgante (cliente).
- **Tipo de documento** e número do outorgado (escritório/procurador).
- **Código do serviço** (opcional, ex: `OBTERPROCURACAO41`).
- **Ambiente**: `TRIAL` ou `PRODUCAO`.

### 2. Validar junto à SERPRO

O sistema chama o endpoint `OBTERPROCURACAO41` do Integra Contador para
verificar se a procuração é válida perante a SERPRO. O resultado é:

- ✅ **Válida**: procuração registrada com status `verificada` e período de
  validade (`valido_de` / `valido_ate`).
- ❌ **Inválida**: erro explícito informando o motivo (ex: procuração não
  encontrada, revogada, expirada).

### 3. TTL de validação

A validação da procuração tem TTL configurável
(`SERPRO_PROCURACAO_CACHE_TTL_SECS`, padrão 300 segundos). Após o TTL, o
sistema deve revalidar antes de permitir novas chamadas que dependam da
procuração.

### 4. Falha fechada (fail-closed)

Se a procuração não estiver válida para um serviço específico, chamadas que
exijam essa procuração são rejeitadas com erro explícito (`ProcuracaoNegada`).
Não há fallback silencioso.

---

**Nota:** O formato da resposta de `OBTERPROCURACAO41` não está totalmente
documentado pela SERPRO (pendência P7). As procurações são persistidas como
linhas agregadas (`codigo_servico NULL`) com o `payload_redigido` JSONB bruto.
