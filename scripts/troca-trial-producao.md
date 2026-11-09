# Troca Segura de Trial para Produção (scripts/troca-trial-producao.md)

> Procedimento seguro para o gestor de tenants SaaS alterar o modo de um
> escritório de `TRIAL` para `PRODUCAO`, com confirmação e auditoria.

## Visão geral

A troca de Trial para Produção é uma operação sensível que:
1. Exige autenticação como gestor SaaS (audiência `ADMIN`).
2. Usa concorrência otimista por versão (`versao`).
3. Bloqueia a troca se faltarem credenciais de produção ou certificado A1 válido.
4. Registra auditoria completa (`auditoria_eventos`).

## Pré-requisitos

1. Credenciais de produção do Integra Contador configuradas para o escritório
   (`SERPRO_CONSUMER_KEY` / `SERPRO_CONSUMER_SECRET` ou credencial por escritório).
2. Certificado A1 válido cadastrado (para mTLS em produção, P5).
3. URL de produção configurada (quando aplicável, D5/P4/P5).

## Passo a passo

### 1. Acessar a tela de administração de tenants

No backoffice SaaS, navegue para:
```
Administração > Escritórios > {escritório} > SERPRO > Modo de operação
```

Ou diretamente pela rota:
```
PATCH /api/v1/admin/escritorios/{id}/serpro/trial
```

### 2. Informar os dados

O corpo da requisição deve conter:
```json
{
  "modoTrialSerpro": false,
  "versao": <versão atual do escritório>
}
```

- **`modoTrialSerpro`**: `false` para mudar para Produção, `true` para voltar ao Trial.
- **`versao`**: versão atual do escritório (concorrência otimista; obtida via `GET /api/v1/admin/escritorios/{id}`).

### 3. Confirmação

O sistema valida:
- O administrador tem audiência `ADMIN`.
- O escritório existe.
- A versão confere (concorrência otimista; se diverge, retorna 409 Conflito).
- Se mudando para Produção: credencial de produção do Integra Contador está presente e certificado A1 está em validade. Caso contrário, retorna erro explícito (`ProducaoBloqueada`).

### 4. Resultado

Em caso de sucesso, retorna:
```json
{
  "modoTrialSerpro": false,
  "ambienteEfetivo": "PRODUCAO",
  "versaoAtual": <nova versão>,
  "atualizadoEm": "2026-11-09T12:00:00Z"
}
```

### 5. Auditoria

A troca é registrada em `auditoria_eventos` com:
- `acao`: alteração de modo Trial/Produção
- `escritorio_id`: alvo da troca
- `administrador_id`: quem executou
- `ip`: origem da requisição
- `valor_novo` / `valor_anterior`: estados antes e depois

## Bloqueios de Produção

A troca para Produção é **bloqueada** quando:

| Bloqueio | Motivo |
|----------|--------|
| D5 | URL de produção da Dívida Ativa não confirmada (chamada bloqueada com `DividaAtivaProducaoBloqueada`). |
| P5 | mTLS/e-CNPJ do Integra Contador não validado em homologação. |
| Credencial ausente | `SERPRO_CONSUMER_KEY`/`SECRET` não configurados. |
| Certificado inválido | Certificado A1 expirado, revogado ou não encontrado. |

---

**Importante:** A troca para Produção deve ser feita apenas após confirmação de
todos os itens do checklist de go-live (`docs/GO-LIVE.md`). Em caso de dúvida,
manter o escritório em modo `TRIAL`.
