# Contributing to Chedum

Thanks for taking the time to contribute. Bug reports, feature ideas, docs and
pull requests are all welcome.

By participating you agree to follow the
[Code of Conduct](CODE_OF_CONDUCT.md) and to keep discussions respectful and
focused on the project. Security issues should follow
[SECURITY.md](SECURITY.md) instead of a public issue.

## Prerequisites

- Node.js 22 or newer
- Rust stable (see `rust-toolchain.toml`)
- Platform toolchain:
  - **macOS**: Xcode Command Line Tools (`xcode-select --install`)
  - **Windows**: Visual Studio Build Tools with the "Desktop development with
    C++" workload and WebView2 (preinstalled on Windows 11)
  - **Linux**: WebKitGTK 4.1 and the usual Tauri dependencies, for example on
    Debian/Ubuntu:

    ```bash
    sudo apt-get install libwebkit2gtk-4.1-dev libappindicator3-dev \
      librsvg2-dev patchelf build-essential curl wget file
    ```

## Setup

```bash
npm install
npm run fetch:stockfish   # pinned Stockfish 19 for the current platform
npm run tauri dev
```

The Stockfish binary is not stored in git. The fetch script verifies the
pinned SHA-256 checksum and writes
`src-tauri/binaries/chedum-stockfish-<target-triple>` (`.exe` on Windows), which is
what Tauri's `externalBin` expects.

If you already have Stockfish installed, Chedum also finds it on `PATH`, and
`CHEDUM_STOCKFISH=/path/to/stockfish` overrides discovery.

## Checks

Run the same gates as CI before opening a pull request:

```bash
cd src-tauri
cargo fmt --all -- --check
cargo test
cargo clippy --all-targets -- -D warnings

cd ..
npm test
npx tsc --noEmit
npm run build
npm audit --omit=dev
```

## Building bundles

```bash
npm run tauri build
```

Tauri produces the native format for the host: `.app`/`.dmg` on macOS,
`.deb`/`.AppImage` on Linux and `.msi`/NSIS `.exe` on Windows. Release builds
are reproducible from the matrix in `.github/workflows/release.yml`.

## Code guidelines

- TypeScript is strict; frontend code talks to Rust only through
  `src/services/`.
- Keep business logic out of `lib.rs`, `main.rs`, command handlers and
  `App.tsx`.
- New source files stay under 300 lines.
- The UI is localized in English and Russian; user-facing strings go through
  the i18n layer, and the backend returns structured tags instead of prose.
- Database access is blocking work; call it through `Db::run` rather than on
  the async runtime.
- Prefer focused commits in the Conventional Commits style
  (`fix(engine): …`, `feat(library): …`, `docs: …`).

## Pull requests

Describe what changed and why, list the platforms you tested on, and mention
any follow-up work. If the change touches analysis or accuracy math, include
the command or fixture you used to verify it. CI runs on macOS, Linux and
Windows, so a green run is expected before review.
