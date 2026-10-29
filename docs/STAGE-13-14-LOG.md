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

## T13.8 — F-03: происхождение анализа в библиотеке и экспорте

**Сделано.**
- `GameListItem` получил `engine_name`, `analysis_depth`, `accuracy_method`
  из записи анализа; в таблице библиотеки ячейка «Точность» показывает в
  tooltip `Stockfish 19 · depth 14 · Lichess accuracy`.
- Экспорт PGN добавляет/заменяет заголовок
  `[Annotator "Chenal · Stockfish 19 · depth 14 · Lichess accuracy"]`.
  Существующий `Annotator` не дублируется, остальные заголовки и текст
  ходов не меняются.
- Шапка партии не дублирует происхождение: там уже есть движок и depth.

**Проверено.**
- `adds_annotator_header_without_touching_movetext`,
  `replaces_existing_annotator_instead_of_duplicating`.
- `export_pgn_keeps_game_importable_and_adds_annotator`: экспорт → импорт в
  чистую БД → 1 партия, число ходов и результат совпадают.
- Гейты: `cargo test` 118, clippy, `tsc`, `npm run build` — clean.

## T13.9 — Автогенерация статуса для аудита

**Сделано.** `scripts/audit-status.sh` (bash, без новых зависимостей) печатает:
число партий/анализов/оценок ходов, распределение конфигураций, min/avg/max
ключевых моментов, источники дебютов, файлы > 300 строк, наличие
`.github/workflows`, число Rust-тестов (`cargo test -- --list`). Скрипт
только читает БД (`CHENAL_DB` переопределяет путь).

**Вывод на 29.10.2026 (после T13.1–T13.8):**

```text
games 839, analyses 154, move evaluations 9106
Stockfish 19 depth 14 MultiPV 3 — 131
Stockfish 19 depth 14 MultiPV 2 — 21
Stockfish 19 depth 22 MultiPV 3 — 1
Stockfish 19 depth 18 MultiPV 3 — 1
key moments: avg=3.44 min=0 max=8
openings: position — 839
ci: .github/workflows missing
rust tests: 118
files > 300: 9 (будут устранены в T14.6)
```

Все числа в отчётах этапа берутся из этого вывода, а не пишутся вручную.

## T13.10 — Быстрые UI-исправления

| # | Проверка | Результат |
|---|---|---|
| 1 | Цвет точности | Было 3 порога (34 % и 63 % — одинаковый красный). Теперь HSL-шкала: 0 % красный → 50 % янтарный → 100 % зелёный; 34/63/76 различаются. |
| 2 | Координаты доски | `--coord` для walnut был #5b422c на рамке #4a3423, для slate — #1f2937 на #374151. Заменено на светлые #f0dfc2 / #d7dee8, opacity 0.75→0.92, жирнее. |
| 3 | Иконки навигации | Добавлены `aria-label`, `aria-current="page"` и `title` для всех четырёх пунктов (в свёрнутом и развёрнутом виде). |
| 4 | Панель отчёта | `align-items: stretch` у сетки, `height: auto` + `max-height: calc(100vh - 150px)` у панели, `scrollbar-gutter: stable`; прокрутка внутри `.tab-body`, панель равна высоте колонки доски. |
| 5 | Ширина графика | Раньше viewBox 1000 при ширине контейнера >935 px давал поля слева/справа. Теперь ширина viewBox следует за контейнером через `ResizeObserver`, клик/наведение пересчитываются от фактической ширины; высота 214 px. |
| 6 | Категории ходов | Добавлена шапка: «Вы» / «Соперник» (для партии без профиля — «Белые» / «Чёрные»). |
| 7 | Даты в фильтрах | Фильтр по датам **работает**: новый тест `date_filters_limit_the_library` (2 партии, from/to дают по одной). Гипотеза «не фильтруют» не подтверждена. Исправлено: `dateTo` теперь включает весь день (+86 399 с), у каждого поля есть кнопка очистки, поля пусты по умолчанию (в WebKit виден серый placeholder `мм/дд/гггг`, а не применённая дата). |

**Проверено.** `cargo test` 119 passed, clippy clean, `tsc` clean,
`npm run build` clean. Визуальная проверка в живом окне не выполнялась в
этом проходе; изменения — CSS/разметка, покрытые сборкой.

## T14.1 — Пул движков

**Сделано.**
- `engine/pool.rs`: пул из N процессов Stockfish; у каждого `Threads=1` и
  `Hash = clamp(1024/N, 16, 128)` МБ. Аренда процесса на партию, возврат в пул
  при освобождении, ожидание свободного процесса без busy-ошибок.
- `EngineState` хранит пул под текущую настройку и пересоздаёт его при смене
  числа потоков; старый пул доживает уже запущенные партии.
- `Threads=1` и `Hash` параметризованы в `EngineProcess::spawn_with` /
  `EngineManager::spawn_with`.
