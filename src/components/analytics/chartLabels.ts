import { tPhase } from "../../i18n";
import type { Phase } from "../../types/models";

const LABEL_FONT_SIZE = 9;
const LABEL_GAP = 6;
const LABEL_INSET = 5;
/** Average glyph width as a fraction of the font size (SF Pro / Inter). */
const AVG_GLYPH_RATIO = 0.58;

export interface PhaseLabelStart {
  phase: Phase;
  ply: number;
}

export interface PlacedPhaseLabel {
  phase: Phase;
  /** Left edge of the label in viewBox coordinates. */
  x: number;
  anchor: "start";
}

export function estimateLabelWidth(text: string): number {
  return text.length * LABEL_FONT_SIZE * AVG_GLYPH_RATIO;
}

/// Lays out phase labels in a single row above the chart.
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
    // Order matters: try the space after the boundary first, then before it.
    const candidates = [boundary + LABEL_INSET, boundary - LABEL_INSET - width];
    const left = candidates.find((x) => x >= cursor && x + width <= plot.right);
    if (left === undefined) continue;
    placed.push({ phase: start.phase, x: left, anchor: "start" });
    cursor = left + width + LABEL_GAP;
  }
  return placed;
}
