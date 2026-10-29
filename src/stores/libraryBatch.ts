import type { Setter } from "solid-js";

import { api } from "../services/api";
import type { BatchState, GameFilter, GameListItem } from "../types/models";
import { pluralRu, toErrorCode, toErrorMessage } from "../utils/format";
import { analysisStore } from "./analysisStore";
import { gameStore } from "./gameStore";
import { settingsStore } from "./settingsStore";
import { ui } from "./uiStore";

/** Ошибки, после которых продолжать очередь бессмысленно. */
const FATAL_CODES = new Set(["engine", "busy", "internal"]);

export interface BatchControllerDeps {
  batch: () => BatchState;
  setBatch: Setter<BatchState>;
  filter: () => GameFilter;
  games: () => GameListItem[];
  selected: () => number[];
  reload: () => Promise<void>;
}

function summaryLabel(
  stopped: boolean,
  fatal: boolean,
  done: number,
  total: number,
  failed: number,
): string {
  if (stopped) return "Очередь остановлена";
  if (fatal) return "Очередь прервана: ошибка движка";
  if (failed > 0) return `Готово: ${done} из ${total}, ошибок ${failed}`;
  return "Анализ завершён";
}

function positionsFor(game: GameListItem | undefined): number {
  return game ? Math.max(1, game.totalPlies + 1) : 0;
}

