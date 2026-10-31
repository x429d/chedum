import { createSignal } from "solid-js";

import { api } from "../services/api";
import type { EngineUpdate } from "../types/models";
import { toErrorMessage } from "../utils/format";
import { ui } from "./uiStore";

const [liveEnabled, setLiveEnabled] = createSignal(false);
const [liveUpdate, setLiveUpdate] = createSignal<EngineUpdate | null>(null);
const [liveLoading, setLiveLoading] = createSignal(false);
let stopCurrent: (() => Promise<void>) | null = null;
let generation = 0;

async function stop(): Promise<void> {
  generation += 1;
  const stopper = stopCurrent;
  stopCurrent = null;
  if (stopper) {
    try {
      await stopper();
    } catch {
      // stopping the engine is idempotent
    }
  }
  setLiveLoading(false);
}

async function start(fen: string, depth: number, multipv: number): Promise<void> {
  await stop();
  const current = ++generation;
  setLiveUpdate(null);
  setLiveLoading(true);
  const session = api.liveAnalyze(fen, depth, multipv, (update) => {
    if (current === generation) setLiveUpdate(update);
  });
  stopCurrent = session.stop;
  try {
    const final = await session.promise;
    if (current === generation) setLiveUpdate(final);
  } catch (error) {
    if (current === generation) ui.notify(toErrorMessage(error), "error");
  } finally {
    if (current === generation) setLiveLoading(false);
  }
}

function toggle(): void {
  setLiveEnabled((value) => !value);
}

function reset(): void {
  setLiveUpdate(null);
}

export const analysisStore = {
  liveEnabled,
  setLiveEnabled,
  toggle,
  liveUpdate,
  liveLoading,
  start,
  stop,
  reset,
};
