# Chenal — progress log

## Stage 1 — Initialization, disk profile, database, ECO

- [x] Tauri v2 + SolidJS + TypeScript + Tailwind v4 project initialized.
- [x] Rust disk diet in `src-tauri/Cargo.toml` (`debug = 0`, `strip`, thin LTO, `opt-level = "s"`).
- [x] Clippy denies for `unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`.
- [x] Stockfish 19 arm64 binary prepared (thin slice) at
      `src-tauri/binaries/stockfish-aarch64-apple-darwin` and made executable.
- [x] SQLite WAL migrations for `synced_archives`, `games`, `game_analyses`, `move_evaluations`,
      `app_settings` plus all required indexes.
- [x] Embedded ECO dictionary: 3,865 openings with ECO code, name and PGN line.
- [x] macOS app icon generated and Tauri bundle configuration prepared.
- **Gate 1:** `cargo test` and `cargo clippy --all-targets -- -D warnings` pass.

## Stage 2 — Math core, PGN/clocks, UCI manager, classifier

- [x] Win probability logistic formula, mate probability and white-relative normalization.
- [x] CAPS accuracy formula, weighted accuracy, ACPL cap at 300 cp, performance Elo.
- [x] Priority classifier for all 11 categories with critical-move detection.
- [x] Opening / middlegame / endgame detector using material and developed pieces.
- [x] Time-management flags (`time_trouble`, `impulsive_error`, `overthink`).
- [x] Streaming `pgn-reader` parser, `[%clk]` / `[%emt]` extraction, PGN writer.
- [x] Async Stockfish manager (`Arc<tokio::sync::Mutex<EngineProcess>>`), UCI parser, 75 ms
      throttled updates, cancellation and guaranteed kill.
- [x] Move Coach explanations in Russian and deterministic game summary.
- **Gate 2:** 74 Rust unit tests pass, including all 11 categories, accuracy, clocks and eval
      normalization.

## Stage 3 — Chess.com PubAPI and local storage

- [x] PubAPI client with required `User-Agent`, 1 request/second limiter, `429`/`Retry-After`
      backoff, archive games import.
- [x] Incremental archive planning: finished months skipped, current month refreshed.
- [x] Deduplication by external id and transactional batch inserts.
- [x] PGN import (files/text), export, notes, favorites, move comments.
- [x] Four rich demo games with real Stockfish depth-14 analysis embedded for first launch.
- **Gate 3:** sync mock tests, dedupe tests, repository and SQL aggregation tests pass.

## Stage 4 — SVG board, variation tree, live analysis

- [x] Vector SVG pieces (original set, later replaced by six open-source sets in Stage 7).
- [x] Custom board with three themes, coordinates, last move, check, legal destinations,
      click-to-move, pointer drag-and-drop, flip and fullscreen.
- [x] Category badges on destination squares.
- [x] Multi-arrow layer: MultiPV 1, MultiPV 2, played error and opponent threat.
- [x] Eval bar with animated white share and exact eval (`+1.45` / `M3`).
- [x] Chessops variation exploration and animated main-line preview.
- [x] Live Stockfish panel with throttled MultiPV updates.
- **Gate 4:** `npx tsc --noEmit` passes with strict TypeScript and no `any`.

## Stage 5 — Visual analytics, Move Coach, Retry Mistake

- [x] Dual-layer chart: Win% area, phase separators, error/brilliant markers, time histogram with
      time-trouble/impulsive highlighting, click-to-ply.
- [x] Game report: summary, ECO, accuracy rings, performance Elo, ACPL, 11-category table with
      filters, phase bars, time-management summary.
- [x] Move Coach card with delta, time, explanation, MultiPV lines and main-line playback.
- [x] Two-column move list, category filtering and user notes.
- [x] Key moments and interactive Retry Mistake trainer with instant verification and solution.
- **Gate 5:** `npm run build` passes.

## Stage 6 — Library, Player Hub, final audit

- [x] Library filters, selection, batch queue with progress/cancel, PGN import/export, sync panel.
- [x] Pagination with 25/50/100 page sizes, stable action column and batch deletion with native confirmation.
- [x] Keyboard navigation: ←/→ step through moves, Home/End jump to start/end.
- [x] Full-width window drag region with double-click maximize.
- [x] Player Analytics Hub: KPI cards, color splits, opening matrix, phase weaknesses, accuracy
      trend, time-management stats.
- [x] Settings: profile, analysis depth/MultiPV, board theme, piece set, arrows, engine diagnostics.
- [x] Headless Playwright smoke test of all views with mocked IPC: no console errors.
- [x] Native macOS `.app` bundle built successfully (105.4 MiB, Stockfish resource executable):
      `src-tauri/target/release/bundle/macos/Chenal.app`.

### Final Gate 6 audit

- [x] No file over 300 lines (`find ... wc -l`).
- [x] No `any`, `as any`, `@ts-ignore`, `@ts-expect-error` in `src/`.
- [x] No `todo!`, `unimplemented!` or `TODO` in `src-tauri/src/`.
- [x] `cargo test` — 74 passed.
- [x] `cargo clippy --all-targets -- -D warnings` — clean.
- [x] `npx tsc --noEmit` — clean.
- [x] `npm run build` — clean.
- [x] `npm run tauri build -- --bundles app` — clean.
- [x] `cargo fmt --check` — clean.

