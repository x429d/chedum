import { For } from "solid-js";

import type { MoveCategory } from "../../types/models";

interface Glyph {
  /** SVG paths in a 24×24 box. */
  paths: string[];
  /** Stroke width in the 24×24 box (used when the glyph is stroked). */
  stroke: number;
  /** Filled silhouettes read better for some marks. */
  filled?: boolean;
}

/**
 * Move-quality marks: the classic chess annotation punctuation (`!`, `?`, `?!`, …)
 * plus matching pictograms, drawn in one weight so the badges stay consistent.
 */
const GLYPHS: Record<MoveCategory, Glyph> = {
  brilliant: {
    paths: ["M8.4 4.4v9.4", "M15.6 4.4v9.4", "M8.4 19.3h.01", "M15.6 19.3h.01"],
    stroke: 3.2,
  },
  great: { paths: ["M12 4.4v9.4", "M12 19.3h.01"], stroke: 3.6 },
  best: {
    paths: [
      "M12 2.7l2.9 5.9 6.5.9-4.7 4.6 1.1 6.5L12 17.5l-5.8 3.1 1.1-6.5-4.7-4.6 6.5-.9z",
    ],
    stroke: 0,
    filled: true,
  },
  excellent: {
    // Thumb-up silhouette, drawn by us in the Material Icons manner (Apache 2.0).
    paths: [
      "M3.3 9.6h2.7v10.8H3.3z",
      "M8.3 20.4h9.1a2 2 0 0 0 1.94-1.5l1.5-5.9a2 2 0 0 0-1.94-2.5h-4.9l.62-3.2a2.6 2.6 0 0 0-2.56-3.1 1.6 1.6 0 0 0-1.46.9L7.7 9.9a2 2 0 0 0-.2.85v7.85a1.8 1.8 0 0 0 .8 1.8z",
    ],
    stroke: 0,
    filled: true,
  },
  good: { paths: ["M20 6.4 9.4 17 4 11.6"], stroke: 3 },
  book: {
    paths: [
      "M12 7.3v13.4",
      "M3 17.9a1 1 0 0 1-1-1V4.6a1 1 0 0 1 1-1h4.6a4.2 4.2 0 0 1 4.4 4.2 4.2 4.2 0 0 1 4.4-4.2H21a1 1 0 0 1 1 1v12.3a1 1 0 0 1-1 1h-5.6a3.4 3.4 0 0 0-3.4 3.4 3.4 3.4 0 0 0-3.4-3.4z",
    ],
    stroke: 2,
  },
  inaccuracy: {
    paths: [
      "M3.6 8.8a3.1 3.1 0 1 1 4.6 2.7c-.9.6-1.4 1.2-1.4 2.3",
      "M6.1 17.1h.01",
      "M15.4 6.5v6.9",
      "M15.4 17.1h.01",
    ],
    stroke: 2.6,
  },
  mistake: {
    paths: [
      "M7.2 8.9a4.8 4.8 0 1 1 7.2 4.2c-1.4.9-2.4 1.9-2.4 3.6",
      "M12 19.9h.01",
    ],
    stroke: 2.7,
  },
  miss: { paths: ["M6.9 6.9l10.2 10.2", "M17.1 6.9L6.9 17.1"], stroke: 3 },
  blunder: {
    paths: [
      "M2.9 8.7a3 3 0 1 1 4.4 2.6c-.85.6-1.35 1.15-1.35 2.2",
      "M5.2 17h.01",
      "M13 8.7a3 3 0 1 1 4.4 2.6c-.85.6-1.35 1.15-1.35 2.2",
      "M15.3 17h.01",
    ],
    stroke: 2.5,
  },
  // Only move: the move runs into the wall of alternatives.
  forced: { paths: ["M3.6 12h10.8", "M10.6 7.2 15.4 12l-4.8 4.8", "M19.2 5.4v13.2"], stroke: 2.5 },
};

export function CategoryGlyph(props: { category: MoveCategory; size?: number }) {
  const glyph = () => GLYPHS[props.category];
  return (
    <svg
      viewBox="0 0 24 24"
      width={props.size ?? 16}
      height={props.size ?? 16}
      fill={glyph().filled ? "currentColor" : "none"}
      stroke={glyph().filled ? "none" : "currentColor"}
      stroke-width={glyph().stroke}
      stroke-linecap="round"
      stroke-linejoin="round"
      aria-hidden="true"
    >
      <For each={glyph().paths}>{(d) => <path d={d} />}</For>
    </svg>
  );
}
