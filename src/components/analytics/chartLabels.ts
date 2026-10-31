import { tPhase } from "../../i18n";
import type { Phase } from "../../types/models";

const LABEL_FONT_SIZE = 9;
const LABEL_GAP = 6;
const LABEL_INSET = 5;
/** Средняя ширина символа в долях кегля (SF Pro / Inter). */
const AVG_GLYPH_RATIO = 0.58;

export interface PhaseLabelStart {
  phase: Phase;
  ply: number;
}

export interface PlacedPhaseLabel {
  phase: Phase;
  /** Левый край подписи в координатах viewBox. */
  x: number;
  anchor: "start";
}

/// Ширина подписи оценивается по числу символов.
export function estimateLabelWidth(text: string): number {
  return text.length * LABEL_FONT_SIZE * AVG_GLYPH_RATIO;
}

/// Раскладка подписей фаз в одну строку над графиком.
export function layoutPhaseLabels(
  starts: PhaseLabelStart[],
  plot: { left: number; right: number },
  xOf: (ply: number) => number,
): PlacedPhaseLabel[] {
  const placed: PlacedPhaseLabel[] = [];
  let cursor = plot.left;
  for (const start of starts) {
    const width = estimateLabelWidth(tPhase(start.phase));
    const boundary = xOf(start.ply);
    // Порядок важен: сперва место после границы, затем — перед ней.
    const candidates = [boundary + LABEL_INSET, boundary - LABEL_INSET - width];
    const left = candidates.find((x) => x >= cursor && x + width <= plot.right);
    if (left === undefined) continue;
    placed.push({ phase: start.phase, x: left, anchor: "start" });
    cursor = left + width + LABEL_GAP;
  }
  return placed;
}
