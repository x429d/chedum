# Changelog

## [0.1.0] - 2026-11-01

- Chedum now builds and runs on macOS (Apple Silicon and Intel), Windows and
  Linux. Stockfish is fetched as a per-platform Tauri sidecar, the database
  path and engine lookup are platform-aware, and CI/release workflows cover
  all three operating systems.
- The project is prepared for open source: MIT license, contributing guide,
  security policy, issue and pull-request templates.
- Coach explanations, game summaries, sync progress, and error messages are now localized in
  the UI; the backend stores structured tags and parameters instead of prose.
- Sync results separate new games, duplicates, filtered games, and parse errors. Long syncs can
  be cancelled, and an account's sync history can be reset.
- Settings are validated in one place, including depth, MultiPV, workers, board theme, piece
  set, and language.
- Database migrations run in transactions and create a `VACUUM INTO` backup before upgrading.
- Blocking SQLite work runs on the blocking pool instead of async runtime workers.
- Added structured file logging, a diagnostics command, and a UI error boundary.
- Keyboard and screen-reader support for the board, library rows, evaluation chart, and toasts.
- Added Vitest coverage for localization helpers and summary generation; CI now checks
  `cargo fmt`, frontend tests, and `npm audit`.
- Corrected the bundled font license (JetBrains Mono OFL), subset the fonts to WOFF2
  (13 MB → 316 KB), and removed the Stockfish binary from git in favor of a pinned
  fetch script.

[0.1.0]: https://github.com/x429d/chedum/releases/tag/v0.1.0
