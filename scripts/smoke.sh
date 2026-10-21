#!/bin/sh
set -eu

RAIZ=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
PORTA=80

if [ -f "$RAIZ/.env" ]; then
    valor=$(grep -E '^HTTP_PORT=' "$RAIZ/.env" | head -n 1 | cut -d= -f2- | tr -d '\r')
    if [ -n "$valor" ]; then
        PORTA=$valor
    fi
fi

BASE_URL="http://localhost:$PORTA"
CORPO=$(mktemp)
CABECALHOS=$(mktemp)
trap 'rm -f "$CORPO" "$CABECALHOS"' EXIT

falhas=0
codigo=""
ultima_descricao=""

pedido() {
    caminho=$1
    shift
    codigo=$(curl -sS --max-time 15 -o "$CORPO" -D "$CABECALHOS" -w '%{http_code}' "$@" "$BASE_URL$caminho" 2>/dev/null || true)
    if [ -z "$codigo" ]; then
        codigo="000"
    fi
}

verificar_status() {
    ultima_descricao=$1
    if [ "$codigo" = "$2" ]; then
        printf 'OK    %s\n' "$ultima_descricao"
    else
        printf 'FALHA %s (esperado %s, obtido %s)\n' "$ultima_descricao" "$2" "$codigo"
        falhas=$((falhas + 1))
    fi
}

verificar_corpo() {
    if grep -Eq "$1" "$CORPO"; then
        printf 'OK    %s\n' "$2"
    else
        printf 'FALHA %s (corpo inesperado)\n' "$2"
        falhas=$((falhas + 1))
    fi
}

verificar_cabecalho() {
    if grep -Eiq "$1" "$CABECALHOS"; then
        printf 'OK    %s\n' "$2"
    else
        printf 'FALHA %s (cabecalho ausente)\n' "$2"
        falhas=$((falhas + 1))
    fi
}

printf 'aguardando a stack em %s...\n' "$BASE_URL"
pronto=0
tentativa=1
while [ "$tentativa" -le 60 ]; do
    codigo=$(curl -sS --max-time 3 -o /dev/null -w '%{http_code}' "$BASE_URL/api/v1/health" 2>/dev/null || true)
    if [ "$codigo" = "200" ]; then
        pronto=1
        break
    fi
    sleep 2
    tentativa=$((tentativa + 1))
done

if [ "$pronto" -ne 1 ]; then
    printf 'FALHA a stack nao ficou saudavel em 120s\n' >&2
    exit 1
fi

pedido "/healthz"
verificar_status "/healthz responde 200" "200"
verificar_corpo '^ok' "healthz retorna ok"

pedido "/"
verificar_status "/ responde 200" "200"

pedido "/admin/"
verificar_status "/admin/ responde 200" "200"

pedido "/admin/qualquer-rota-spa"
verificar_status "rota SPA do admin responde 200" "200"

pedido "/app/"
verificar_status "/app/ responde 200" "200"

pedido "/login"
verificar_status "/login responde 301" "301"
verificar_cabecalho '^location: */app/login' "/login redireciona para /app/login"

pedido "/api/v1/health"
verificar_status "API health responde 200" "200"
verificar_corpo '"db"[[:space:]]*:[[:space:]]*"ok"' "API health reporta db ok"
verificar_cabecalho '^x-request-id:' "API health devolve X-Request-Id"

pedido "/api/v1/nao-existe"
verificar_status "rota de API inexistente responde 404" "404"
verificar_cabecalho '^content-type:.*application/json' "404 de API e JSON"
verificar_corpo '"code"[[:space:]]*:[[:space:]]*"ROTA_NAO_ENCONTRADA"' "404 de API usa envelope padrao"

pedido "/api/v1/health" -H "X-Request-Id: abc"
verificar_status "health aceita requisicao com request-id invalido" "200"
if grep -Eiq '^x-request-id: *abc *\r?$' "$CABECALHOS"; then
    printf 'FALHA request-id invalido foi ecoado\n'
    falhas=$((falhas + 1))
else
    printf 'OK    request-id invalido nao e ecoado\n'
fi

if [ "$falhas" -ne 0 ]; then
    printf '\n%d verificacao(oes) falharam.\n' "$falhas" >&2
    exit 1
fi

printf '\nSmoke OK.\n'
