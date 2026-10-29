#!/bin/bash
# Сводка состояния Chenal для аудита. Только чтение; БД не изменяется.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DB="${CHENAL_DB:-$HOME/Library/Application Support/com.chenal.studio/chenal.db}"

echo "== Chenal audit =="
echo "date: $(date '+%Y-%m-%d %H:%M:%S')"
echo "root: $ROOT"
echo "db: $DB"

if [ -f "$DB" ]; then
  echo
  echo "-- data --"
  sqlite3 "$DB" "SELECT 'games', COUNT(*) FROM games;"
  sqlite3 "$DB" "SELECT 'analyses', COUNT(*) FROM game_analyses;"
  sqlite3 "$DB" "SELECT 'move evaluations', COUNT(*) FROM move_evaluations;"
  echo "configs:"
  sqlite3 "$DB" "SELECT '  ' || engine_name ||
    ' depth ' || depth || ' MultiPV ' || multipv || ' — ' || COUNT(*)
    FROM game_analyses GROUP BY config_key ORDER BY COUNT(*) DESC;"
  echo "key moments:"
  sqlite3 "$DB" "SELECT '  avg=' || printf('%.2f', AVG(key_moments_count)) ||
    ' min=' || MIN(key_moments_count) || ' max=' || MAX(key_moments_count)
    FROM game_analyses;"
  echo "openings:"
  sqlite3 "$DB" "SELECT '  ' || COALESCE(opening_source, 'none') || ' — ' || COUNT(*)
    FROM games GROUP BY opening_source;"
else
  echo
  echo "database not found"
fi

echo
echo "-- files over 300 lines --"
FILES=$(cd "$ROOT" && find src src-tauri/src -type f \( -name '*.rs' -o -name '*.ts' -o -name '*.tsx' \) \
  -exec wc -l {} + | awk '$1 > 300 && $2 != "total" { print "  " $1 " " $2 }')
if [ -n "$FILES" ]; then
  echo "$FILES"
else
  echo "  none"
fi

echo
echo "-- ci --"
if ls "$ROOT"/.github/workflows/*.yml >/dev/null 2>&1; then
  for workflow in "$ROOT"/.github/workflows/*.yml; do
    echo "  $(basename "$workflow")"
  done
else
  echo "  .github/workflows missing"
fi

echo
echo "-- rust tests --"
(cd "$ROOT/src-tauri" && cargo test -- --list 2>/dev/null | grep -c ': test$' | sed 's/^/  count: /')
