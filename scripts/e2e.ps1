<#
.SYNOPSIS
  Sobe um stack local mÃ­nimo e executa a suÃ­te E2E do Playwright.

.DESCRIPTION
  Orquestra o que os testes E2E precisam: banco PostgreSQL, backend com mocks
  Trial, seed de admin/tenant/plano, builds estÃ¡ticos dos dois frontends e os
  servidores estÃ¡ticos que fazem proxy de /api para o backend.

  Requisitos: PostgreSQL 16+ acessÃ­vel, Rust, Node 24 e pnpm. O navegador do
  Playwright Ã© instalado com: pnpm --filter frontend? (na pasta frontend)
  pnpm exec playwright install chromium

  O script NÃƒO usa SERPRO real: SERPRO_TRIAL_MOCK=true e as credenciais Trial
  podem ficar vazias. Use -Reset para recriar o banco de E2E do zero.

.PARAMETER DatabaseUrl
  Banco exclusivo de E2E. PadrÃ£o: postgres://accurit:accurit@127.0.0.1:5432/accurit_e2e.

.PARAMETER AdminUrl
  ConexÃ£o administrativa usada para criar/recriar o banco. PadrÃ£o:
  postgres://postgres:postgres@127.0.0.1:5432/postgres.

.PARAMETER Reset
  DROP/CREATE do banco de E2E antes de subir o stack.

.PARAMETER SkipBuild
  Reaproveita builds existentes de backend e frontends.

.EXAMPLE
  ./scripts/e2e.ps1 -Reset
#>
[CmdletBinding()]
param(
    [string]$DatabaseUrl = 'postgres://accurit:accurit@127.0.0.1:5432/accurit_e2e',
    [string]$AdminUrl = 'postgres://postgres:postgres@127.0.0.1:5432/postgres',
    [switch]$Reset,
    [switch]$SkipBuild
)

$ErrorActionPreference = 'Stop'
$raiz = Split-Path -Parent $PSScriptRoot
$frontend = Join-Path $raiz 'frontend'
$backendDir = Join-Path $raiz 'backend'
$node = (Get-Command node -ErrorAction Stop).Source
$processos = New-Object System.Collections.Generic.List[System.Diagnostics.Process]

function Escrever([string]$mensagem) { Write-Host "==> $mensagem" }

function Url-Banco([string]$url) {
    $uri = [uri]$url
    $porta = 5432
    if ($uri.Port -gt 0) { $porta = $uri.Port }
    $usuario = ''
    $senha = ''
    if (-not [string]::IsNullOrWhiteSpace($uri.UserInfo)) {
        $partes = $uri.UserInfo.Split(':')
        $usuario = $partes[0]
        if ($partes.Length -gt 1) { $senha = $partes[1] }
    }
    return [pscustomobject]@{
        Host = $uri.Host
        Port = $porta
        User = $usuario
        Senha = $senha
        Nome = $uri.AbsolutePath.Trim('/')
    }
}

function Executar-Psql([string]$url, [string[]]$argumentos) {
    $info = Url-Banco $url
    $anterior = $env:PGPASSWORD
    $env:PGPASSWORD = $info.Senha
    try {
        & psql -h $info.Host -p $info.Port -U $info.User -d $info.Nome @argumentos
        if ($LASTEXITCODE -ne 0) { throw "psql falhou (cÃ³digo $LASTEXITCODE)." }
    }
    finally {
        $env:PGPASSWORD = $anterior
    }
}

function Esperar-Http([string]$url, [int]$segundos = 120) {
    $limite = (Get-Date).AddSeconds($segundos)
    while ((Get-Date) -lt $limite) {
        try {
            $resposta = Invoke-WebRequest -Uri $url -UseBasicParsing -TimeoutSec 3
            if ($resposta.StatusCode -ge 200 -and $resposta.StatusCode -lt 500) { return }
        }
        catch {
            Start-Sleep -Milliseconds 750
        }
    }
    throw "timeout aguardando $url"
}

