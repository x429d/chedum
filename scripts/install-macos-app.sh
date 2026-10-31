#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SOURCE="$ROOT/src-tauri/target/release/bundle/macos/Chenal.app"
DESTINATION="/Applications/Chenal.app"

if [[ ! -d "$SOURCE" ]]; then
  echo "Chenal.app не собран. Сначала: npm run tauri build -- --bundles app" >&2
  exit 1
fi

# Remove only stale or duplicate copies of Chenal.app, leaving other apps untouched.
for directory in /Applications "$HOME/Applications"; do
  if [[ -d "$directory" ]]; then
    find "$directory" -maxdepth 1 -name "Chenal.app" -type d -exec rm -rf {} + 2>/dev/null || true
  fi
done

ditto "$SOURCE" "$DESTINATION"

# Keep Spotlight from indexing the build copy, so search only surfaces /Applications.
touch "$ROOT/src-tauri/target/.metadata_never_index"

echo "Установлено: $DESTINATION"
