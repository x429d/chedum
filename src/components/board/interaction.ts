import type { Chess } from "chessops/chess";
import { makeSquare, parseSquare } from "chessops/util";
import { createMemo, createSignal } from "solid-js";

import type { ArrowSpec, Color } from "../../types/models";
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

/** Правая кнопка рисует жёлтую стрелку; цвета как на Chess.com. */
const ARROW_COLOR = "#FACC15";
const ARROW_COLOR_SHIFT = "#22C55E";
const ARROW_COLOR_ALT = "#3B82F6";
const ARROW_COLOR_CTRL = "#EF4444";
const ARROW_WIDTH = 0.14;

function arrowColor(event: PointerEvent): string {
  if (event.ctrlKey || event.metaKey) return ARROW_COLOR_CTRL;
  if (event.altKey) return ARROW_COLOR_ALT;
  if (event.shiftKey) return ARROW_COLOR_SHIFT;
  return ARROW_COLOR;
}

/**
 * Pointer state machine for the board: whole-cell selection, click-to-move,
 * drag-to-move and right-button arrow drawing.
 */
export function createBoardPointer(options: BoardPointerOptions) {
  const [selected, setSelected] = createSignal<string | null>(null);
  const [drag, setDrag] = createSignal<DragState | null>(null);
  const [hover, setHover] = createSignal<string | null>(null);
  const [userArrows, setUserArrows] = createSignal<ArrowSpec[]>([]);
  const [drawingArrow, setDrawingArrow] = createSignal<ArrowSpec | null>(null);

  const pointOf = (event: PointerEvent): { x: number; y: number } | null => {
    const rect = options.board()?.getBoundingClientRect();
    return rect ? boardPoint(rect, event.clientX, event.clientY) : null;
  };

  const squareOf = (event: PointerEvent): string | null => {
    const point = pointOf(event);
    return point ? squareAt(point, options.orientation()) : null;
  };

  const capture = (event: PointerEvent): void => {
    try {
      options.board()?.setPointerCapture(event.pointerId);
    } catch {
      // pointer capture может быть недоступен в некоторых WebView
    }
  };

  const release = (event: PointerEvent): void => {
    try {
      const board = options.board();
      if (board?.hasPointerCapture(event.pointerId)) {
        board.releasePointerCapture(event.pointerId);
      }
    } catch {
      // pointer capture уже мог быть освобождён
    }
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
    if (!options.interactive()) return;
    const square = squareOf(event);
    if (!square) return;
    // Right button: start an arrow. The preview is updated in handleMove.
    if (event.button === 2) {
      if (drag()) return;
      setDrawingArrow({
        from: square,
        to: square,
        color: arrowColor(event),
        width: ARROW_WIDTH,
      });
      capture(event);
      return;
    }
    if (event.button !== 0 || drag() || drawingArrow()) return;
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
    capture(event);
  };

  const handleMove = (event: PointerEvent): void => {
    const square = squareOf(event);
    setHover(square);
    const arrow = drawingArrow();
    if (arrow) {
      if (square) setDrawingArrow({ ...arrow, to: square });
      return;
    }
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

  const finishArrow = (arrow: ArrowSpec, target: string | null): void => {
    if (!target || target === arrow.from) {
      // Правый клик без перетаскивания очищает все стрелки.
      setUserArrows([]);
      return;
    }
    setUserArrows((items) => {
      const existing = items.find(
        (item) => item.from === arrow.from && item.to === target,
      );
      if (existing?.color === arrow.color) {
        return items.filter((item) => item !== existing);
      }
      const without = items.filter((item) => !(item.from === arrow.from && item.to === target));
      return [...without, { ...arrow, to: target }];
    });
  };

  const handleUp = (event: PointerEvent): void => {
    const arrow = drawingArrow();
    if (arrow) {
      // Если кнопку отпустили за доской, оставляем последнюю подсвеченную клетку.
      finishArrow(arrow, squareOf(event) ?? arrow.to);
      setDrawingArrow(null);
      release(event);
      return;
    }
    const current = drag();
    if (!current || event.button !== 0) return;
    const target = squareOf(event);
    if (target && target !== current.from && legal().has(target)) {
      attemptMove(current.from, target, false);
    } else if (target === current.from && current.wasSelected) {
      setSelected(null);
    }
    setDrag(null);
    release(event);
  };

  const reset = (): void => {
    setSelected(null);
    setDrag(null);
    setDrawingArrow(null);
    setUserArrows([]);
  };

  return {
    selected,
    drag,
    hover,
    ownPieces,
    legal,
    userArrows,
    drawingArrow,
    handleDown,
    handleMove,
    handleUp,
    handleCancel: () => {
      setDrag(null);
      setDrawingArrow(null);
    },
    clearHover: () => setHover(null),
    reset,
  };
}
