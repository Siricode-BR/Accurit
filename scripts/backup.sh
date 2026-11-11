#!/usr/bin/env bash
# Gera backup lógico do PostgreSQL do Accurit (formato custom) e valida o dump.
#
# Uso:
#   DATABASE_URL=postgres://usuario:senha@host:5432/accurit scripts/backup.sh [rotulo]
#
# Variáveis:
#   DATABASE_URL  URL do banco de origem (obrigatória)
#   BACKUP_DIR    diretório de saída (padrão: <raiz>/backups)
#   KEEP_DAYS     retenção local em dias; 0 desabilita (padrão: 7)
#
# O pg_dump precisa ser da mesma versão ou mais novo que o servidor. Este script
# não substitui PITR: RPO próximo de zero exige arquivamento de WAL.
set -euo pipefail

raiz="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
: "${DATABASE_URL:?defina DATABASE_URL}"
backup_dir="${BACKUP_DIR:-$raiz/backups}"
keep_days="${KEEP_DAYS:-7}"
rotulo="${1:-}"

for ferramenta in pg_dump pg_restore; do
  command -v "$ferramenta" >/dev/null 2>&1 || {
    echo "erro: $ferramenta não encontrado no PATH" >&2
    exit 1
  }
done

mkdir -p "$backup_dir"
timestamp="$(date -u +%Y%m%dT%H%M%SZ)"
sufixo=""
if [[ -n "$rotulo" ]]; then
  sufixo="-$(printf '%s' "$rotulo" | tr -c 'A-Za-z0-9._-' '-')"
fi
arquivo="$backup_dir/accurit-$timestamp$sufixo.dump"

echo "Gerando backup em $arquivo ..."
pg_dump \
  --dbname="$DATABASE_URL" \
  --format=custom \
  --compress=9 \
  --no-owner \
  --no-privileges \
  --file="$arquivo"

[[ -s "$arquivo" ]] || { echo "erro: dump vazio" >&2; exit 1; }

echo "Validando estrutura do dump ..."
pg_restore --list "$arquivo" | grep -Eq 'TABLE DATA' || {
  echo "erro: dump não contém dados de tabelas" >&2
  exit 1
}

sha256sum "$arquivo" > "$arquivo.sha256"
chmod 600 "$arquivo" "$arquivo.sha256"

if [[ "$keep_days" -gt 0 ]]; then
  find "$backup_dir" -maxdepth 1 -name 'accurit-*.dump' -type f \
    -mtime "+$keep_days" -print -delete | while read -r antigo; do
      echo "Removendo backup antigo: $(basename "$antigo")"
      rm -f "$antigo.sha256"
    done
fi

echo "Backup concluído: $arquivo ($(du -h "$arquivo" | cut -f1))"
cat "$arquivo.sha256"
