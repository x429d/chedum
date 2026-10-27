import type { Color } from "../types/models";

export const FILES = "abcdefgh";

const VIEW_SIZE = 8.5;
const VIEW_MARGIN = 0.25;

export interface BoardPoint {
  x: number;
  y: number;
}

/**
 * Converts a pointer position into board units, where the 8x8 grid spans 0..8.
 * The SVG viewBox is padded by a 0.25 frame, so raw percentages cannot be used.
 */
export function boardPoint(rect: DOMRect, clientX: number, clientY: number): BoardPoint {
  return {
    x: ((clientX - rect.left) / rect.width) * VIEW_SIZE - VIEW_MARGIN,
    y: ((clientY - rect.top) / rect.height) * VIEW_SIZE - VIEW_MARGIN,
  };
}

export function squareAt(point: BoardPoint, orientation: Color): string | null {
  if (point.x < 0 || point.y < 0 || point.x >= 8 || point.y >= 8) return null;
  const col = Math.floor(point.x);
  const row = Math.floor(point.y);
  const file = orientation === "white" ? col : 7 - col;
  const rank = orientation === "white" ? 7 - row : row;
  return `${FILES[file]}${rank + 1}`;
}
