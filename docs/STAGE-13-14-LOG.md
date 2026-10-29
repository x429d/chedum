# Chenal — журнал этапов 13–14

Ветка: `stage-13-14`. Дата начала: 2026-10-29.
Все проверки выполнялись локально на macOS arm64; живая база перед миграциями
скопирована в `chenal.db.bak-20261029`.

## Базовая линия (до изменений)

| Гейт | Результат |
|---|---|
| `cargo test` | 87 passed, 2.78 s |
| `cargo clippy --all-targets -- -D warnings` | clean |
| `npx tsc --noEmit` | clean |
| `npm run build` | clean, 1.20 s |
| Партий в БД | 839 |
| Анализов | 154 (depth 14 — 152, depth 18 — 1, depth 22 — 1) |

## T13.1 — Резюме партии и карточка точности

**Воспроизведение.** Партия `games.id=843` (Wakanov–KKolobe, depth 22):
`game_analyses.white_accuracy = 63.42`, при этом сохранённый `summary_text`
содержал «Точность ваших ходов — 74.6%».

**Причина.** Резюме писалось один раз в `finalize` вместе с анализом. После
перехода на алгоритм Lichess `backfill_lichess_accuracy` пересчитал колонки
точности, но не текст резюме. Числа разошлись: 74.6 — точность до пересчёта,
63.4 — после.

**Сделано.**
- `analytics/summary.rs`: текст собирает `summary_from_analysis(&SummaryContext,
  &GameAnalysis, time_trouble, impulsive)`. Вход — только поля записи анализа
  (точность, ACPL, категории, фазы, число ключевых моментов), никакого
  пересчёта из ходов.
- `service/build_helpers.rs::finalize` сначала собирает `GameAnalysis`, затем
  текст из него.
- `db/analyses_repo.rs::get_analysis` пересобирает текст при каждом чтении,
  поэтому даже старая запись в БД показывает те же числа, что и карточки.
- `backfill_summaries` (ключ `summary_text_version = single-source-v2`)
  пересобирает сохранённый текст для всех анализов без движка; вызывается в
  `lib.rs` при старте.

**Проверено.**
- `summary_numbers_match_analysis_fields` — 63.4 %, ACPL 101.6, категории,
  фаза и число ключевых моментов совпадают с полями структуры.
- `regenerated_summary_follows_updated_accuracy` — после изменения точности
  текст содержит новое значение.
- `summary_text_always_matches_accuracy_fields` — запись с устаревшим текстом
  («74.6») читается как «63.4»; `backfill_summaries` идемпотентен.
- Гейты: `cargo test` 91 passed, clippy clean, `tsc` clean, `npm run build`
  clean.

## T13.2 — Определение дебюта по позиции

**Воспроизведение.** Партия `id=843`: 1.e4 e5 2.Nc3 c6 3.Nf3 Qf6… Заголовок PGN
содержит `ECO "C25"`, тега `Opening` нет, а в библиотеке стояло
«King's Gambit Declined: Falkbeer Countergambit, Charousek Gambit, Morphy Defense».
Причина в `pgn/parser.rs`: `eco` брался из заголовка PGN, `opening_name` — из
позиционного поиска. Два разных источника давали два разных дебюта.

**Вторая причина.** `EcoBook` хранил одну запись на FEN, и при построении
побеждала последняя строка датасета, содержащая позицию. Позицию после 1.e4
содержат 2032 линии, поэтому выигрывала случайная (C99). Проверено отдельным
прогоном: `entries containing position after 1.e4: 2032`.

**Сделано.**
- `eco/mod.rs`: для каждой позиции выбирается самое точное описание —
  минимальный остаток строки (`remaining = длина линии − ply`), затем большая
  глубина, затем устойчивый лексикографический порядок. Добавлен
  `identify_with_ply` (ECO, имя, полуход партии).
- `pgn/parser.rs`: `identify_opening` — ECO и имя всегда из одного источника.
  Позиция первична; заголовки PGN (`ECO`, `Opening`, `ECOUrl`) — только если
  позиция не найдена, с нормализацией (пробелы, хвостовая пунктуация,
  повторные сегменты, slug URL → читаемое имя).
- Миграция `user_version 2`: `games.opening_ply`, `games.opening_source`
  (`position` | `pgn_header`). Добавление колонок идемпотентно.
- `service/openings.rs`: пересчёт всех партий без движка по версии датасета
  (FNV-1a от `eco.json`), идемпотентно; вызывается при старте приложения.
