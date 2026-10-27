import type { Setter } from "solid-js";

import { api } from "../services/api";
import type { BatchState, GameListItem } from "../types/models";
import { toErrorMessage } from "../utils/format";
import { gameStore } from "./gameStore";
import { settingsStore } from "./settingsStore";
import { ui } from "./uiStore";

export interface BatchControllerDeps {
  batch: () => BatchState;
  setBatch: Setter<BatchState>;
  games: () => GameListItem[];
  selected: () => number[];
  reload: () => Promise<void>;
}

export function createBatchController(deps: BatchControllerDeps) {
  let cancelled = false;

  async function analyzeIds(ids: number[], label: string): Promise<void> {
    if (ids.length === 0 || deps.batch().running) return;
    if (gameStore.analyzing()) {
      ui.notify("Дождитесь завершения анализа партии", "info");
      return;
    }
    cancelled = false;
    const settings = settingsStore.settings();
    deps.setBatch({ running: true, current: 0, total: ids.length, percent: 0, label });
    for (let index = 0; index < ids.length; index += 1) {
      if (cancelled) break;
      deps.setBatch((current) => ({
        ...current,
        current: index,
        label: `${label}: ${index + 1} из ${ids.length}`,
      }));
      try {
        await api.analyzeGame(ids[index], settings.analysisDepth, settings.multipv, (progress) => {
          deps.setBatch((current) => ({
            ...current,
            percent: ((index + progress.percent / 100) / ids.length) * 100,
            label: progress.message,
          }));
        });
      } catch (error) {
        if (!cancelled) ui.notify(toErrorMessage(error), "error");
      }
    }
    const stopped = cancelled;
    deps.setBatch((current) => ({
      ...current,
      running: false,
      percent: stopped ? current.percent : 100,
      label: stopped ? "Очередь остановлена" : "Анализ завершён",
    }));
    if (!stopped) ui.notify(`Готово: ${ids.length} партий`, "success");
    await deps.reload();
    await settingsStore.refreshEngine();
  }

  function analyzeSelected(): void {
    const ids =
      deps.selected().length > 0
        ? deps.selected()
        : deps
            .games()
            .filter((game) => !game.hasAnalysis)
            .map((game) => game.id);
    if (ids.length === 0) {
      ui.notify("Нет партий, ожидающих анализа", "info");
      return;
    }
    void analyzeIds(ids, "Пакетный анализ");
  }

  function analyzeOne(id: number): void {
    void analyzeIds([id], "Анализ партии");
  }

  async function cancel(): Promise<void> {
    cancelled = true;
    try {
      await api.stopEngine();
    } catch {
      // движок мог уже завершиться
    }
    deps.setBatch((current) => ({ ...current, label: "Останавливаю очередь…" }));
  }

  return { analyzeIds, analyzeSelected, analyzeOne, cancel };
}
