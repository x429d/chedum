import type { Phase } from "../../types/models";

export const H = 214;
export const ML = 38;
export const MR = 14;
export const MT = 12;
export const EH = 138;
export const HIST_TOP = 166;
export const HIST_H = 38;

export const PHASE_COLORS: Record<Phase, string> = {
  opening: "#2f9e8b",
  middlegame: "#d9a441",
  endgame: "#6d8fe0",
};

export interface ChartPoint {
  ply: number;
  x: number;
  y: number;
  win: number;
}

export function chartX(ply: number, total: number, plotW: number): number {
  return ML + (ply / total) * plotW;
}

export function chartY(win: number): number {
  return MT + (1 - Math.max(0, Math.min(100, win)) / 100) * EH;
}

export function buildLinePath(points: ChartPoint[]): string {
  return points
    .map((point, index) => `${index === 0 ? "M" : "L"} ${point.x.toFixed(2)} ${point.y.toFixed(2)}`)
    .join(" ");
}

export function buildPhaseRuns(
  points: ChartPoint[],
  phaseOf: (ply: number) => Phase,
): { phase: Phase; area: string }[] {
  const bottom = HIST_TOP + HIST_H;
  const runs: { phase: Phase; area: string }[] = [];
  const first = points[0];
  if (!first) return runs;
  let phase = phaseOf(first.ply);
  let start = first;
  let path = `M ${first.x.toFixed(2)} ${first.y.toFixed(2)}`;
  const flush = (endX: number) => {
    runs.push({
      phase,
      area: `${path} L ${endX.toFixed(2)} ${bottom} L ${start.x.toFixed(2)} ${bottom} Z`,
    });
  };
  for (let index = 1; index < points.length; index += 1) {
    const point = points[index];
    if (!point) continue;
    path += ` L ${point.x.toFixed(2)} ${point.y.toFixed(2)}`;
    const next = phaseOf(point.ply);
    if (next === phase) continue;
    flush(point.x);
    phase = next;
    start = point;
    path = `M ${point.x.toFixed(2)} ${point.y.toFixed(2)}`;
  }
  const last = points[points.length - 1];
  flush(last ? last.x : start.x);
  return runs;
}
