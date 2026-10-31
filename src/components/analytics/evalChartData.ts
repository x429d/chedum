import type { MoveEvaluation, Phase, UserColor } from "../../types/models";

export type ChartScope = "user" | "opponent" | "all";

export function inScope(ply: number, scope: ChartScope, userColor?: UserColor): boolean {
  if (scope === "all") return true;
  const isWhiteMove = ply % 2 === 1;
  const userCol = userColor === "black" ? "black" : "white";
  const isUserMove = userCol === "white" ? isWhiteMove : !isWhiteMove;
  return scope === "user" ? isUserMove : !isUserMove;
}

export function markerPoints(
  moves: MoveEvaluation[],
  isPlyInScope: (ply: number) => boolean,
): { move: MoveEvaluation; ply: number }[] {
  return moves
    .map((move, index) => ({ move, ply: index + 1 }))
    .filter(
      ({ move, ply }) =>
        isPlyInScope(ply) &&
        (["brilliant", "great", "miss", "mistake", "blunder"].includes(move.classification) ||
          move.isCritical),
    );
}

export function phaseMarksFor(
  moves: MoveEvaluation[],
  x: (ply: number) => number,
): { phase: Phase; ply: number; x: number }[] {
  const marks: { phase: Phase; ply: number; x: number }[] = [];
  const seen = new Set<Phase>();
  moves.forEach((move, index) => {
    if (!seen.has(move.phase)) {
      seen.add(move.phase);
      marks.push({ phase: move.phase, ply: index + 1, x: x(index + 1) });
    }
  });
  return marks.filter((mark) => mark.phase !== "opening");
}

export function scopedMovesFor(
  moves: MoveEvaluation[],
  isPlyInScope: (ply: number) => boolean,
): MoveEvaluation[] {
  return moves.filter((move) => isPlyInScope(move.ply));
}

export function maxMoveTime(moves: MoveEvaluation[]): number {
  const values = moves
    .map((move) => move.timeSpentSec)
    .filter((value): value is number => value !== null && Number.isFinite(value));
  return Math.max(1, ...values);
}
