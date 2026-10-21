#!/bin/sh
set -eu

: "${DATABASE_URL:?DATABASE_URL is required}"
: "${ADMIN_DATABASE_URL:?ADMIN_DATABASE_URL is required}"

TEMP_DB="accurit_schema_tmp"
SCHEMA_FILE="src/infrastructure/persistence/schema.rs"

cleanup() {
    psql "$ADMIN_DATABASE_URL" -v ON_ERROR_STOP=1 \
        -c "DROP DATABASE IF EXISTS \"$TEMP_DB\" WITH (FORCE);" >/dev/null 2>&1 || true
}
trap cleanup EXIT

cleanup
psql "$ADMIN_DATABASE_URL" -v ON_ERROR_STOP=1 \
    -c "CREATE DATABASE \"$TEMP_DB\";" >/dev/null

diesel migration run --database-url "$DATABASE_URL" >/dev/null
diesel print-schema --database-url "$DATABASE_URL" > "$SCHEMA_FILE"
rustfmt --edition 2024 "$SCHEMA_FILE"

printf 'schema.rs gerado em %s\n' "$SCHEMA_FILE"
