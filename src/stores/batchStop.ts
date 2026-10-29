import { api } from "../services/api";
import { toErrorMessage } from "../utils/format";
import { ui } from "./uiStore";

/** Ждёт, пока воркеры завершат текущие партии и очередь отпустит движок. */
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
 * Останавливает пакетный анализ только после явного согласия пользователя
 * и дожидается, пока воркеры отпустят движок.
 */
export async function stopBatchWithConfirm(
  running: () => boolean,
  cancel: () => Promise<void>,
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
  ui.notify("Останавливаю пакетный анализ…", "info");
  await cancel();
  if (!(await waitForIdle(running))) {
    ui.notify("Не удалось остановить очередь анализа", "error");
    return false;
  }
  return true;
}
