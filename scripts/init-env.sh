#!/bin/sh
set -eu

umask 077

RAIZ=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
EXEMPLO="$RAIZ/.env.example"
DESTINO="$RAIZ/.env"

if [ -e "$DESTINO" ]; then
    printf 'erro: %s já existe; remova-o ou renomeie-o antes de continuar.\n' "$DESTINO" >&2
    exit 1
fi

if [ ! -f "$EXEMPLO" ]; then
    printf 'erro: %s não encontrado.\n' "$EXEMPLO" >&2
    exit 1
fi

if ! command -v openssl >/dev/null 2>&1; then
    printf 'erro: openssl não encontrado no PATH.\n' >&2
    exit 1
fi

POSTGRES_PASSWORD=$(openssl rand -hex 24)
POSTGRES_APP_PASSWORD=$(openssl rand -hex 24)
JWT_SECRET=$(openssl rand -base64 48 | tr -d '\n')
CERTIFICATE_MASTER_KEY=$(openssl rand -base64 32 | tr -d '\n')

sed \
    -e "s|^POSTGRES_PASSWORD=.*|POSTGRES_PASSWORD=$POSTGRES_PASSWORD|" \
    -e "s|^POSTGRES_APP_PASSWORD=.*|POSTGRES_APP_PASSWORD=$POSTGRES_APP_PASSWORD|" \
    -e "s|^DATABASE_URL=.*|DATABASE_URL=postgres://accurit_app:$POSTGRES_APP_PASSWORD@postgres:5432/accurit|" \
    -e "s|^TEST_DATABASE_ADMIN_URL=.*|TEST_DATABASE_ADMIN_URL=postgres://accurit:$POSTGRES_PASSWORD@postgres:5432/accurit|" \
    -e "s|^JWT_SECRET=.*|JWT_SECRET=$JWT_SECRET|" \
    -e "s|^CERTIFICATE_MASTER_KEY=.*|CERTIFICATE_MASTER_KEY=$CERTIFICATE_MASTER_KEY|" \
    "$EXEMPLO" > "$DESTINO"

if grep -q 'change-me' "$DESTINO"; then
    rm -f "$DESTINO"
    printf 'erro: ainda há placeholders no .env gerado; verifique .env.example.\n' >&2
    exit 1
fi

printf 'arquivo .env criado a partir de .env.example (segredos não exibidos).\n'
