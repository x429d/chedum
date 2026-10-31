import { applyLine } from "../utils/chess";

export interface PreviewState {
  fens: string[];
  index: number;
}

/** Animates a short engine line on the board. The controller owns its timers
 * so the store only has to call `stop` on navigation. */
export function createPreviewController(setPreview: (value: PreviewState | null) => void) {
  let previewTimer: number | undefined;
  let previewEndTimer: number | undefined;

  function stop(): void {
    if (previewTimer !== undefined) {
      window.clearInterval(previewTimer);
      previewTimer = undefined;
    }
    if (previewEndTimer !== undefined) {
      window.clearTimeout(previewEndTimer);
      previewEndTimer = undefined;
    }
    setPreview(null);
  }

  function play(fen: string, uciMoves: string[]): void {
    stop();
    const fens = applyLine(fen, uciMoves.slice(0, 4));
    if (fens.length < 2) return;
    let index = 0;
    setPreview({ fens, index });
    previewTimer = window.setInterval(() => {
      index += 1;
      if (index >= fens.length) {
        if (previewTimer !== undefined) window.clearInterval(previewTimer);
        previewTimer = undefined;
        previewEndTimer = window.setTimeout(() => {
          previewEndTimer = undefined;
          setPreview(null);
        }, 950);
        return;
      }
      setPreview({ fens, index });
    }, 680);
  }

  return { play, stop };
}
