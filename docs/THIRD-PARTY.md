# Third-party notices

Chedum bundles or links the following third-party works. The full texts are
shipped in `src-tauri/resources/` and `src/assets/pieces/ATTRIBUTION.md`.

| Component | License | Source |
|---|---|---|
| Stockfish (bundled per-platform binary) | GPL-3.0 | https://github.com/official-stockfish/Stockfish |
| JetBrains Mono Nerd Font | SIL OFL 1.1 | https://github.com/JetBrains/JetBrainsMono |
| ECO opening lines | CC0-1.0 | https://github.com/lichess-org/chess-openings |
| Chess piece sets | GPL-2.0+, Apache-2.0, CC-BY-4.0, MIT, CC0 | see `src-tauri/resources/CHESS-PIECE-LICENSES.txt` |

## Stockfish and GPL

Stockfish is distributed under GPL-3.0. If you distribute a Chedum bundle that
contains the Stockfish binary, you must also provide the corresponding source
code and the GPL notice. Chedum ships the official Stockfish 19 release
(`sf_19`) for macOS, Linux and Windows, fetched by
`npm run fetch:stockfish`. The corresponding source is:

<https://github.com/official-stockfish/Stockfish/tree/sf_19>

The GPL text is bundled at `src-tauri/resources/STOCKFISH-COPYING.txt`.

## Project license

Chedum's own source code is licensed under MIT, see [LICENSE](../LICENSE).
The third-party licenses above apply to their respective components only.
