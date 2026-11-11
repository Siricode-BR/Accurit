<#
.SYNOPSIS
  Restaura um backup lógico do Accurit em um banco PostgreSQL.

.DESCRIPTION
  Restauração destrutiva: por padrão exige -Force e recusa banco com tabelas.
  Use -Limpar para apagar o schema public antes de restaurar (ambiente de
  drill/homologação). O script valida o SHA-256 do sidecar quando existir,
  executa pg_restore em transação única e confere a versão de migration.

  NÃO aponte este script para produção sem janela de manutenção e aprovação.
  Para produção, prefira restaurar em um banco novo e promover por troca de
  conexão, mantendo o banco antigo intacto até a validação.

.PARAMETER Backup
  Caminho do arquivo .dump gerado por scripts/backup.ps1.

.PARAMETER DatabaseUrl
  URL do banco de destino. Padrão: variável de ambiente DATABASE_URL.

.PARAMETER Force
  Confirma a restauração destrutiva. Obrigatório.

.PARAMETER Limpar
  Executa DROP SCHEMA public CASCADE; CREATE SCHEMA public antes do restore.
  Sem este switch, o banco de destino precisa estar sem tabelas.

.PARAMETER PularVerificacaoHash
  Ignora a checagem do sidecar .sha256. Use apenas se o sidecar não existir.

.EXAMPLE
  ./scripts/restore.ps1 -Backup ./backups/accurit-20261110T120000Z.dump `
    -DatabaseUrl 'postgres://accurit:senha@127.0.0.1:5432/accurit_drill' `
    -Force -Limpar
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$Backup,
    [string]$DatabaseUrl = $env:DATABASE_URL,
    [switch]$Force,
    [switch]$Limpar,
    [switch]$PularVerificacaoHash
)

$ErrorActionPreference = 'Stop'

if (-not $Force) {
    throw 'Restauração destrutiva. Revise o destino e repita com -Force.'
}

if ([string]::IsNullOrWhiteSpace($DatabaseUrl)) {
    throw 'Informe -DatabaseUrl ou defina DATABASE_URL.'
}

foreach ($ferramenta in @('pg_restore', 'psql')) {
    if (-not (Get-Command $ferramenta -ErrorAction SilentlyContinue)) {
        throw "$ferramenta não encontrado no PATH."
    }
}

$Backup = (Resolve-Path -LiteralPath $Backup).Path
if (-not (Test-Path -LiteralPath $Backup -PathType Leaf)) {
    throw "Backup não encontrado: $Backup"
}

$hashFile = "$Backup.sha256"
if (-not $PularVerificacaoHash -and (Test-Path -LiteralPath $hashFile)) {
    $esperado = (Get-Content -LiteralPath $hashFile -TotalCount 1).Split(' ')[0].Trim().ToLowerInvariant()
    $obtido = (Get-FileHash -LiteralPath $Backup -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($esperado -ne $obtido) {
        throw "SHA-256 divergente. Esperado $esperado, obtido $obtido."
    }
    Write-Host 'SHA-256 do backup conferido.'
}
elseif (-not $PularVerificacaoHash) {
    Write-Warning "Sidecar $hashFile ausente; seguindo sem verificação de integridade."
}

$tabelas = (& psql $DatabaseUrl -v ON_ERROR_STOP=1 -tAc `
    "SELECT count(*) FROM information_schema.tables WHERE table_schema NOT IN ('pg_catalog','information_schema');").Trim()
if ($LASTEXITCODE -ne 0) {
    throw 'Não foi possível consultar o banco de destino.'
}

if ([int]$tabelas -gt 0) {
    if (-not $Limpar) {
        throw "O banco de destino já contém $tabelas tabela(s). Use -Limpar para um drill ou escolha um banco vazio."
    }
    Write-Host "Limpando schema public do destino ($tabelas tabelas) ..."
    & psql $DatabaseUrl -v ON_ERROR_STOP=1 -c 'DROP SCHEMA public CASCADE; CREATE SCHEMA public;'
    if ($LASTEXITCODE -ne 0) {
        throw 'Falha ao limpar o schema public.'
    }
}

Write-Host "Restaurando $Backup ..."
& pg_restore `
    --dbname=$DatabaseUrl `
    --clean `
    --if-exists `
    --no-owner `
    --no-privileges `
    --exit-on-error `
    --single-transaction `
    $Backup

if ($LASTEXITCODE -ne 0) {
    throw "pg_restore falhou com código $LASTEXITCODE."
}

$totalTabelas = (& psql $DatabaseUrl -tAc `
    "SELECT count(*) FROM information_schema.tables WHERE table_schema NOT IN ('pg_catalog','information_schema');").Trim()
$migration = (& psql $DatabaseUrl -tAc `
    "SELECT coalesce(max(version), 'sem migrations') FROM __diesel_schema_migrations;").Trim()

Write-Host "Restauração concluída: $totalTabelas tabela(s); migration mais recente: $migration"
Write-Host 'Valide a aplicação contra o banco restaurado antes de qualquer promoção.'
