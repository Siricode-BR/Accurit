$ErrorActionPreference = "Stop"

$raiz = Split-Path -Parent $PSScriptRoot
$exemplo = Join-Path $raiz ".env.example"
$destino = Join-Path $raiz ".env"

if (Test-Path -LiteralPath $destino) {
    Write-Error "erro: $destino já existe; remova-o ou renomeie-o antes de continuar."
    exit 1
}

if (-not (Test-Path -LiteralPath $exemplo)) {
    Write-Error "erro: $exemplo não encontrado."
    exit 1
}

function New-Hex([int]$quantidadeBytes) {
    $bytes = New-Object byte[] $quantidadeBytes
    $rng.GetBytes($bytes)
    return -join ($bytes | ForEach-Object { $_.ToString("x2") })
}

function New-Base64([int]$quantidadeBytes) {
    $bytes = New-Object byte[] $quantidadeBytes
    $rng.GetBytes($bytes)
    return [Convert]::ToBase64String($bytes)
}

$rng = [System.Security.Cryptography.RandomNumberGenerator]::Create()
try {
    $senhaAdmin = New-Hex 24
    $senhaApp = New-Hex 24
    $jwtSecret = New-Base64 48
    $chaveMestre = New-Base64 32
}
finally {
    $rng.Dispose()
}

$linhas = Get-Content -LiteralPath $exemplo -Encoding UTF8 | ForEach-Object {
    if ($_ -match '^POSTGRES_PASSWORD=') {
        "POSTGRES_PASSWORD=$senhaAdmin"
    }
    elseif ($_ -match '^POSTGRES_APP_PASSWORD=') {
        "POSTGRES_APP_PASSWORD=$senhaApp"
    }
    elseif ($_ -match '^DATABASE_URL=') {
        "DATABASE_URL=postgres://accurit_app:$senhaApp@postgres:5432/accurit"
    }
    elseif ($_ -match '^TEST_DATABASE_ADMIN_URL=') {
        "TEST_DATABASE_ADMIN_URL=postgres://accurit:$senhaAdmin@postgres:5432/accurit"
    }
    elseif ($_ -match '^JWT_SECRET=') {
        "JWT_SECRET=$jwtSecret"
    }
    elseif ($_ -match '^CERTIFICATE_MASTER_KEY=') {
        "CERTIFICATE_MASTER_KEY=$chaveMestre"
    }
    else {
        $_
    }
}

$utf8SemBom = New-Object System.Text.UTF8Encoding($false)
[System.IO.File]::WriteAllLines($destino, $linhas, $utf8SemBom)

if (Select-String -LiteralPath $destino -Pattern 'change-me' -Quiet) {
    Remove-Item -LiteralPath $destino -Force
    Write-Error "erro: ainda há placeholders no .env gerado; verifique .env.example."
    exit 1
}

Write-Output "arquivo .env criado a partir de .env.example (segredos não exibidos)."
