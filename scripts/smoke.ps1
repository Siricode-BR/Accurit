$ErrorActionPreference = "Stop"

Add-Type -AssemblyName System.Net.Http

$raiz = Split-Path -Parent $PSScriptRoot
$porta = 80
$arquivoEnv = Join-Path $raiz ".env"

if (Test-Path -LiteralPath $arquivoEnv) {
    $linhaPorta = Get-Content -LiteralPath $arquivoEnv |
        Where-Object { $_ -match '^HTTP_PORT=' } |
        Select-Object -First 1
    if ($linhaPorta) {
        $porta = [int](($linhaPorta -split '=', 2)[1].Trim())
    }
}

$baseUrl = "http://localhost:$porta"
$handler = New-Object System.Net.Http.HttpClientHandler
$handler.AllowAutoRedirect = $false
$client = New-Object System.Net.Http.HttpClient($handler)
$client.Timeout = [TimeSpan]::FromSeconds(15)

function Invoke-Pedido([string]$caminho, [hashtable]$cabecalhos = @{}) {
    $requisicao = New-Object System.Net.Http.HttpRequestMessage(
        [System.Net.Http.HttpMethod]::Get,
        "$baseUrl$caminho"
    )

    foreach ($chave in $cabecalhos.Keys) {
        [void]$requisicao.Headers.TryAddWithoutValidation($chave, $cabecalhos[$chave])
    }

    try {
        $resposta = $client.SendAsync($requisicao).GetAwaiter().GetResult()
        $corpo = $resposta.Content.ReadAsStringAsync().GetAwaiter().GetResult()
        $headers = @{}

        foreach ($header in $resposta.Headers) {
            $headers[$header.Key.ToLowerInvariant()] = ($header.Value -join ", ")
        }
        foreach ($header in $resposta.Content.Headers) {
            $headers[$header.Key.ToLowerInvariant()] = ($header.Value -join ", ")
        }

        return [pscustomobject]@{
            Status  = [int]$resposta.StatusCode
            Headers = $headers
            Body    = $corpo
        }
    }
    catch {
        return [pscustomobject]@{
            Status  = 0
            Headers = @{}
            Body    = ""
        }
    }
    finally {
        $requisicao.Dispose()
    }
}

$falhas = 0

function Verificar([string]$descricao, [scriptblock]$teste) {
    try {
        & $teste
        Write-Output "OK    $descricao"
    }
    catch {
        Write-Output "FALHA $descricao ($($_.Exception.Message))"
        $script:falhas++
    }
}

Write-Output "aguardando a stack em $baseUrl..."
$pronto = $false
$limite = (Get-Date).AddSeconds(120)

do {
    $resposta = Invoke-Pedido "/api/v1/health"
    if ($resposta.Status -eq 200) {
        $pronto = $true
        break
    }
    Start-Sleep -Seconds 2
} while ((Get-Date) -lt $limite)

if (-not $pronto) {
    Write-Error "a stack nao ficou saudavel em 120s"
    exit 1
}

$healthz = Invoke-Pedido "/healthz"
Verificar "/healthz responde 200" {
    if ($healthz.Status -ne 200) { throw "obtido $($healthz.Status)" }
}
Verificar "healthz retorna ok" {
    if ($healthz.Body.Trim() -ne "ok") { throw "corpo inesperado" }
}

$raizWeb = Invoke-Pedido "/"
Verificar "/ responde 200" {
    if ($raizWeb.Status -ne 200) { throw "obtido $($raizWeb.Status)" }
}

$admin = Invoke-Pedido "/admin/"
Verificar "/admin/ responde 200" {
    if ($admin.Status -ne 200) { throw "obtido $($admin.Status)" }
}

$adminSpa = Invoke-Pedido "/admin/qualquer-rota-spa"
Verificar "rota SPA do admin responde 200" {
    if ($adminSpa.Status -ne 200) { throw "obtido $($adminSpa.Status)" }
}

$app = Invoke-Pedido "/app/"
Verificar "/app/ responde 200" {
    if ($app.Status -ne 200) { throw "obtido $($app.Status)" }
}

$login = Invoke-Pedido "/login"
Verificar "/login responde 301" {
    if ($login.Status -ne 301) { throw "obtido $($login.Status)" }
}
Verificar "/login redireciona para /app/login" {
    $location = $login.Headers["location"]
    if (-not $location -or -not $location.EndsWith("/app/login")) {
        throw "location inesperado"
    }
}

$health = Invoke-Pedido "/api/v1/health"
Verificar "API health responde 200" {
    if ($health.Status -ne 200) { throw "obtido $($health.Status)" }
}
Verificar "API health reporta db ok" {
    $json = $health.Body | ConvertFrom-Json
    if ($json.db -ne "ok") { throw "db != ok" }
}
Verificar "API health devolve X-Request-Id" {
    if (-not $health.Headers["x-request-id"]) { throw "cabecalho ausente" }
}

$inexistente = Invoke-Pedido "/api/v1/nao-existe"
Verificar "rota de API inexistente responde 404" {
    if ($inexistente.Status -ne 404) { throw "obtido $($inexistente.Status)" }
}
Verificar "404 de API e JSON" {
    if ($inexistente.Headers["content-type"] -notmatch "application/json") {
        throw "content-type inesperado"
    }
}
Verificar "404 de API usa envelope padrao" {
    $json = $inexistente.Body | ConvertFrom-Json
    if ($json.error.code -ne "ROTA_NAO_ENCONTRADA") { throw "code inesperado" }
}

$requestIdInvalido = Invoke-Pedido "/api/v1/health" @{ "X-Request-Id" = "abc" }
Verificar "health aceita requisicao com request-id invalido" {
    if ($requestIdInvalido.Status -ne 200) { throw "obtido $($requestIdInvalido.Status)" }
}
Verificar "request-id invalido nao e ecoado" {
    if ($requestIdInvalido.Headers["x-request-id"] -eq "abc") {
        throw "request-id invalido ecoado"
    }
}

$client.Dispose()
$handler.Dispose()

if ($falhas -ne 0) {
    Write-Error "$falhas verificacao(oes) falharam."
    exit 1
}

Write-Output ""
Write-Output "Smoke OK."
