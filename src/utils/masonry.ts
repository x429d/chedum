/** Visual gap between dashboard cards, in pixels. */
export const MASONRY_GAP = 16;

export interface MasonrySlot {
  /** 1-based grid column. */
  column: number;
  /** First grid row line the card occupies. */
  start: number;
  /** Number of 1px rows the card covers. */
  span: number;
}

/**
 * Packs measured card heights into the currently shortest column.
 *
 * Rows are 1px tall and the gap is folded into the span, so `start` is the next
 * free row line of the chosen column. This keeps the dashboard free of holes
 * when neighbours have different heights.
 */
export function packMasonry(
  heights: readonly number[],
  columns: number,
  gap: number = MASONRY_GAP,
): MasonrySlot[] {
  const count = Math.max(1, Math.floor(columns));
  const next = new Array<number>(count).fill(1);
  return heights.map((height) => {
    let column = 0;
    for (let candidate = 1; candidate < count; candidate += 1) {
      if ((next[candidate] ?? 0) < (next[column] ?? 0)) column = candidate;
    }
    const start = next[column] ?? 1;
    const measured = Number.isFinite(height) ? height : 0;
    const span = Math.max(1, Math.ceil(measured + gap));
    next[column] = start + span;
    return { column: column + 1, start, span };
  });
}