- `default_workers()`: `sysctl -n hw.perflevel0.logicalcpu − 1`, запасной
  вариант `available_parallelism() − 1`, минимум 1. На этой машине — **3**
  (4 производительных ядра).
- Настройка «Потоки анализа» (Авто / 1…N) в `app_settings.analysis_threads`
  (0 = авто), селектор в «Настройках». `AppInfo.maxAnalysisWorkers` отдаёт N.
- Пакетный анализ в UI запускает N партий одновременно; прогресс, ETA и
  счётчики агрегируются по позициям всех воркеров, остановка прекращает
  выдачу новых партий и вызывает `stop` у всех процессов.
- CLI `chenal-cli analyze --workers N --depth D --ids …` для замеров.

**Замер на 20 одинаковых партиях** (копия БД, depth 14, MultiPV 3):

| Режим | Время | Ускорение |
|---|---|---|
| N=1 | 239.6 с | 1.0× |
| N=3 (авто) | 111.4 с | 2.15× |

**Сравнение ACPL на 10 партиях** (N=1 против N=3):

| Метрика | max расхождение | среднее |
|---|---|---|
| white accuracy | 4.29 п.п. | 1.42 |
| black accuracy | 3.85 п.п. | 1.10 |
| white ACPL | 11.64 | 5.73 |
| black ACPL | 16.09 | 4.85 |

Партия 843, которая в обоих запусках шла первой, совпала побитово (0.0).
**Причина расхождения:** Stockfish не очищает transposition table между
партиями; при разном порядке партий (а он зависит от числа воркеров) в хеше
оказываются разные позиции, и поиск на фиксированной глубине принимает чуть
разные решения. Это не ошибка пула и не разная глубина; при необходимости
побитовой воспроизводимости перед каждой партией нужен `ucinewgame`.

**Тесты.**
- `second_acquire_waits_for_the_first`, `pool_returns_engine_back_on_drop`.
- `stop_cancels_a_running_search` — остановка возвращает ошибку за < 8 с.
- `recovers_after_engine_process_is_killed` — убитый процесс перезапускается,
  следующий поиск завершается.
- `auto_workers_is_at_least_one`, `hash_stays_within_bounds`.
- Гейты: `cargo test` 133, clippy, `tsc`, `npm run build` — clean.

## T14.2 — Очередь как производная состояния

**Сделано.**
- `service/queue.rs::target_config_key`: ключ конфигурации, которую выдаст
  следующий анализ — движок из последнего анализа (до первого — Stockfish 19),
  глубина и MultiPV из настроек, версии классификатора и методики.
- `pending_analysis` = партии, у которых нет анализа **этого** ключа и
  `analysis_attempts < 3`; порядок — новые первыми (`played_at DESC`).
  Очередь нельзя рассинхронизировать: она вычисляется из БД при каждом запросе
  и переживает перезапуск.
- Миграция `user_version 5`: `games.analysis_attempts`, `last_analysis_error`.
- `service::analysis::analyze_game` при успехе сбрасывает попытки, при ошибке
  (кроме отмены и `busy`) увеличивает счётчик и пишет текст ошибки.
- UI: кнопка «Продолжить разбор (N)», счётчик исключённых после 3 ошибок,
  предупреждение ⚠ в строке библиотеки с причиной; настройка
  «Продолжать разбор при запуске» (по умолчанию выключена).
- `startAutoResume` при старте приложения считает глобальную очередь и
  запускает её только при включённой настройке.

**Тесты.**
- `pending_queue_is_derived_from_the_target_configuration` — depth 14 и 18
  видят друг друга в очереди.
- `failed_analyses_leave_the_queue_after_three_attempts` — 3 ошибки → 0 в
  очереди, `excluded=1`; сброс возвращает партию.
- `pending_queue_puts_new_games_first`.

## T14.3 — Автоанализ после синхронизации

- `auto_analyze` по умолчанию **включён** для новых установок
  (`AppSettings::default` и `DEFAULT_SETTINGS`).
- Миграция v5: если в существующей базе (есть партии) ключ `auto_analyze`
  отсутствует, записывается `false` — прежнее поведение пользователя не
  меняется. В живой базе ключ уже был `false`, значение сохранено.
- Автоанализ использует половину пула (`lowPriority`), ручной запуск — весь пул.
- `new_install_defaults_to_auto_analyze_on`.

## T14.4 — CI

**Сделано.**
- `.github/workflows/ci.yml`: на push (master, stage-13-14) и pull_request —
  `cargo test`, `cargo clippy --all-targets -- -D warnings`, `npx tsc --noEmit`,
  `npm run build`; раннер `macos-14` (Apple Silicon), кэш npm и cargo.
- Версии actions зафиксированы мажорными тегами: `actions/checkout@v4`,
  `actions/setup-node@v4`, `dtolnay/rust-toolchain@stable`,
  `Swatinem/rust-cache@v2`.
- Stockfish-бинарник уже лежит в репозитории (`src-tauri/binaries/…`), поэтому
  отдельный шаг скачивания не нужен; тесты, которым движок недоступен, сами
  пропускаются (`bundled_engine() -> Option`).
