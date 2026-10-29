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
    let positionsBefore = 0;
    let seenPositions = 0;
    let gamePly = 0;
    try {
      // Сначала гасим live-анализ: иначе его stop_engine прилетит уже
      // в первую партию пакета и отменит её.
      await analysisStore.stop();
      const settings = settingsStore.settings();
      startedAt = Date.now();
      deps.setBatch({
        running: true,
        current: 0,
        total: ids.length,
        percent: 0,
        label,
        ply: 0,
        plyTotal: 0,
        done: 0,
        failed: 0,
        elapsedSec: 0,
        etaSec: null,
      });
      stopTimer();
      timer = window.setInterval(() => {
        deps.setBatch((current) =>
          current.running
            ? withTiming(current, positionsBefore + gamePly, positionsTotal)
            : current,
        );
      }, 1000);
      for (let index = 0; index < ids.length; index += 1) {
        if (cancelled) break;
        seenPositions = 0;
        gamePly = 0;
        deps.setBatch((current) =>
          withTiming(
            {
              ...current,
              current: index,
              done,
              failed,
              ply: 0,
              plyTotal: 0,
              label: `${label}: ${index + 1} из ${ids.length}`,
            },
            positionsBefore,
            positionsTotal,
          ),
        );
        try {
          await api.analyzeGame(
            ids[index],
            settings.analysisDepth,
            settings.multipv,
            (progress) => {
              seenPositions = Math.max(seenPositions, progress.total);
              gamePly = progress.ply;
              deps.setBatch((current) =>
                withTiming(
                  {
                    ...current,
                    percent: ((index + progress.percent / 100) / ids.length) * 100,
                    label: progress.message,
                    ply: progress.ply,
                    plyTotal: progress.total,
                  },
                  positionsBefore + progress.ply,
                  positionsTotal,
                ),
              );
            },
          );
          done += 1;
          positionsBefore += Math.max(seenPositions, gamePly, 1);
        } catch (error) {
          if (cancelled) break;
          failed += 1;
          positionsBefore += gamePly;
          if (firstError === null) firstError = toErrorMessage(error);
          const code = toErrorCode(error);
          if (code !== null && FATAL_CODES.has(code)) {
            fatal = true;
            break;
          }
        }
      }
    } finally {
      // Очередь обязана разблокироваться при любом сбое.
      stopTimer();
      running = false;
      const stopped = cancelled;
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
  async function analyzePending(notifyEmpty = true): Promise<void> {
    try {
      const plan = await api.listPendingAnalysis(deps.filter());
      if (plan.ids.length === 0) {
        if (notifyEmpty) ui.notify("Нет партий, ожидающих анализа", "info");
        return;
      }
      await analyzeIds(plan.ids, "Разбор новых", plan.positions);
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