function Parar-Processos {
    foreach ($processo in $processos) {
        if ($processo -and -not $processo.HasExited) {
            Stop-Process -Id $processo.Id -Force -ErrorAction SilentlyContinue
        }
    }
}

try {
    foreach ($porta in @(8080, 4201, 4202)) {
        $emUso = Get-NetTCPConnection -State Listen -LocalPort $porta -ErrorAction SilentlyContinue
        if ($emUso) {
            throw "porta $porta já está em uso (PID $($emUso[0].OwningProcess)). Encerre o processo antes de rodar o E2E."
        }
    }

    Escrever 'Preparando banco de E2E'
    $info = Url-Banco $DatabaseUrl
    $infoAdmin = Url-Banco $AdminUrl

    if ($Reset) {
        Executar-Psql $AdminUrl @('-c', "DROP DATABASE IF EXISTS `"$($info.Nome)`" WITH (FORCE);")
    }

    $existe = (Executar-Psql $AdminUrl @(
        '-tAc',
        "SELECT 1 FROM pg_database WHERE datname = '$($info.Nome)';"
    ) | Out-String).Trim()

    if ($existe -ne '1') {
        Executar-Psql $AdminUrl @(
            '-c',
            "CREATE DATABASE `"$($info.Nome)`" OWNER `"$($info.User)`";"
        )
    }

    if (-not $SkipBuild) {
        Escrever 'Compilando backend'
        Push-Location $backendDir
        try {
            cargo build
            if ($LASTEXITCODE -ne 0) { throw 'cargo build falhou.' }
        }
        finally { Pop-Location }

        Escrever 'Buildando frontends admin e client'
        Push-Location $frontend
        try {
            pnpm --filter admin build
            if ($LASTEXITCODE -ne 0) { throw 'build do admin falhou.' }
            pnpm --filter client build
            if ($LASTEXITCODE -ne 0) { throw 'build do client falhou.' }
        }
        finally { Pop-Location }
    }

    Escrever 'Configurando ambiente do backend'
    $env:APP_ENV = 'development'
    $env:DATABASE_URL = $DatabaseUrl
    $env:JWT_SECRET = 'e2e-secret-with-at-least-32-bytes-long'
    $env:CERTIFICATE_MASTER_KEY = 'BwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwc='
    $env:SERPRO_TOKEN_URL = 'https://gateway.apiserpro.serpro.gov.br/token'
    $env:SERPRO_SAPI_AUTH_URL = 'https://autenticacao.sapi.serpro.gov.br/authenticate'
    $env:SERPRO_INTEGRA_TRIAL_URL = 'https://gateway.apiserpro.serpro.gov.br/integra-contador-trial/v1'
    $env:SERPRO_DIVIDA_ATIVA_TRIAL_URL = 'https://gateway.apiserpro.serpro.gov.br/consulta-divida-ativa-trial/api/v1'
    $env:SERPRO_INTEGRA_PROD_URL = 'https://gateway.apiserpro.serpro.gov.br/integra-contador/v1'
    $env:SERPRO_TRIAL_MOCK = 'true'
    $env:AUTH_COOKIE_SECURE = 'false'
    $env:CORS_ALLOWED_ORIGINS = 'http://127.0.0.1:4201,http://127.0.0.1:4202'
    $env:METRICS_TOKEN = 'e2e-metrics'
    $env:RUST_LOG = 'warn'
    $env:BOOTSTRAP_ADMIN_PASSWORD = 'e2e-admin-123'
    $env:BOOTSTRAP_USER_PASSWORD = 'e2e-user-123'

    $logDir = Join-Path $env:TEMP 'accurit-e2e'
    New-Item -ItemType Directory -Force -Path $logDir | Out-Null

    Escrever 'Iniciando backend (migrations automÃ¡ticas)'
    $backendExe = Join-Path $backendDir 'target\debug\backend.exe'
    $backend = Start-Process -FilePath $backendExe -WorkingDirectory $backendDir `
        -RedirectStandardOutput (Join-Path $logDir 'backend.out.log') `
        -RedirectStandardError (Join-Path $logDir 'backend.err.log') `
        -PassThru
    $processos.Add($backend)
    Esperar-Http 'http://127.0.0.1:8080/api/v1/health'

    Escrever 'Semeando admin, tenant e plano'
    $adminExiste = (Executar-Psql $DatabaseUrl @('-tAc', "SELECT count(*) FROM administradores WHERE email = 'admin@e2e.test';") | Out-String).Trim()
    if ([int]$adminExiste -eq 0) {
        & $backendExe bootstrap-admin --nome 'Gestor E2E' --email 'admin@e2e.test' --cpf '11144477735'
        if ($LASTEXITCODE -ne 0) { throw 'bootstrap-admin falhou.' }
    }

    $tenantExiste = (Executar-Psql $DatabaseUrl @('-tAc', "SELECT count(*) FROM usuarios WHERE email = 'titular@e2e.test';") | Out-String).Trim()
    if ([int]$tenantExiste -eq 0) {
        & $backendExe bootstrap-tenant --escritorio-cnpj '11222333000181' --escritorio-nome 'Escritorio E2E' `
            --email 'titular@e2e.test' --cpf '52998224725' --papel 'CLIENTE'
        if ($LASTEXITCODE -ne 0) { throw 'bootstrap-tenant falhou.' }
    }

    $sqlPlano = @'
