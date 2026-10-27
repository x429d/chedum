import type { ArrowSpec, MoveEvaluation } from "../types/models";

export interface MovePair {
  number: number;
  white: MoveEvaluation | null;
  black: MoveEvaluation | null;
}

export function buildArrows(move: MoveEvaluation | null): ArrowSpec[] {
  if (!move) return [];
  const result: ArrowSpec[] = [];
  const first = move.pvLines[0];
  const second = move.pvLines[1];
  const best = first?.uciMoves[0] ?? move.bestUci;
  if (best && best.length >= 4) {
    result.push({ from: best.slice(0, 2), to: best.slice(2, 4), color: "#14B8A6", width: 0.15 });
  }
  const secondMove = second?.uciMoves[0];
  if (secondMove && secondMove.length >= 4) {
    result.push({
      from: secondMove.slice(0, 2),
      to: secondMove.slice(2, 4),
      color: "#3B82F6",
      opacity: 0.72,
      width: 0.115,
    });
  }
  if (["inaccuracy", "mistake", "miss", "blunder"].includes(move.classification)) {
    result.push({
      from: move.playedUci.slice(0, 2),
      to: move.playedUci.slice(2, 4),
      color: "#EF4444",
      opacity: 0.9,
      width: 0.13,
    });
  }
  const threat = first?.uciMoves[1];
  if (threat && threat.length >= 4) {
    result.push({
      from: threat.slice(0, 2),
      to: threat.slice(2, 4),
      color: "#F97316",
      dashed: true,
      opacity: 0.8,
      width: 0.115,
    });
  }
  return result;
}

export function groupMoves(list: MoveEvaluation[]): MovePair[] {
  const pairs: MovePair[] = [];
  for (let index = 0; index < list.length; index += 2) {
    pairs.push({
      number: index / 2 + 1,
      white: list[index] ?? null,
      black: list[index + 1] ?? null,
    });
  }
  return pairs;
}
