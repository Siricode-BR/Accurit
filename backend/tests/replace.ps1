$content = Get-Content http_usuarios.rs -Raw
$content = $content -replace 'semear_usuario\(&a\.banco, "alfa",', 'semear_usuario(&a.banco, CNPJ_PADRAO,'
$content = $content -replace '"alfa",', '"52998224725896",'
$content = $content -replace '\"alfa\"', '"52998224725896"'
Set-Content http_usuarios.rs -Value $content -Encoding utf8