import { t } from "../i18n";
import { api } from "../services/api";
import { toErrorMessage } from "../utils/format";
import { ui } from "./uiStore";

/** Waits until workers finish their current games and the queue releases the engine. */
function waitForIdle(running: () => boolean, timeoutMs = 30_000): Promise<boolean> {
  if (!running()) return Promise.resolve(true);
  return new Promise((resolve) => {
    const started = Date.now();
    const tick = (): void => {
      if (!running()) {
        resolve(true);
        return;
      }
      if (Date.now() - started >= timeoutMs) {
        resolve(false);
        return;
      }
      window.setTimeout(tick, 50);
    };
    window.setTimeout(tick, 50);
  });
}

/**
 * Stops batch analysis only after explicit user confirmation
 * and waits for the workers to release the engine.
 */
export async function stopBatchWithConfirm(
  running: () => boolean,
  cancel: () => Promise<void> | void,
  message: string,
): Promise<boolean> {
  if (!running()) return true;
  let confirmed = false;
  try {
    confirmed = await api.confirmStopBatch(message);
  } catch (error) {
    ui.notify(toErrorMessage(error), "error");
    return false;
  }
  if (!confirmed) return false;
  ui.notify(t("library.queueStopping"), "info");
  await cancel();
  if (!(await waitForIdle(running))) {
    ui.notify(t("library.queueStopError"), "error");
    return false;
  }
  return true;
}
