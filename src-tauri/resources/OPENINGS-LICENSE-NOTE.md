# Opening dataset

`eco.json` is the embedded opening dictionary: 3 865 named ECO lines.

- Source: [lichess-org/chess-openings](https://github.com/lichess-org/chess-openings)
- License: CC0 1.0 Universal (public domain dedication)
- Format used here: one JSON object per line with `eco`, `name`, `pgn`.

The file is read at build time by `src-tauri/src/eco/mod.rs` (`include_str!`)
and is not modified by the application. Opening names and ECO codes are
recomputed from positions against this dataset; no names are invented.