export function createBatchController(deps: BatchControllerDeps) {
  let cancelled = false;
  let running = false;
  let startedAt = 0;
  let timer: number | undefined;

  function stopTimer(): void {
    if (timer !== undefined) {
      window.clearInterval(timer);
      timer = undefined;
    }
  }

  /**
   * Добавляет к состоянию прошедшее время и оценку остатка.
   * Если известно число позиций очереди, темп считается по позициям:
   * оценка появляется уже на первой партии, а не после первой готовой.
   */
  function withTiming(
    state: BatchState,
    positionsDone: number,
    positionsTotal: number,
  ): BatchState {
    const elapsedSec =
      startedAt > 0
        ? Math.max(0, Math.round((Date.now() - startedAt) / 1000))
        : state.elapsedSec;
    const processed = state.done + state.failed;
    let etaSec: number | null = null;
    if (positionsTotal > 0 && positionsDone > 0 && positionsTotal > positionsDone) {
      etaSec = Math.max(
        0,
        Math.round((elapsedSec / positionsDone) * (positionsTotal - positionsDone)),
      );
    } else if (processed > 0 && state.total > processed) {
      etaSec = Math.round((elapsedSec / processed) * (state.total - processed));
    }
    return { ...state, elapsedSec, etaSec };
  }

  async function analyzeIds(
    ids: number[],
    label: string,
    positionsTotal = 0,
    lowPriority = false,
  ): Promise<void> {
    if (ids.length === 0 || running || deps.batch().running) return;
    if (gameStore.analyzing()) {
      ui.notify("Дождитесь завершения анализа партии", "info");
      return;
    }
    running = true;
    cancelled = false;
    let done = 0;
    let failed = 0;
    let firstError: string | null = null;
    let fatal = false;
    let completedPositions = 0;
    const activePly: number[] = [];
    const activeTotal: number[] = [];
    let nextIndex = 0;
    const positionsDone = () =>
      completedPositions + activePly.reduce((sum, value) => sum + value, 0);
    const percent = () => {
      if (positionsTotal > 0) {
        return Math.min(99.5, (positionsDone() / positionsTotal) * 100);
      }
      return Math.min(99.5, ((done + failed) / ids.length) * 100);
    };
    try {
      // Сначала гасим live-анализ: иначе его stop_engine прилетит уже
      // в первые партии пакета и отменит их.
      await analysisStore.stop();
      const settings = settingsStore.settings();
      const maximum = settingsStore.info()?.maxAnalysisWorkers ?? 1;
      const requested = settings.analysisThreads === 0 ? maximum : settings.analysisThreads;
      const full = Math.max(1, Math.min(ids.length, requested, maximum));
      // Автоанализ после импорта не занимает весь пул.
      const workers = lowPriority ? Math.max(1, Math.floor(full / 2)) : full;
      startedAt = Date.now();
      deps.setBatch({
        running: true,
        current: 0,
        total: ids.length,
        percent: 0,
        label,
        ply: 0,
        plyTotal: positionsTotal,
        done: 0,
        failed: 0,
        elapsedSec: 0,
        etaSec: null,
      });
      stopTimer();
      timer = window.setInterval(() => {
        deps.setBatch((current) =>
          current.running
            ? withTiming(current, positionsDone(), positionsTotal)
            : current,
        );
      }, 1000);
      const runWorker = async (slot: number): Promise<void> => {
        activePly[slot] = 0;
        activeTotal[slot] = 0;
        while (!cancelled) {
          const index = nextIndex;
          nextIndex += 1;
          if (index >= ids.length) break;
          deps.setBatch((current) =>
            withTiming(
              {
                ...current,
                current: done + failed,
                done,
                failed,
                label: `${label}: ${done + failed + 1} из ${ids.length} (${workers} потоков)`,
              },
              positionsDone(),
              positionsTotal,
            ),
          );
          try {
            await api.analyzeGame(
              ids[index],
              settings.analysisDepth,
              settings.multipv,
              (progress) => {
                activePly[slot] = progress.ply;
                activeTotal[slot] = Math.max(activeTotal[slot] ?? 0, progress.total);
                deps.setBatch((current) =>
                  withTiming(
                    {
                      ...current,
                      percent: percent(),
                      label: progress.message,
                      ply: positionsDone(),
                      plyTotal: positionsTotal || progress.total,
                    },
                    positionsDone(),
                    positionsTotal,
                  ),
                );
              },
            );
            done += 1;
            completedPositions += Math.max(activeTotal[slot] ?? 0, activePly[slot] ?? 0, 1);
          } catch (error) {
            if (cancelled) break;
            failed += 1;
            completedPositions += activePly[slot] ?? 0;
            if (firstError === null) firstError = toErrorMessage(error);
            const code = toErrorCode(error);
            if (code !== null && FATAL_CODES.has(code)) {
              fatal = true;
              cancelled = true;
              break;
            }
          } finally {
            activePly[slot] = 0;
            activeTotal[slot] = 0;
          }
        }
      };
      await Promise.all(
        Array.from({ length: workers }, (_, slot) => runWorker(slot)),
      );
    } finally {
      // Очередь обязана разблокироваться при любом сбое.
      stopTimer();
      running = false;
      const stopped = cancelled && !fatal;
      const elapsedSec =
        startedAt > 0 ? Math.max(0, Math.round((Date.now() - startedAt) / 1000)) : 0;
      startedAt = 0;
      deps.setBatch((current) => ({
        ...current,
        running: false,
        percent: stopped || fatal ? current.percent : 100,
        label: summaryLabel(stopped, fatal, done, ids.length, failed),
        ply: 0,
        plyTotal: 0,
        done,
        failed,
        elapsedSec,
        etaSec: null,
      }));
      if (stopped) {
        ui.notify(`Очередь остановлена: ${done} из ${ids.length}`, "info");
      } else if (fatal) {
        ui.notify(firstError ?? "Очередь прервана: движок недоступен", "error");
      } else if (failed > 0) {
        ui.notify(
          `Обработано ${done} из ${ids.length}, ошибок ${failed}: ${firstError ?? "см. журнал"}`,
          "error",
        );
      } else {
        ui.notify(
          `Готово: ${ids.length} ${pluralRu(ids.length, "партия", "партии", "партий")}`,
          "success",
        );
      }
    }
    await deps.reload();
    await settingsStore.refreshEngine();
  }

  /** Все партии без анализа по текущему фильтру, а не только видимая страница. */
  async function analyzePending(notifyEmpty = true, lowPriority = false): Promise<void> {
    try {
      const plan = await api.listPendingAnalysis(deps.filter());
      if (plan.ids.length === 0) {
        if (notifyEmpty) ui.notify("Нет партий, ожидающих анализа", "info");
        return;
      }
      await analyzeIds(plan.ids, "Разбор новых", plan.positions, lowPriority);
    } catch (error) {
      ui.notify(toErrorMessage(error), "error");
    }
  }

  function analyzeSelected(): void {
    const selection = deps.selected();
    if (selection.length > 0) {
      const positions = deps
        .games()
        .filter((game) => selection.includes(game.id))
        .reduce((sum, game) => sum + positionsFor(game), 0);
      void analyzeIds(selection, "Пакетный анализ", positions);
      return;
    }
    void analyzePending();
  }

  function analyzeOne(id: number): void {
    const game = deps.games().find((item) => item.id === id);
    void analyzeIds([id], "Анализ партии", positionsFor(game));
  }

  async function cancel(): Promise<void> {
    cancelled = true;
    deps.setBatch((current) =>
      current.running ? { ...current, label: "Останавливаю очередь…" } : current,
    );
    try {
      await api.stopEngine();
    } catch {
      // движок мог уже завершиться
    }
  }

  return { analyzeIds, analyzePending, analyzeSelected, analyzeOne, cancel };
}
