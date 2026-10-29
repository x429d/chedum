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
