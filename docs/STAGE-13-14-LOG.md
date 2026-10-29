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

## T13.4 — Ключевые моменты

**Воспроизведение.** У партии 843 было «ключевых: 26» на 53 полухода; счётчик
в `build.rs` увеличивался на каждом `is_key_moment()` **или** `is_critical`,
включая ходы соперника и brilliant/great. Retry брал все mistake/blunder/miss
игрока без ограничения и без расстояния.

**Сделано.**
- `analytics/moments.rs`: `select_key_moments` — только ходы игрока, потеря
  Win% ≥ 8, минимум 3 полухода между моментами, максимум 8, приоритет
  равным/выигрышным позициям (+10). Константы именованные.
- Миграция `user_version 3`: `move_evaluations.is_key_moment`.
- `service/build.rs` проставляет флаги после сборки ходов; поле
  `is_key_moment` в DTO и SQL.
- `backfill_key_moments` (ключ `errors-gap3-v2`) пересобирает флаги из
  сохранённых оценок без движка и обновляет `key_moments_count` + резюме.
- Retry и вкладка моментов используют `isKeyMoment`, ходы соперника не
  попадают.

**Проверено.**
- `clean_game_has_no_moments` — 0 ошибок → 0 моментов.
- `ten_blunders_are_capped_at_eight` — 10 зевков → 8.
- `selected_moments_are_at_least_three_plies_apart`.
- `opponent_moves_are_never_selected`, `equal_or_winning_positions_get_priority`,
  `small_losses_below_threshold_are_ignored`.
- `backfill_selects_key_moments_and_updates_summary` — 30 полуходов, 8
  моментов, резюме «Ключевых моментов: 8».
- Живая база после пересчёта: медиана **3**, среднее 3.44, min 0, max 8;
  у партии 843 — **7** вместо 26. Распределение: 0→12, 1→25, 2→26, 3→20,
  4→21, 5→20, 6→11, 7→9, 8→10.
- Гейты: `cargo test` 113, clippy, `tsc`, `npm run build` — clean.

**Примечание.** Медиана 2 при первоначальном пороге только для
mistake/blunder/miss не проходила приёмку 3–8: значимые потери Win% чаще
классифицируются как «неточность». Порог по величине потери, а не по ярлыку,
даёт медиану 3 и сохраняет 0 для чистой партии.

## T13.5 — Размер выборки и подписи базы

**Воспроизведение.** В матрице репертуара «Точность» показывалась без n:
строки с 9 разобранными партиями и с 3 выглядели одинаково. В шапке хаба
смешивались 839 партий (результаты) и 154 анализа (точность/ACPL/Elo) без
подписи. У «Уязвимых фаз» было число ходов, но не число партий.

**Сделано.**
- `OpeningStat` получил `analyzed`, `price` (партии × средняя потеря Win% на
  ходах игрока) и `reliable` (n ≥ 5, константа `MIN_RELIABLE_SAMPLE`).
- Матрица: колонка «Разобрано», строки с n < 5 приглушены, вместо процента —
  «мало данных» с точным значением и n в tooltip.
- Переключатель сортировки «Партии» / «По цене» (по умолчанию — по числу
  партий); бэкенд возвращает до 40 строк, UI показывает 12.
- Карточки хаба: «по всем 839», «по 154 разобранным», Performance Elo —
  «по 154 разобранным · экспериментально».
- «Уязвимые фазы»: `N ходов · M партий`, `PhaseStat.games`.
- Цена дебюта считается по ходам игрока с учётом цвета, а не по всем ходам.

**Проверено.**
- `aggregates_hub_statistics`: 2 партии → `analyzed=2`, `reliable=false`,
  `price>0`, у фазы `games=2`.
- `performance_estimate_requires_twenty_analyses`: 20 партий одного дебюта →
  `analyzed=20`, `reliable=true`, цена > 0.
- Живая база (топ-5 по партиям): B01 Scandinavian 58 партий / разобрано 9 /
  70.8 %; B02 39 / 7 / 65.5 %; C29 39 / 4 / 80.9 % (приглушена);
  D00 27 / 0; C25 26 / 3 / 55.5 % (приглушена).
- Гейты: `cargo test` 113, clippy, `tsc`, `npm run build` — clean.

## T13.6 — F-02: провенанс анализа и одна конфигурация в хабе

**Воспроизведение.** В `game_analyses` не было версий движка/классификатора/
методики, а SQL хаба соединял все анализы подряд. На живой базе: depth 14
MultiPV 3 — 131, depth 14 MultiPV 2 — 21, depth 18 — 1, depth 22 — 1.

**Сделано.**
- Миграция `user_version 4`: `engine_version`, `classifier_version`,
  `accuracy_method_version`, `config_key` (составной ключ
  `движок|версия|depth|multipv|classifier|accuracy`). Старым записям
  проставлены `Stockfish 19`, `19`, `classifier-v1`, `lichess-v1`.
- `EngineManager::version()` берёт версию из `id name Stockfish 19`;
  `GameAnalysis` хранит все поля, `insert_analysis` пишет `config_key`.
- Хаб выбирает самую частую конфигурацию (`select_config`) и фильтрует по
  `config_key` **все** агрегаты: KPI, точность по цветам, дебюты, фазы,
  тренд, тайм-менеджмент, Performance Elo.
- Над хабом — плашка «Конфигурация: Stockfish 19 · depth 14 · MultiPV 3 ·
  учтено 131 из 154» и кнопка «Пересчитать на depth 14 (23)» для партий с
  другой конфигурацией. Кнопка использует текущие настройки и не удаляет
  старые анализы.

**Проверено.**
- `hub_ignores_analyses_from_another_configuration`: два анализа depth 14 +
  один depth 18 → средняя точность не меняется, `analyzed=2`, `total=3`,
  `mismatched_game_ids=[id3]`.
- `aggregates_hub_statistics`: конфигурация depth 14, `included=2`.
- Живая база: `hub` CLI — конфигурация Stockfish 19 · depth 14 · MultiPV 3,
  учтено **131 из 154**, несовпадающие: 21 анализ MultiPV 2 + depth 18 + 22.
- Гейты: `cargo test` 114, clippy, `tsc`, `npm run build` — clean.

**Уточнение находки F-02.** Разнородность глубин оказалась шире: 21 анализ
depth 14 сделан с MultiPV 2, остальные — с MultiPV 3. Строгий ключ
конфигурации исключает и их; кнопка пересчёта покрывает все 23 партии.

## T13.7 — F-05: тренд берёт последние 200 партий

**Воспроизведение.** `db/stats_extra.rs` содержал `ORDER BY g.played_at ASC
LIMIT 200`: после 200 разобранных партий график застыл бы на самых старых.

**Сделано.**
- SQL: подзапрос `ORDER BY played_at DESC, id DESC LIMIT 200`, внешний
  `ORDER BY played_at ASC, id ASC` — UI получает хронологический порядок без
  изменений на фронтенде.
- Учитывается конфигурация T13.6 (`a.config_key = ?1`).
- Партии с `played_at <= 0` исключены.

**Проверено.**
- `trend_keeps_last_two_hundred_games_in_chronological_order`: 300 фикстур +
  партия без даты → 200 точек, первая `played_at=101`, последняя `300`,
  строго возрастает, нулевой даты нет.
- Живая база (`chenal-cli hub`): 131 точка выбранной конфигурации,
  хронологический порядок; первая 78.94 при 1632587812, последняя 96.27 при
  1677405650.
- Гейты: `cargo test` 115, clippy, `tsc`, `npm run build` — clean.
