import type { Chess } from "chessops/chess";
import { makeSquare, parseSquare } from "chessops/util";
import { createMemo, createSignal } from "solid-js";

import type { Color } from "../../types/models";
import { boardPoint, squareAt } from "../../utils/boardGeometry";

export interface DragState {
  from: string;
  x: number;
  y: number;
  grabX: number;
  grabY: number;
  wasSelected: boolean;
}

export interface BoardPointerOptions {
  board: () => SVGSVGElement | undefined;
  position: () => Chess | null;
  orientation: () => Color;
  interactive: () => boolean;
  onMove: (from: string, to: string, animate: boolean) => void;
}

/**
 * Pointer state machine for the board: whole-cell selection, click-to-move and drag-to-move.
 * The square is derived from the pointer position, so the piece glyph is not a hit target.
 */
export function createBoardPointer(options: BoardPointerOptions) {
  const [selected, setSelected] = createSignal<string | null>(null);
  const [drag, setDrag] = createSignal<DragState | null>(null);
  const [hover, setHover] = createSignal<string | null>(null);

  const pointOf = (event: PointerEvent): { x: number; y: number } | null => {
    const rect = options.board()?.getBoundingClientRect();
    return rect ? boardPoint(rect, event.clientX, event.clientY) : null;
  };

  const squareOf = (event: PointerEvent): string | null => {
    const point = pointOf(event);
    return point ? squareAt(point, options.orientation()) : null;
  };

  const ownPieces = createMemo(() => {
    const pos = options.position();
    const names = new Set<string>();
    if (!pos) return names;
    for (const [square, piece] of pos.board) {
      if (piece.color === pos.turn) names.add(makeSquare(square));
    }
    return names;
  });

  const legal = createMemo(() => {
    const from = selected();
    const pos = options.position();
    if (!from || !pos) return new Set<string>();
    const square = parseSquare(from);
    if (square === undefined) return new Set<string>();
    return new Set(Array.from(pos.dests(square)).map((target) => makeSquare(target)));
  });

  const attemptMove = (from: string, to: string, animate = true): void => {
    if (!options.interactive()) return;
    options.onMove(from, to, animate);
    setSelected(null);
    setDrag(null);
  };

  const handleDown = (event: PointerEvent): void => {
    if (!options.interactive() || drag()) return;
    const square = squareOf(event);
    if (!square) return;
    const from = selected();
    if (from && legal().has(square)) {
      attemptMove(from, square);
      return;
    }
    if (!ownPieces().has(square)) {
      setSelected(null);
      return;
    }
    const point = pointOf(event);
    if (!point) return;
    const wasSelected = from === square;
    const centerX = Math.floor(point.x) + 0.5;
    const centerY = Math.floor(point.y) + 0.5;
    setSelected(square);
    setDrag({
      from: square,
      x: centerX,
      y: centerY,
      grabX: point.x - centerX,
      grabY: point.y - centerY,
      wasSelected,
    });
    try {
      options.board()?.setPointerCapture(event.pointerId);
    } catch {
      // pointer capture может быть недоступен в некоторых WebView
    }
  };

  const handleMove = (event: PointerEvent): void => {
    setHover(squareOf(event));
    const current = drag();
    if (!current) return;
    const point = pointOf(event);
    if (!point) return;
    setDrag({
      ...current,
      x: point.x - current.grabX,
      y: point.y - current.grabY,
    });
  };

  const handleUp = (event: PointerEvent): void => {
    const current = drag();
    if (!current) return;
    const target = squareOf(event);
    if (target && target !== current.from && legal().has(target)) {
      attemptMove(current.from, target, false);
    } else if (target === current.from && current.wasSelected) {
      setSelected(null);
    }
    setDrag(null);
    try {
      const board = options.board();
      if (board?.hasPointerCapture(event.pointerId)) {
        board.releasePointerCapture(event.pointerId);
      }
    } catch {
      // pointer capture уже мог быть освобождён
    }
  };

  const reset = (): void => {
    setSelected(null);
    setDrag(null);
  };

  return {
    selected,
    drag,
    hover,
    ownPieces,
    legal,
    handleDown,
    handleMove,
    handleUp,
    handleCancel: () => setDrag(null),
    clearHover: () => setHover(null),
    reset,
  };
}
