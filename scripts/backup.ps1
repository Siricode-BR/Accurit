<#
.SYNOPSIS
  Gera backup lógico do PostgreSQL do Accurit em formato custom e valida o arquivo.

.DESCRIPTION
  Usa pg_dump no formato custom (restaurável com pg_restore), sem owner/ACL para
  facilitar a restauração em outro ambiente. Gera sidecar SHA-256, confere o
  índice do dump com pg_restore --list e remove backups mais antigos que
  -KeepDays.

  O pg_dump precisa ser da mesma versão ou mais novo que o servidor PostgreSQL.
  Este script NÃO substitui PITR: para RPO próximo de zero é necessário arquivar
  WAL no provedor/gerenciador do banco.

.PARAMETER DatabaseUrl
  URL do banco de origem. Padrão: variável de ambiente DATABASE_URL.

.PARAMETER OutDir
  Diretório de saída. Padrão: <raiz>/backups.

.PARAMETER Label
  Rótulo opcional no nome do arquivo (ex.: pre-deploy).

.PARAMETER KeepDays
  Retenção local em dias. Use 0 para não remover nada. Padrão: 7.

.PARAMETER SkipVerify
  Não executa pg_restore --list (não recomendado).

.EXAMPLE
  $env:DATABASE_URL='postgres://accurit:senha@127.0.0.1:5432/accurit'
  ./scripts/backup.ps1 -Label pre-deploy
#>
[CmdletBinding()]
param(
    [string]$DatabaseUrl = $env:DATABASE_URL,
    [string]$OutDir = (Join-Path (Split-Path -Parent $PSScriptRoot) 'backups'),
    [string]$Label = '',
    [int]$KeepDays = 7,
    [switch]$SkipVerify
)

$ErrorActionPreference = 'Stop'

if ([string]::IsNullOrWhiteSpace($DatabaseUrl)) {
    throw 'Informe -DatabaseUrl ou defina DATABASE_URL.'
}

foreach ($ferramenta in @('pg_dump', 'pg_restore')) {
    if (-not (Get-Command $ferramenta -ErrorAction SilentlyContinue)) {
        throw "$ferramenta não encontrado no PATH. Instale o cliente PostgreSQL correspondente."
    }
}

New-Item -ItemType Directory -Force -Path $OutDir | Out-Null

$timestamp = (Get-Date).ToUniversalTime().ToString('yyyyMMddTHHmmssZ')
$sufixo = if ([string]::IsNullOrWhiteSpace($Label)) { '' } else { '-' + ($Label -replace '[^A-Za-z0-9._-]', '-') }
$arquivo = Join-Path $OutDir "accurit-$timestamp$sufixo.dump"
$hashFile = "$arquivo.sha256"

Write-Host "Gerando backup em $arquivo ..."
& pg_dump `
    --dbname=$DatabaseUrl `
    --format=custom `
    --compress=9 `
    --no-owner `
    --no-privileges `
    --file=$arquivo

if ($LASTEXITCODE -ne 0) {
    throw "pg_dump falhou com código $LASTEXITCODE."
}

if (-not (Test-Path -LiteralPath $arquivo) -or (Get-Item -LiteralPath $arquivo).Length -eq 0) {
    throw 'pg_dump não produziu um arquivo válido.'
}

if (-not $SkipVerify) {
    Write-Host 'Validando estrutura do dump ...'
    $listagem = & pg_restore --list $arquivo
    if ($LASTEXITCODE -ne 0 -or ($listagem -join "`n") -notmatch 'Table data|TABLE DATA') {
        throw 'O dump não pôde ser lido por pg_restore --list ou não contém dados de tabelas.'
    }
}

$hash = (Get-FileHash -LiteralPath $arquivo -Algorithm SHA256).Hash.ToLowerInvariant()
"$hash  $(Split-Path -Leaf $arquivo)" | Set-Content -LiteralPath $hashFile -Encoding ascii

if ($KeepDays -gt 0) {
    $limite = (Get-Date).AddDays(-$KeepDays)
    Get-ChildItem -LiteralPath $OutDir -Filter 'accurit-*.dump' |
        Where-Object { $_.LastWriteTime -lt $limite } |
        ForEach-Object {
            Write-Host "Removendo backup antigo: $($_.Name)"
            Remove-Item -LiteralPath $_.FullName -Force
            $sidecar = "$($_.FullName).sha256"
            if (Test-Path -LiteralPath $sidecar) {
                Remove-Item -LiteralPath $sidecar -Force
            }
        }
}

$tamanhoMb = [math]::Round((Get-Item -LiteralPath $arquivo).Length / 1MB, 2)
Write-Host "Backup concluído: $arquivo ($tamanhoMb MB)"
Write-Host "SHA-256: $hash"
