# Security policy

## Supported versions

Chedum is pre-1.0. Security fixes land on the latest release and on `master`;
older releases are not supported.

## Reporting a vulnerability

Please do not open a public issue for a security problem. Use GitHub's private
vulnerability reporting:

<https://github.com/x429d/chedum/security/advisories/new>

Include as much detail as you can:

- Chedum version and operating system
- steps to reproduce, or a proof of concept
- the plain-text diagnostics from **Settings → Diagnostics** (it contains
  paths and engine status only, never game data)
- impact and any suggested fix

You can expect an acknowledgement within a few days. Please give the maintainer
a reasonable window to release a fix before publishing details.

## Scope

Chedum is a local-first desktop app. The main risk areas are:

- PGN and remote-game parsing (Chess.com Published Data API, Lichess export)
- Stockfish process invocation and the bundled engine binary
- SQLite storage and PGN import/export paths

Reports about third-party components (Tauri, SQLite, Stockfish, Rust crates)
are welcome, but please also notify the upstream project.
