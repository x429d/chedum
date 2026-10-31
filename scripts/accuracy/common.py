"""Shared paths for the research scripts.

The app stores its database in the platform application-data directory, and
the Stockfish sidecar is named after the Rust target triple.
"""

import os
import platform
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

_TARGETS = {
    ("Darwin", "arm64"): "aarch64-apple-darwin",
    ("Darwin", "x86_64"): "x86_64-apple-darwin",
    ("Linux", "x86_64"): "x86_64-unknown-linux-gnu",
    ("Linux", "aarch64"): "aarch64-unknown-linux-gnu",
    ("Windows", "AMD64"): "x86_64-pc-windows-msvc",
    ("Windows", "ARM64"): "aarch64-pc-windows-msvc",
}


def default_db() -> Path:
    override = os.environ.get("CHEDUM_DB")
    if override:
        return Path(override)
    if sys.platform == "darwin":
        return Path.home() / "Library/Application Support/com.chedum.studio/chedum.db"
    if os.name == "nt":
        return Path(os.environ.get("APPDATA", "")) / "com.chedum.studio" / "chedum.db"
    data_home = os.environ.get("XDG_DATA_HOME")
    base = Path(data_home) if data_home else Path.home() / ".local/share"
    return base / "com.chedum.studio" / "chedum.db"


def stockfish() -> str:
    override = os.environ.get("CHEDUM_STOCKFISH")
    if override:
        return override
    target = _TARGETS.get((platform.system(), platform.machine()))
    suffix = ".exe" if os.name == "nt" else ""
    if target:
        bundled = ROOT / "src-tauri" / "binaries" / f"stockfish-{target}{suffix}"
        if bundled.is_file():
            return str(bundled)
    return "stockfish"
