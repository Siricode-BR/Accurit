# Backup e restauração

Este runbook cobre o backup lógico e a restauração do PostgreSQL do Accurit. Ele foi
exercitado localmente contra o schema completo (40 tabelas e 32 migrations) e as
contagens das tabelas principais ficaram idênticas após o restore.

## Objetivos

| Item | Alvo operacional |
| --- | --- |
| RPO (perda máxima aceitável) | até 24 h com o dump diário; próximo de zero somente com arquivamento de WAL/PITR do provedor |
| RTO (tempo de retorno) | até 2 h para restaurar o dump e validar a aplicação |
| Retenção local | 7 dias por padrão (`KEEP_DAYS`), ajustável |
| Retenção offsite | definir por contrato/escritório; mínimo sugerido de 30 dias criptografados |
| Criptografia | obrigatória em repouso e em trânsito; nunca versionar dumps |

## O que os scripts fazem

- `scripts/backup.ps1` e `scripts/backup.sh`
  - `pg_dump --format=custom --compress=9 --no-owner --no-privileges`;
  - valida o arquivo com `pg_restore --list`;
  - gera sidecar SHA-256;
  - remove dumps locais mais antigos que a retenção.
- `scripts/restore.ps1` e `scripts/restore.sh`
  - exigem confirmação explícita (`-Force`/`--force`);
  - conferem o SHA-256 quando o sidecar existe;
  - recusam destino com tabelas, salvo `-Limpar`/`--limpar`;
  - restauram com `--exit-on-error --single-transaction --no-owner --no-privileges`;
  - conferem a quantidade de tabelas e a migration mais recente.

Os scripts **não** configuram WAL/PITR, replicação, criptografia do bucket ou rotação
de credenciais. Esses itens pertencem ao provedor de banco/infraestrutura.

## Backup

PowerShell:

```powershell
$env:DATABASE_URL='postgres://accurit:senha@127.0.0.1:5432/accurit'
./scripts/backup.ps1 -OutDir D:\backups\accurit -Label diario -KeepDays 7
```

Bash:

```bash
DATABASE_URL='postgres://accurit:senha@127.0.0.1:5432/accurit' \
BACKUP_DIR=/var/backups/accurit KEEP_DAYS=7 \
./scripts/backup.sh diario
```

Requisitos:

- `pg_dump`/`pg_restore` da mesma versão ou mais novos que o servidor;
- espaço livre compatível com o tamanho comprimido do banco;
- acesso de leitura ao banco e permissão de escrita no diretório de destino.

Agendamento sugerido (exemplo em cron, fora do container):

```cron
15 2 * * * cd /opt/accurit && DATABASE_URL="$(cat /run/secrets/database_url)" BACKUP_DIR=/var/backups/accurit ./scripts/backup.sh diario >> /var/log/accurit-backup.log 2>&1
```

Depois do backup, copie o `.dump` e o `.sha256` para o armazenamento offsite
criptografado e registre data, tamanho, hash e responsável. O arquivo local não deve
ser a única cópia.

## Restauração

### Drill ou homologação

```powershell
# banco de destino novo e vazio
$env:DATABASE_URL='postgres://accurit:senha@127.0.0.1:5432/accurit_drill'
./scripts/restore.ps1 `
  -Backup D:\backups\accurit\accurit-20261110T020000Z-diario.dump `
  -Force -Limpar
```

```bash
DATABASE_URL='postgres://accurit:senha@127.0.0.1:5432/accurit_drill' \
./scripts/restore.sh --backup /var/backups/accurit/accurit-20261110T020000Z-diario.dump --force --limpar
```

### Produção

1. Abra janela de manutenção e coloque a aplicação em modo somente leitura ou pare as
   réplicas do backend.
2. Crie um banco **novo** (ex.: `accurit_restore_20261110`) e restaure nele; não
   sobrescreva o banco atual durante a validação.
3. Rode as verificações abaixo.
4. Aponte `DATABASE_URL` para o banco restaurado, suba uma réplica e valide login,
   clientes, consumo, relatórios e LGPD.
5. Só então promova as demais réplicas e remova o banco antigo após o período de
   segurança definido.

O `pg_restore --clean` do script é adequado ao banco de destino do drill. Em produção,
prefira um banco vazio e a troca de conexão para permitir rollback imediato.

## Verificações pós-restore

```sql
SELECT count(*) AS migrations FROM __diesel_schema_migrations;
SELECT max(version) AS ultima_migration FROM __diesel_schema_migrations;

SELECT count(*) FROM escritorios;
SELECT count(*) FROM usuarios;
SELECT count(*) FROM clientes;
SELECT count(*) FROM relatorios;
SELECT count(*) FROM chamadas_serpro;
```

Checklist:

- [ ] `GET /api/v1/health` responde `200` e `db: ok`;
- [ ] login de admin e de um usuário do escritório funciona;
- [ ] listagem de clientes e detalhe de um cliente retornam dados coerentes;
- [ ] painel de consumo mostra o mês corrente e o limite do plano;
- [ ] geração/download de relatório funciona;
- [ ] exportação LGPD de um cliente de teste funciona;
- [ ] contagens das tabelas principais conferem com a origem;
- [ ] nenhuma credencial, certificado ou token aparece nos logs.

## Segurança e LGPD

- Dumps contêm dados pessoais e fiscais: trate-os como dados sensíveis de nível
  máximo, com acesso nominal, criptografia e trilha de auditoria.
- Restrinja a cópia offsite ao mínimo de pessoas e registre cada download.
- Destrua mídias/dumps expirados de forma segura.
- Nunca coloque `DATABASE_URL`, senhas ou dumps no Git; `.env*` e `backups/` são
  ignorados.
- A retenção do backup deve respeitar a política de retenção e eliminação do
  escritório contratante.

## Limitações conhecidas

- Backup lógico diário não garante RPO próximo de zero; para isso, habilite PITR/WAL
  no PostgreSQL gerenciado ou use `pg_basebackup` + arquivamento contínuo.
- O drill local não substitui um ensaio em ambiente real com as credenciais,
  volumes e tempos de transferência de produção.
- O restore em banco com extensões/roles específicas pode exigir ajuste de owner;
  os scripts usam `--no-owner --no-privileges` para facilitar a promoção entre
  ambientes.
