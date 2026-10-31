#!/usr/bin/env bash
# Downloads the pinned Stockfish build used by the app and tests.
# The binary is intentionally not stored in git.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEST="$ROOT/src-tauri/binaries/stockfish-aarch64-apple-darwin"
VERSION="sf_19"
URL="https://github.com/official-stockfish/Stockfish/releases/download/${VERSION}/stockfish-macos-universal.tar.gz"
SHA256="a1f0e3bcc5a6927a11fe6fc8e54a779754645f3c2bae2cf13420fd1957adaa77"

if [[ -x "$DEST" ]]; then
  echo "Stockfish already present: $DEST"
  exit 0
fi

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

echo "Downloading Stockfish ${VERSION}…"
curl -fL --retry 3 --retry-delay 2 -o "$TMP/stockfish.tar.gz" "$URL"
echo "${SHA256}  $TMP/stockfish.tar.gz" | shasum -a 256 -c -

tar -xzf "$TMP/stockfish.tar.gz" -C "$TMP"
mkdir -p "$(dirname "$DEST")"
cp "$TMP/stockfish/stockfish-macos-universal" "$DEST"
chmod +x "$DEST"
echo "Stockfish installed: $DEST"
