# Chess piece sets

Chenal ships six open-source SVG piece sets from the
[Lichess piece collection](https://github.com/lichess-org/lila/tree/master/public/piece).
The active set is chosen in **Настройки → Набор фигур**.

| Set        | Author                       | License    | Source                                                        |
| ---------- | ---------------------------- | ---------- | ------------------------------------------------------------- |
| cburnett   | Colin M.L. Burnett           | GPLv2+     | https://github.com/lichess-org/lila/tree/master/public/piece/cburnett |
| merida     | Armando Hernandez Marroquin  | GPLv2+     | https://github.com/lichess-org/lila/tree/master/public/piece/merida |
| chessnut   | Alexis Luengas               | Apache 2.0 | https://github.com/lichess-org/lila/tree/master/public/piece/chessnut |
| rhosgfx    | RhosGFX                      | CC0 1.0    | https://github.com/lichess-org/lila/tree/master/public/piece/rhosgfx |
| celtic     | Maurizio Monge               | MIT        | https://github.com/lichess-org/lila/tree/master/public/piece/celtic |
| kiwen-suwi | neverRare                    | CC BY 4.0  | https://github.com/lichess-org/lila/tree/master/public/piece/kiwen-suwi |

The full license texts are bundled with the app in
`src-tauri/resources/CHESS-PIECE-LICENSES.txt` and are listed in Tauri's resource bundle.

`import.meta.glob` in `src/utils/pieceSets.ts` resolves the SVG files at build time, so a set
can be added by dropping twelve files (`wK`, `wQ`, `wR`, `wB`, `wN`, `wP` and the same in
black) into `src/assets/pieces/<set>/` and registering it in `PIECE_SETS` plus the
`PieceSet` union.
