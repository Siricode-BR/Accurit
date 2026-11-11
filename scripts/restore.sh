#!/usr/bin/env bash
# Restaura um backup lógico do Accurit em um banco PostgreSQL.
#
# Uso (destrutivo, exige --force):
#   DATABASE_URL=postgres://... scripts/restore.sh --backup caminho.dump --force [--limpar]
#
# Opções:
#   --backup ARQUIVO   dump gerado por scripts/backup.sh (obrigatório)
#   --force            confirma a restauração destrutiva
#   --limpar           DROP SCHEMA public CASCADE antes de restaurar
#   --sem-hash         não confere o sidecar .sha256
#
# Sem --limpar o banco de destino precisa estar sem tabelas. Para produção,
# restaure em um banco novo e promova por troca de conexão.
set -euo pipefail

: "${DATABASE_URL:?defina DATABASE_URL}"
backup=""
force=0
limpar=0
verificar_hash=1

while [[ $# -gt 0 ]]; do
  case "$1" in
    --backup) backup="${2:?--backup exige um caminho}"; shift 2 ;;
    --force) force=1; shift ;;
    --limpar) limpar=1; shift ;;
    --sem-hash) verificar_hash=0; shift ;;
    -h|--help) sed -n '1,20p' "$0"; exit 0 ;;
    *) echo "opção desconhecida: $1" >&2; exit 2 ;;
  esac
done

[[ "$force" -eq 1 ]] || { echo "erro: restauração destrutiva; repita com --force" >&2; exit 2; }
[[ -n "$backup" && -f "$backup" ]] || { echo "erro: informe --backup de um arquivo existente" >&2; exit 2; }
for ferramenta in pg_restore psql; do
  command -v "$ferramenta" >/dev/null 2>&1 || {
    echo "erro: $ferramenta não encontrado no PATH" >&2
    exit 1
  }
done

if [[ "$verificar_hash" -eq 1 ]]; then
  if [[ -f "$backup.sha256" ]]; then
    (cd "$(dirname "$backup")" && sha256sum -c "$(basename "$backup").sha256")
  else
    echo "aviso: sidecar $backup.sha256 ausente; seguindo sem verificação" >&2
  fi
fi

tabelas="$(psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -tAc \
  "SELECT count(*) FROM information_schema.tables WHERE table_schema NOT IN ('pg_catalog','information_schema');")"
if [[ "$tabelas" -gt 0 ]]; then
  if [[ "$limpar" -ne 1 ]]; then
    echo "erro: destino já contém $tabelas tabela(s); use --limpar ou um banco vazio" >&2
    exit 1
  fi
  echo "Limpando schema public do destino ($tabelas tabelas) ..."
  psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -c 'DROP SCHEMA public CASCADE; CREATE SCHEMA public;'
fi

echo "Restaurando $backup ..."
pg_restore \
  --dbname="$DATABASE_URL" \
  --clean --if-exists \
  --no-owner --no-privileges \
  --exit-on-error --single-transaction \
  "$backup"

total="$(psql "$DATABASE_URL" -tAc \
  "SELECT count(*) FROM information_schema.tables WHERE table_schema NOT IN ('pg_catalog','information_schema');")"
migration="$(psql "$DATABASE_URL" -tAc \
  "SELECT coalesce(max(version), 'sem migrations') FROM __diesel_schema_migrations;")"

echo "Restauração concluída: $total tabela(s); migration mais recente: $migration"
echo "Valide a aplicação contra o banco restaurado antes de qualquer promoção."
