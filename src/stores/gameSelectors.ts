import type { BoardBadge, GameDetail, MoveEvaluation } from "../types/models";
import type { ArrowSpec } from "../types/models";
import { buildArrows, groupMoves } from "../utils/moveView";
import { START_FEN, targetSquare } from "../utils/chess";
import { formatEval } from "../utils/format";
import type { VariationState } from "./gameHistory";
import type { PreviewState } from "./gamePreview";

export function initialFenFor(data: GameDetail | null): string {
  if (!data) return START_FEN;
  return data.moves[0]?.fenBefore ?? data.game.initialFen ?? START_FEN;
}

export function currentFenFor(
  initialFen: string,
  move: MoveEvaluation | null,
  variation: VariationState | null,
): string {
  if (variation) return variation.fens[variation.fens.length - 1] ?? initialFen;
  return move?.fenAfter ?? initialFen;
}

export function boardFenFor(currentFen: string, preview: PreviewState | null): string {
  return preview ? preview.fens[preview.index] ?? currentFen : currentFen;
}

export function lastMoveUciFor(
  move: MoveEvaluation | null,
  variation: VariationState | null,
  preview: PreviewState | null,
): string | null {
  if (preview) return null;
  if (variation) return variation.ucis[variation.ucis.length - 1] ?? null;
  return move?.playedUci ?? null;
}

export function badgeFor(
  move: MoveEvaluation | null,
  variation: VariationState | null,
  preview: PreviewState | null,
): BoardBadge | null {
  if (preview || variation || !move) return null;
  const square = targetSquare(move.playedUci);
  return square ? { square, category: move.classification } : null;
}

export function evalLabelFor(move: MoveEvaluation | null): string {
  return move ? formatEval(move.evalCp, move.evalMate) : "0.00";
}

export function arrowsFor(
  move: MoveEvaluation | null,
  showArrows: boolean,
  variation: VariationState | null,
  preview: PreviewState | null,
): ArrowSpec[] {
  if (!showArrows || preview || variation) return [];
  return buildArrows(move);
}

export function movePairsFor(moves: MoveEvaluation[]) {
  return groupMoves(moves);
}