- `chenal-cli openings` — тот же пересчёт для аудита и замеров.

**Проверено.**
- `vienna_game_is_not_falkbeer` — 1.e4 e5 2.Nc3 даёт C25/Vienna Game.
- `ruy_lopez_family_is_c60_to_c99` — 1.e4 e5 2.Nf3 Nc6 3.Bb5 → C60–C99.
- `scandinavian_is_b01`, `queens_gambit_family_is_d`.
- `transposition_gives_the_same_opening` — два порядка ходов в QGD дают
  одинаковый результат.
- `identified_names_are_exact_dataset_entries` — имя всегда совпадает с
  записью датасета, склеек нет.
- `backfill_updates_store_and_is_idempotent` — пересчёт меняет C31→C60 и
  повторный запуск возвращает 0.
- Живая база: 839 партий пересчитано, `opening_source` = `position` у 839.

**Топ-10 матрицы, было → стало** (по числу партий):

| Было | n | Стало | n |
|---|---|---|---|
| B01 Van Geet Opening: Grünfeld Defense | 72 | B01 Scandinavian Defense | 58 |
| B02 Alekhine Defense: Scandinavian Variation, Geschev Gambit | 39 | B02 Alekhine Defense: Scandinavian Variation, Geschev Gambit | 39 |
| C28 King's Gambit Declined: Falkbeer… Morphy Defense | 39 | C29 Vienna Game: Vienna Gambit | 39 |
| C25 King's Gambit Declined: Falkbeer… Morphy Defense | 26 | D00 Queen's Pawn Game: Accelerated London System | 27 |
| B01 Scandinavian Defense: Valencian Variation, Main Line | 24 | C25 Vienna Game | 26 |
| B01 Scandinavian Defense: Richter Variation | 23 | B00 Pirc Defense | 19 |
| B07 Philidor Defense: Lion Variation, Sozin Variation | 18 | B01 Scandinavian Defense: Gubinsky-Melts Defense | 17 |
| B01 Zukertort Opening: Tennison Gambit, Brigg's Trap | 15 | B01 Scandinavian Defense: Modern Variation | 15 |
| D00 Queen's Pawn Game: Accelerated London System, … | 13 | A00 Van Geet Opening: Dougherty Gambit | 13 |
| D02 Queen's Pawn Game: Accelerated London System, … | 13 | A06 Zukertort Opening: Tennison Gambit, Brigg's Trap | 13 |

Названия B02/B00 остаются необычными, но это дословные записи
`lichess-org/chess-openings` для реально встретившихся позиций; в отличие от
прошлого состояния, ECO и название теперь описывают одну и ту же линию.

## T13.3 — Performance Elo

**Формула (была).** `base + (accuracy − 75)·18 − (ACPL − 35)·4.5`, где base —
рейтинг игрока в партии; результат обрезался в 400…3200.

**Калибровка на живой базе** (154 анализа, ники/рейтинги из партий):

| Связь | Pearson | Spearman |
|---|---|---|
| Elo ↔ точность | 0.283 | 0.256 |
| Elo ↔ ACPL | −0.129 | −0.130 |
| Elo ↔ прежний Performance Elo | 0.363 | — |

Средний сдвиг прежней метрики: **−109.6 Elo**, MAE **283.6 Elo**, r² ≈ 0.08.
Связь слабая → по правилу задания метрика убрана с уровня партии.

**Сделано ([РЕШЕНИЕ], вариант «связь слабая»).**
- В отчёте партии вместо Performance Elo — классический **Performance
  Rating по результату**: рейтинг соперника +400 / 0 / −400 за победу /
  ничью / поражение.
- В хабе оценка по точности остаётся только при **n ≥ 20** разобранных
  партиях, с разбросом `± σ` и подписью «оценка по точности,
  экспериментально». При n < 20 — «—» и подсказка.
- `docs/PERFORMANCE-ELO.md`: формула, замеры и ограничения.

**Проверено.**
- `performance_estimate_requires_twenty_analyses` — 20 анализов включают
  метрику, 2 анализа (в `aggregates_hub_statistics`) её скрывают.
- `perf_elo_formula` — прежняя формула остаётся для совместимости хаба.
- Гейты: `cargo test` 105, clippy, `tsc`, `npm run build` — clean.

**Последствие для пользователя.** Числа вида «513» рядом с рейтингом 1021
больше не показываются. В отчёте партии value = рейтинг соперника ±400,
что объясняется подсказкой.
