import type { Setter } from "solid-js";

import { pluralGames, t } from "../i18n";
import { api } from "../services/api";
import type { BatchState, GameFilter, GameListItem } from "../types/models";
import { toErrorCode, toErrorMessage } from "../utils/format";
import { analysisProgressLabel } from "../utils/progress";
import { analysisStore } from "./analysisStore";
import { summaryLabel, withBatchTiming } from "./batchTiming";
import { bumpDataVersion } from "./dataVersion";
import { gameStore } from "./gameStore";
import { settingsStore } from "./settingsStore";
import { ui } from "./uiStore";

/** Errors that make continuing the queue pointless. */
const FATAL_CODES = new Set(["engine", "busy", "internal"]);

export interface BatchControllerDeps {
  batch: () => BatchState;
  setBatch: Setter<BatchState>;
  filter: () => GameFilter;
  games: () => GameListItem[];
  selected: () => number[];
  reload: () => Promise<void>;
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

  async function analyzeIds(
    ids: number[],
    label: string,
    positionsTotal = 0,
    lowPriority = false,
  ): Promise<void> {
    if (ids.length === 0 || running || deps.batch().running) return;
    if (gameStore.analyzing()) {
      ui.notify(t("library.waitForGameAnalysis"), "info");
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
      // Stop live analysis first: otherwise its stop_engine would arrive during
      // the batch's first games and cancel them.
      await analysisStore.stop();
      const settings = settingsStore.settings();
      const maximum = settingsStore.info()?.maxAnalysisWorkers ?? 1;
      const requested = settings.analysisThreads === 0 ? maximum : settings.analysisThreads;
      const poolSize = Math.max(1, Math.min(requested, maximum));
      const full = Math.max(1, Math.min(ids.length, requested, maximum));
      // Auto-analysis after import does not use the whole pool.
      const workers = lowPriority ? Math.max(1, Math.floor(full / 2)) : full;
      // The pool has at least two engines per game: idle processes are better
      // spent on positions of a single game than left idle.
      const parallel = !lowPriority && ids.length * 2 <= poolSize;
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
            ? withBatchTiming(current, startedAt, positionsDone(), positionsTotal)
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
            withBatchTiming(
              {
                ...current,
                current: done + failed,
                done,
                failed,
                label: `${label}: ${done + failed + 1} / ${ids.length} (${t("library.threadsCount", { threads: workers })})`,
              },
              startedAt,
              positionsDone(),
              positionsTotal,
            ),
          );
          try {
            await api.analyzeGame(
              ids[index],
              settings.analysisDepth,
              settings.multipv,
              parallel,
              (progress) => {
                activePly[slot] = progress.ply;
                activeTotal[slot] = Math.max(activeTotal[slot] ?? 0, progress.total);
                deps.setBatch((current) =>
                  withBatchTiming(
                    {
                      ...current,
                      percent: percent(),
                      label: analysisProgressLabel(progress),
                      ply: positionsDone(),
                      plyTotal: positionsTotal || progress.total,
                    },
                    startedAt,
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
            const code = toErrorCode(error);
            if (code === "cancelled") {
              cancelled = true;
              break;
            }
            failed += 1;
            completedPositions += activePly[slot] ?? 0;
            if (firstError === null) firstError = toErrorMessage(error);
            if (code !== null && FATAL_CODES.has(code)) {
              fatal = true;
              cancelled = true;
              void api.stopEngine();
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
      // The queue must unblock on any failure.
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
        ui.notify(`${t("library.queueStopped")}: ${done} / ${ids.length}`, "info");
      } else if (fatal) {
        ui.notify(firstError ?? t("library.queueAbortedEngine"), "error");
      } else if (failed > 0) {
        ui.notify(
          `${t("library.done")}: ${done} / ${ids.length}, ${t("library.errors").toLowerCase()}: ${failed}`,
          "error",
        );
      } else {
        ui.notify(
          `${t("library.done")}: ${pluralGames(ids.length)}`,
          "success",
        );
      }
    }
    await deps.reload();
    bumpDataVersion();
    await settingsStore.refreshEngine();
  }

  /** All unanalyzed games matching the current filter, not just the visible page. */
  async function analyzePending(notifyEmpty = true, lowPriority = false): Promise<void> {
    try {
      const plan = await api.listPendingAnalysis(deps.filter());
      if (plan.ids.length === 0) {
        if (notifyEmpty) ui.notify(t("library.queueEmpty"), "info");
        return;
      }
      await analyzeIds(plan.ids, t("library.newAnalysisLabel"), plan.positions, lowPriority);
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
      void analyzeIds(selection, t("library.batchAnalysisLabel"), positions);
      return;
    }
    void analyzePending();
  }

  function analyzeOne(id: number): void {
    const game = deps.games().find((item) => item.id === id);
    void analyzeIds([id], t("library.gameAnalysisLabel"), positionsFor(game));
  }

  async function cancel(): Promise<void> {
    cancelled = true;
    deps.setBatch((current) =>
      current.running ? { ...current, label: t("library.queueStopping") } : current,
    );
    try {
      await api.stopEngine();
    } catch {
      // The engine may already have finished.
    }
  }

  return { analyzeIds, analyzePending, analyzeSelected, analyzeOne, cancel };
}