All gates are green. The app is ready to run from the bundle above.

## Stage 7 — Open-source piece sets

- [x] Six switchable SVG sets from lichess-org/lila: Cburnett and Merida (GPLv2+),
      Chessnut (Apache 2.0), Rhosgfx (CC0), Celtic (MIT), Kiwen Suwi (CC BY 4.0).
- [x] `pieceSet` persisted in `app_settings` with validation and a safe fallback for old values.
- [x] Visual picker in **Настройки → Набор фигур** with per-set author and license.
- [x] Attribution file plus full license texts bundled as a Tauri resource.
- **Gate 7:** 75 Rust tests, `cargo clippy -D warnings`, `tsc --noEmit` and `npm run build` pass.

## Stage 8 — Board legibility polish

- [x] Dark piece fills lifted to `#2a2a2a` with black outlines, so тёмные фигуры читаются так же,
      как белые (cburnett).
- [x] Arrows thickened and outlined (`#111827` halo), MultiPV / error / threat arrows made more opaque.
- [x] Emoji move marks replaced by a single vector glyph set (`CategoryGlyph`): `!!`, `!`, `★`, thumbs-up,
      `✓`, open book, `?!`, `?`, `✕`, `??` и «только один ход» (стрелка в стену) — одинаковый вес
      штриха, один вид на доске, в списке ходов, в таблице категорий, в Coach и Retry.
- [x] Все иконки интерфейса переведены на Lucide (ISC, `lucide-solid`): навигация, тулбар доски,
      библиотека, настройки, тосты; единый вес штриха 1.75 и размер по умолчанию 18. Бренд-марка
      получила шахматного коня.
- [x] Новый app icon: исходник `src-tauri/icon-source.svg` (тёмный squircle + бирюзовый градиентный
      конь на плинте), пересобран `tauri icon` во все размеры/icns/ico. Бренд-марка в сайдбаре
      рисуется тем же силуэтом (`BrandMark`), поэтому иконка в доке и логотип совпадают.
- [x] Полоса оценки: белая доля клипуется во внутреннем треке, а цифра (±1.45 / M3) вынесена
      наружу и свисает в жёлоб, поэтому больше не обрезается шириной полосы.
- [x] Действия в строке таблицы партий: три тихие ghost-кнопки 28×28 вместо постоянной красной корзины;
      нейтральный цвет, подсветка на hover (удаление — красное только при наведении), избранное — янтарное,
      «Анализировать» получило читаемую иконку графика вместо мелких искр.
- [x] Таблица партий: карточка получила плоскую заливку (`--card`), поэтому закреплённая колонка действий
      больше не рисует тёмный прямоугольник поверх градиента; ACPL скрывается уже с 1440 px и ячейки
      ужимаются, чтобы таблица не уезжала под закреплённую колонку. Избранное — сплошная янтарная звезда,
      при наведении на пустую звезду появляется янтарная подложка.
- **Gate 8:** `tsc --noEmit` и `npm run build` чистые; UI проверен скриншотами на всех темах доски.

## Stage 9 — King brand mark

- [x] Иконка приложения и бренд-марка переведены с коня на короля: единый силуэт (крест + держава
      + чаша) в `src-tauri/icon-source.svg`, все размеры/`icns`/`ico` пересобраны через `tauri icon`,
      `BrandMark` в сайдбаре рисует те же контуры, поэтому док и логотип совпадают.

## Stage 10 — Board hover and square seams

- [x] Ховер клетки больше не задаётся на каждую тему: на тёмной клетке это белая заливка 16 %,
      на светлой — чёрная 13 % (класс `on-light`). Направление выбирается по светлоте клетки,
      поэтому новые темы и наборы цветов работают сразу, без ручных `--sq-hover`.
- [x] Тёмные полоски по краям клетки: доска теперь рисуется одной тёмной плитой плюс светлыми
      квадратами, а подсветки (последний ход, выбор, шах, ховер) кладутся отдельными `rect`
      сверху. Раньше между 64 квадратами просвечивал цвет рамки, и ховер-оверлей делал этот шов
      чёрной обводкой клетки.

## Stage 11 — Lichess-совместимая точность

- [x] Точность партии переведена на алгоритм Lichess: бонус неопределённости +1 к точности хода,
      скользящие окна волатильности Win%, среднее взвешенного и гармонического среднего.
- [x] Точность по фазам считается тем же методом на ходах каждой фазы.
- [x] Одноразовый пересчёт уже сохранённых анализов без движка
      (`analyses_repo::backfill_lichess_accuracy`, ключ `accuracy_algo = lichess-v1`).
- [x] В интерфейсе метрика подписана как «метод Lichess» с подсказкой об отличии от CAPS2.
- **Gate 11:** `cargo test` (84), `cargo clippy --all-targets -- -D warnings`, `tsc` и `npm run build`
      чистые; расхождение с Chess.com описано в `docs/ACCURACY-PARITY.md`.
