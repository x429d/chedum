import type { ArrowSpec } from "../types/models";

export interface VariationState {
  fens: string[];
  sans: string[];
  ucis: string[];
  basePly: number;
}

/** Board state captured before an undoable action. */
export interface BoardSnapshot {
  variation: VariationState | null;
  ply: number;
  arrows: ArrowSpec[];
}

export interface HistoryDeps {
  variation: () => VariationState | null;
  ply: () => number;
  arrows: () => ArrowSpec[];
  setVariation: (value: VariationState | null) => void;
  setPly: (value: number) => void;
  setArrows: (value: ArrowSpec[]) => void;
  stopPreview: () => void;
}

const HISTORY_LIMIT = 200;

/** Undo/redo stack for a game review session. Every mutation records the
 * state it replaced; a new branch clears the redo stack. */
export function createGameHistory(deps: HistoryDeps) {
  const undoHistory: BoardSnapshot[] = [];
  const redoHistory: BoardSnapshot[] = [];

  function snapshot(): BoardSnapshot {
    return { variation: deps.variation(), ply: deps.ply(), arrows: deps.arrows() };
  }

  function record(): void {
    undoHistory.push(snapshot());
    if (undoHistory.length > HISTORY_LIMIT) undoHistory.shift();
    redoHistory.length = 0;
  }

  function apply(state: BoardSnapshot): void {
    deps.stopPreview();
    deps.setVariation(state.variation);
    deps.setPly(state.ply);
    deps.setArrows(state.arrows);
  }

  function undo(): void {
    const previous = undoHistory.pop();
    if (!previous) return;
    redoHistory.push(snapshot());
    apply(previous);
  }

  function redo(): void {
    const next = redoHistory.pop();
    if (!next) return;
    undoHistory.push(snapshot());
    apply(next);
  }

  function reset(): void {
    undoHistory.length = 0;
    redoHistory.length = 0;
  }

  return { record, undo, redo, reset };
}
