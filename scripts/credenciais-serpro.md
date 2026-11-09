# Credenciais SERPRO (scripts/credenciais-serpro.md)

> Procedimento para o escritório inserir e gerenciar as credenciais SERPRO
> (consumer key/secret para OAuth2, certificado A1 para mTLS) no portal Accurit.

## Pré-requisitos

1. Credenciais fornecidas pela SERPRO (consumer key, consumer secret).
2. Certificado A1 já cadastrado (consulte `scripts/cadastro-certificado-a1.md`).
3. Acesso ao portal como usuário com papel `GESTOR` do escritório.

## Passo a passo

### 1. Inserir credenciais de produção

No portal, navegue para:
```
Administração > SERPRO > Credenciais
```

Informe:
- **Consumer Key**: fornecida pela SERPRO para o ambiente de produção.
- **Consumer Secret**: fornecido pela SERPRO para o ambiente de produção.
- **Ambiente**: `PRODUCAO` (ou `TRIAL` para ambiente de homologação).

As credenciais são armazenadas **cifradas** com a `CERTIFICATE_MASTER_KEY`.

### 2. Associar certificado A1 (para mTLS)

Se o escritório for usar produção do Integra Contador (P5), associe o
certificado A1 cadastrado anteriormente à credencial de produção. O certificado
é usado para estabelecer a conexão mTLS com o SERPRO.

### 3. Configurar ambiente do escritório

O escritório opera em modo `TRIAL` por padrão. Para mudar para `PRODUCAO`,
consulte `scripts/troca-trial-producao.md`.

### 4. Validar credenciais

O sistema faz uma chamada de validação contra o endpoint de homologação do
SERPRO para confirmar que as credenciais estão funcionais antes de salvar.

---

**Nota de segurança:** Credenciais de produção nunca devem ser compartilhadas
entre escritórios. Cada escritório tem suas próprias credenciais isoladas por
`escritorio_id`.