INSERT INTO planos_escritorio (id, nome, descricao, chamadas_mes, clientes_limite, usuarios_limite, preco_mensal_cents)
VALUES ('11111111-1111-4111-8111-111111111111', 'E2E', 'Plano de teste E2E', 1000, 50, 10, 0)
ON CONFLICT (id) DO NOTHING;
INSERT INTO assinaturas_escritorio (id, escritorio_id, plano_id, chamadas_mes_limite, clientes_limite, usuarios_limite, bloqueio_ao_exceder)
SELECT '22222222-2222-4222-8222-222222222222', id, '11111111-1111-4111-8111-111111111111', 1000, 50, 10, TRUE
FROM escritorios WHERE cnpj = '11222333000181'
ON CONFLICT (escritorio_id) DO NOTHING;
'@
    Executar-Psql $DatabaseUrl @('-v', 'ON_ERROR_STOP=1', '-c', $sqlPlano)

    Escrever 'Servindo frontends estÃ¡ticos com proxy /api'
    $env:PORT = '4201'
    $env:DIST = 'apps/admin/dist/admin/browser'
    $env:BACKEND = 'http://127.0.0.1:8080'
    $adminServer = Start-Process -FilePath $node -ArgumentList 'e2e/static-server.mjs' -WorkingDirectory $frontend `
        -RedirectStandardOutput (Join-Path $logDir 'admin.out.log') `
        -RedirectStandardError (Join-Path $logDir 'admin.err.log') `
        -PassThru
    $processos.Add($adminServer)

    $env:PORT = '4202'
    $env:DIST = 'apps/client/dist/client/browser'
    $clientServer = Start-Process -FilePath $node -ArgumentList 'e2e/static-server.mjs' -WorkingDirectory $frontend `
        -RedirectStandardOutput (Join-Path $logDir 'client.out.log') `
        -RedirectStandardError (Join-Path $logDir 'client.err.log') `
        -PassThru
    $processos.Add($clientServer)

    Esperar-Http 'http://127.0.0.1:4201/'
    Esperar-Http 'http://127.0.0.1:4202/'

    Escrever 'Executando Playwright'
    Push-Location $frontend
    try {
        pnpm exec playwright test
        $codigo = $LASTEXITCODE
    }
    finally { Pop-Location }

    if ($codigo -ne 0) { throw "Playwright falhou (cÃ³digo $codigo). Logs em $logDir." }
    Escrever 'E2E concluÃ­do com sucesso.'
}
finally {
    Parar-Processos
}


