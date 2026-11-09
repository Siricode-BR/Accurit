# Cadastro de Certificado A1 (scripts/cadastro-certificado-a1.md)

> Procedimento para o escritório de contabilidade cadastrar o certificado A1
> (e-CNPJ) no portal Accurit, permitindo autenticação mTLS junto ao SERPRO.

## Pré-requisitos

1. Certificado A1 válido (arquivo `.pfx` ou `.p12` + senha).
2. Acesso ao portal Accurit como usuário com papel `GESTOR` do escritório.
3. Variável `CERTIFICATE_MASTER_KEY` configurada no backend (chave mestra de 32 bytes em base64).

## Passo a passo

### 1. Acessar a tela de certificados

No portal do escritório, navegue para:
```
Administração > Certificados A1 > Cadastrar certificado
```

Ou diretamente pela rota `GET /api/v1/certificados` (listar) e `POST /api/v1/certificados` (criar).

### 2. Enviar o arquivo do certificado

- Selecione o arquivo `.pfx` ou `.p12` do certificado A1.
- Informe a senha do certificado.
- O sistema armazena o certificado **cifrado** com a `CERTIFICATE_MASTER_KEY`. A senha nunca é salva em texto claro.

### 3. Validação automática

O sistema valida:
- O certificado não está expirado (`valido_ate` > agora).
- O titular do certificado (CPF/CNPJ) confere com o CNPJ do escritório.
- O fingerprint SHA-256 do certificado é calculado e armazenado para auditoria.

### 4. Associar a credenciais SERPRO

Após cadastrar o certificado, o escritório deve associá-lo a uma credencial SERPRO
(consulte `scripts/credenciais-serpro.md`). O certificado é usado para:
- mTLS em chamadas de produção do Integra Contador (P5).
- Assinatura de XML no fluxo `AUTENTICAPROCURADOR`.

## Revogação

O gestor do escritório pode revogar um certificado a qualquer momento pela rota
`DELETE /api/v1/certificados/{id}`. Após a revogação, chamadas que dependeriam
do certificado falham com erro explícito (`CertificadoInvalido`).

---

**Nota:** O conteúdo decifrado do certificado (usado em mTLS) é obtido pela
infraestrutura na fase de integração SAPI. O banco armazena apenas metadados
(titular, fingerprint, validade) e a versão cifrada.
