import { t } from "../i18n";
import type { BatchState } from "../types/models";

export function summaryLabel(
  stopped: boolean,
  fatal: boolean,
  done: number,
  total: number,
  failed: number,
): string {
  if (stopped) return t("library.queueStopped");
  if (fatal) return t("library.queueAbortedEngine");
  if (failed > 0) return t("library.queueDoneCount", { done, total, failed });
  return t("library.analysisFinished");
}

/** Adds elapsed time and an ETA. When the position count is known the rate is
 * based on positions, so the ETA appears during the first game. */
export function withBatchTiming(
  state: BatchState,
  startedAt: number,
  positionsDone: number,
  positionsTotal: number,
): BatchState {
  const elapsedSec =
    startedAt > 0 ? Math.max(0, Math.round((Date.now() - startedAt) / 1000)) : state.elapsedSec;
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