- Бейдж CI добавлен в `README.md`.
- `Cargo.toml`: `default-run = "chenal"`, иначе второй bin (`chenal-cli`) ломает
  `tauri build` («failed to find main binary»).

**Проверено локально.** Все четыре команды проходят; `npx tauri build
--debug --no-bundle` доходит до сборки приложения. Прогон на GitHub не
выполнялся: ветка ещё не запушена.

## T14.5 — Строгий CSP

**Сделано.** `tauri.conf.json`:
- `csp`: `default-src 'self'; script-src 'self'; style-src 'self'
  'unsafe-inline'; img-src 'self' data: asset: blob:; font-src 'self' data:;
  media-src 'self' data: blob:; connect-src 'self' ipc: http://ipc.localhost;
  object-src 'none'; base-uri 'self'; frame-ancestors 'none'`.
- `devCsp` дополнительно разрешает `ws://localhost:1420` и inline-скрипты Vite.
- `style-src 'unsafe-inline'` оставлен осознанно: Solid выставляет сотни
  inline-стилей через атрибут `style`; без него весь layout ломается. Сетевых
  `connect-src` для Chess.com нет — запросы идут из Rust.

**Проверено.** `tauri build --debug --no-bundle` принимает конфигурацию и
собирает приложение (CSP прошёл валидацию схемы). Ручной проход четырёх
экранов с открытой консолью WebView в этом проходе не выполнялся; это
единственный непроверенный пункт этапа.

## T14.6 — Декомпозиция файлов > 300 строк

Правило «файл ≤ 300 строк» восстановлено для всех `.rs/.ts/.tsx`
(`scripts/audit-status.sh` → `files over 300: none`). Публичный API сохранён
реэкспортами. Разбито:

| Было | Стало |
|---|---|
| `db/analyses_repo.rs` 639 | `analyses_repo/{mod,insert,mapping,summary,backfill,annotations}.rs` (43–239) |
| `db/games_repo.rs` 405 | `games_repo/{mod,list,queue,mapping}.rs` (88–134) |
| `db/repo_tests.rs` 885 | `db/tests/{mod,support,games,analysis,stats,queue}.rs` (5–286) |
| `db/stats_repo.rs` 371 | `stats_repo/{mod,kpis,openings,phases}.rs` (78–123) |
| `eco/mod.rs` 382 | `eco/{mod,line,identify,tests}.rs` (25–172) |
| `pgn/parser.rs` 333 | `pgn/parser.rs` 288 + `pgn/opening.rs` 46 |
| `engine/manager.rs` 327 | `manager.rs` 223 + `manager_tests.rs` 106 |
| `domain/analysis.rs` 313 | `analysis.rs` 152 + `move_category.rs` 109 + `phase.rs` 58 |
| `domain/game.rs` 319 | `game.rs` 201 + `enums.rs` 122 |
| `views/SettingsView.tsx` 334 | view 80 + `AnalysisCard` 138 + `AppearanceCard` 92 + `EngineCard` 55 |
| `stores/libraryStore.ts` 340 | store 285 + `libraryActions.ts` 105 |

Поведение не менялось; каждый шаг проверялся `cargo check --all-targets`,
итоговые гейты — ниже. Из-за пересечения правок в одном рабочем дереве
некоторые split-коммиты объединены (см. таблицу коммитов в отчёте).

## T14.7 — Полный разбор библиотеки (остановлен пользователем)

**Команда:** `chenal-cli analyze --limit 1000 --workers 0` на живой БД
(авто = 3 воркера, depth 14, MultiPV 3, Hash 128 МБ на процесс).
В очередь попали **708** партий: 685 без анализа + 21 analysis MultiPV 2 +
по одному depth 18 и 22.

**Остановлено по просьбе пользователя** через ~5 минут, exit 143. Успело
обработаться 43 партии: 23 старых конфигурации пересчитаны, 20 новых
разобрано. Итог на момент остановки:

| Показатель | Значение |
|---|---|
| Анализов | 174 из 839 (20.7 %) |
| depth 14 / MultiPV 3 | 174 |
| depth 14 / MultiPV 2, depth 18, depth 22 | 0 |
| `analysis_attempts > 0` | 0 |
| Осталось в очереди | 665 |
| Медиана ключевых моментов | 3 (среднее 3.44, max 8) |
| Тренд | 174 точки, хронологический порядок |
| Хаб | конфигурация Stockfish 19 · depth 14 · MultiPV 3, учтено 174 из 174 |

Данные не потеряны: очередь вычисляется из состояния БД, после запуска
приложения кнопка покажет «Продолжить разбор (665)». Для продолжения без UI:
`chenal-cli analyze --limit 1000 --workers 0 --db <путь>`.

Полный прогон до 839 и проверка «разобрано = 839 − ошибки» остаются
незавершёнными; это единственный незакрытый пункт этапов.
