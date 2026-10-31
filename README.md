# Chenal — native chess analytics studio

[![CI](https://github.com/x429d/chenal/actions/workflows/ci.yml/badge.svg)](https://github.com/x429d/chenal/actions/workflows/ci.yml)

Local-first macOS (Apple Silicon) chess analytics studio built with Tauri v2, Rust, SolidJS and
Stockfish. Games are stored in local SQLite, analyzed offline, and presented in a Linear/Raycast
inspired dark interface.

## Features

- **Accounts**: connect any number of Chess.com and Lichess profiles, sync incrementally, switch the
  active account or analyze all accounts together; each game keeps its source and account.
- **Human games only**: Lichess AI, Computer and BOT-titled games are filtered out of sync and PGN
  import, and existing engine games are purged by migration.
- **Library**: import/export PGN, Chess.com PubAPI and Lichess export sync with rate limiting and
  incremental state, filters by account, opponent, color, result, time class, ECO, date, analysis,
  favorites and notes, pagination (25/50/100), batch analysis queue and batch deletion.
- **Analysis Studio**: custom SVG board with three themes and six switchable open-source piece sets,
  drag-and-drop and click-to-move, eval bar with live Stockfish score, multi-arrow layer,
  right-drag user arrows (Shift/Alt/Ctrl change colour, knight moves bend «Г»), move-category badges,
  variation exploration, fullscreen, live Stockfish panel, and ←/→ navigation plus Cmd/Ctrl+Z undo and
  Cmd/Ctrl+Shift+Z redo.
- **Deep analytics**: Win% area chart + synchronized move-time histogram, Lichess-style accuracy
  (win-probability loss with volatility windows and harmonic mean), ACPL,
  performance Elo split by account and platform, 11 move categories, phase accuracy,
  time-management flags.
- **No downgrades**: analyses deeper than current settings are treated as complete; re-analysis is
  offered only for games whose analysis is shallower than the current depth/MultiPV.
- **Move Coach**: Russian explanations for brilliant/great/inaccuracy/mistake/miss/blunder moves,
  MultiPV lines and animated main-line playback.
- **Retry Mistake**: trainer for your own mistakes with instant verification against MultiPV 1.
- **Player Hub**: KPI cards, separate Elo rows per account/platform, opening repertoire matrix,
  vulnerable phases, accuracy trend, color splits and time-management statistics for one account
  or all accounts.
- **Embedded data**: 3,865 ECO opening lines and four pre-analyzed demo games (Stockfish depth 14).

## Data

- ECO opening lines: [lichess-org/chess-openings](https://github.com/lichess-org/chess-openings).
- Chess piece sets: [lichess-org/lila](https://github.com/lichess-org/lila/tree/master/public/piece)
  — cburnett and Merida (GPLv2+), Chessnut (Apache 2.0), Kiwen Suwi (CC BY 4.0),
  Celtic (MIT), Rhosgfx (CC0). Full attributions and license texts live in
  `src/assets/pieces/ATTRIBUTION.md` and `src-tauri/resources/CHESS-PIECE-LICENSES.txt`.
- Demo games use classic games with fictional local handles and generated clock comments; all
  evaluations are produced locally by Stockfish at depth 14.
- Interface typeface: [Spectral](https://github.com/productiontype/Spectral) (SIL OFL 1.1),
  bundled locally with its license at `src/assets/fonts/OFL.txt`.

## Stack

- Tauri v2, Rust 2021
- `rusqlite` (bundled SQLite, WAL), `shakmaty`, `pgn-reader`, `reqwest` (rustls), `tokio`, `sha2`
- SolidJS, TypeScript strict, Tailwind CSS v4, `chessops`
- Stockfish 19 (bundled arm64 binary at `src-tauri/binaries/stockfish-aarch64-apple-darwin`)

## Run

```bash
npm install
npm run tauri dev      # development app
npm run tauri build    # macOS .app / .dmg bundle
npm run install:app    # keep exactly one Chenal.app in /Applications
```

If the bundled engine is missing, Chenal also looks for `/opt/homebrew/bin/stockfish` and
`/usr/local/bin/stockfish`. Stockfish is GPLv3; its license is bundled at
`src-tauri/resources/STOCKFISH-COPYING.txt`.

## Verification

```bash
cd src-tauri
cargo test
cargo clippy --all-targets -- -D warnings

cd ..
npx tsc --noEmit
npm run build
```

The local database lives in `~/Library/Application Support/com.chenal.studio/chenal.db`.
The app is offline-first; the network is used only for the public Chess.com Published Data API and
the public Lichess game-export API.

## Architecture

```text
src-tauri/src/
  domain/     pure models, AppError, FEN/material helpers
  analytics/  Win%, CAPS, ACPL, Elo, 11-category classifier, phases, clocks, coach, summary
  engine/     Stockfish process, UCI parser, 75 ms throttled channel, lazy EngineState
  db/         SQLite migrations, accounts and games repositories, SQL aggregations, demo seed
  chesscom/   PubAPI client, 1 req/s limiter, 429 backoff, incremental archive sync
  lichess/    PGN export client, since-based incremental sync
  pgn/        streaming PGN parser, [%clk] extraction, export writer
  service/    evaluation collection, analysis builder, PGN import/export
  commands/   thin Tauri IPC handlers

src/
  types/      strict DTO mirrors and category palette
  services/   single typed Tauri IPC layer (no invoke in UI components)
  stores/     account, library, game, analysis, settings and UI state
  components/ accounts, board, analytics, library, hub, layout, ui
  views/      Library, Analysis Studio, Player Hub, Accounts, Settings
```

All source files stay under 300 lines; business logic is not placed in `lib.rs`, `main.rs`,
command handlers or `App.tsx`.
