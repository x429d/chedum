import { createEffect, createMemo, createSignal, For, on, Show } from "solid-js";
import type { Piece } from "chessops/types";
import { makeSquare, parseSquare, squareFile, squareRank } from "chessops/util";
import type { ArrowSpec, BoardBadge, BoardTheme, Color } from "../../types/models";
import { positionFromFen } from "../../utils/chess";
import { Arrows } from "./Arrows";
import { CategoryBadge } from "./CategoryBadge";
import { PieceGlyph } from "./Pieces";
const FILES = "abcdefgh";
interface Cell {
  name: string;
  col: number;
  row: number;
  light: boolean;
}

interface BoardPiece {
  name: string;
  piece: Piece;
  col: number;
  row: number;
}

interface DragState {
  from: string;
  x: number;
  y: number;
}

export interface BoardProps {
  fen: string;
  orientation: Color;
  theme: BoardTheme;
  lastMove?: string | null;
  arrows?: ArrowSpec[];
  badge?: BoardBadge | null;
  checkSquare?: string | null;
  interactive?: boolean;
  embedded?: boolean;
  fullscreen?: boolean;
  onMove?: (from: string, to: string) => void;
}

export function Board(props: BoardProps) {
  let boardRef: SVGSVGElement | undefined;
  const [selected, setSelected] = createSignal<string | null>(null);
  const [drag, setDrag] = createSignal<DragState | null>(null);
  const position = createMemo(() => positionFromFen(props.fen));

  const cells = createMemo<Cell[]>(() => {
    const list: Cell[] = [];
    for (let row = 0; row < 8; row += 1) {
      for (let col = 0; col < 8; col += 1) {
        const file = props.orientation === "white" ? col : 7 - col;
        const rank = props.orientation === "white" ? 7 - row : row;
        list.push({
          name: `${FILES[file]}${rank + 1}`,
          col,
          row,
          light: (file + rank) % 2 === 1,
        });
      }
    }
    return list;
  });

  const pieces = createMemo<BoardPiece[]>(() => {
    const pos = position();
    if (!pos) return [];
    const list: BoardPiece[] = [];
    for (const [square, piece] of pos.board) {
      const file = squareFile(square);
      const rank = squareRank(square);
      const col = props.orientation === "white" ? file : 7 - file;
      const row = props.orientation === "white" ? 7 - rank : rank;
      list.push({ name: makeSquare(square), piece, col, row });
    }
    return list;
  });

  const occupied = createMemo(() => new Set(pieces().map((item) => item.name)));
  const legal = createMemo(() => {
    const from = selected();
    const pos = position();
    if (!from || !pos) return new Set<string>();
    const square = parseSquare(from);
    if (square === undefined) return new Set<string>();
    return new Set(Array.from(pos.dests(square)).map((target) => makeSquare(target)));
  });

  createEffect(
    on(
      () => props.fen,
      () => {
        setSelected(null);
        setDrag(null);
      },
      { defer: true },
    ),
  );

  const pointToSquare = (event: PointerEvent): string | null => {
    const rect = boardRef?.getBoundingClientRect();
    if (!rect) return null;
    const x = ((event.clientX - rect.left) / rect.width) * 8;
    const y = ((event.clientY - rect.top) / rect.height) * 8;
    if (x < 0 || y < 0 || x >= 8 || y >= 8) return null;
    const col = Math.floor(x);
    const row = Math.floor(y);
    const file = props.orientation === "white" ? col : 7 - col;
    const rank = props.orientation === "white" ? 7 - row : row;
    return `${FILES[file]}${rank + 1}`;
  };

  const attemptMove = (from: string, to: string): void => {
    if (!props.interactive) return;
    props.onMove?.(from, to);
    setSelected(null);
    setDrag(null);
  };

  const handleRootDown = (event: PointerEvent): void => {
    if (!props.interactive || drag()) return;
    const square = pointToSquare(event);
    if (!square) return;
    if (selected() && legal().has(square)) {
      attemptMove(selected() ?? "", square);
    } else {
      setSelected(null);
    }
  };

  const handlePieceDown = (event: PointerEvent, item: BoardPiece): void => {
    if (!props.interactive) return;
    const pos = position();
    if (!pos || item.piece.color !== pos.turn) return;
    event.preventDefault();
    event.stopPropagation();
    if (selected() === item.name) {
      setSelected(null);
      setDrag(null);
      return;
    }
    setSelected(item.name);
    setDrag({ from: item.name, x: item.col + 0.5, y: item.row + 0.5 });
    try {
      boardRef?.setPointerCapture(event.pointerId);
    } catch {
      // pointer capture может быть недоступен в некоторых WebView
    }
  };

  const handleMove = (event: PointerEvent): void => {
    if (!drag()) return;
    const rect = boardRef?.getBoundingClientRect();
    if (!rect) return;
    const x = ((event.clientX - rect.left) / rect.width) * 8;
    const y = ((event.clientY - rect.top) / rect.height) * 8;
    setDrag((current) => (current ? { ...current, x, y } : null));
  };

  const handleUp = (event: PointerEvent): void => {
    const current = drag();
    if (!current) return;
    const target = pointToSquare(event);
    if (target && target !== current.from && legal().has(target)) {
      attemptMove(current.from, target);
    }
    setDrag(null);
    try {
      if (boardRef?.hasPointerCapture(event.pointerId)) {
        boardRef.releasePointerCapture(event.pointerId);
      }
    } catch {
      // pointer capture уже мог быть освобождён
    }
  };

  const dragPiece = createMemo(() => {
    const current = drag();
    if (!current) return null;
    return pieces().find((item) => item.name === current.from) ?? null;
  });

  const squareClass = (cell: Cell): string => {
    const classes = [cell.light ? "board-sq-light" : "board-sq-dark"];
    const move = props.lastMove;
    if (move && (cell.name === move.slice(0, 2) || cell.name === move.slice(2, 4))) {
      classes.push("board-sq-last");
    }
    if (selected() === cell.name) classes.push("board-sq-selected");
    if (props.checkSquare === cell.name) classes.push("board-sq-check");
    return classes.join(" ");
  };

  const coordFiles = createMemo(() =>
    Array.from({ length: 8 }, (_, col) => ({
      col,
      label: props.orientation === "white" ? FILES[col] : FILES[7 - col],
    })),
  );
  const coordRanks = createMemo(() =>
    Array.from({ length: 8 }, (_, row) => ({
      row,
      label: String(props.orientation === "white" ? 8 - row : row + 1),
    })),
  );

  return (
    <div
      class={`board-wrap theme-${props.theme}`}
      classList={{ fullscreen: props.fullscreen, "board-embed": props.embedded }}
      style={props.embedded ? { width: "100%", "max-width": "none" } : undefined}
    >
      <svg
        ref={boardRef}
        class="board-svg"
        viewBox="-0.25 -0.25 8.5 8.5"
        onPointerDown={handleRootDown}
        onPointerMove={handleMove}
        onPointerUp={handleUp}
        onPointerCancel={() => setDrag(null)}
      >
        <rect x="-0.25" y="-0.25" width="8.5" height="8.5" fill="var(--board-frame, #10141d)" />
        <For each={cells()}>
          {(cell) => <rect x={cell.col} y={cell.row} width="1" height="1" class={squareClass(cell)} />}
        </For>

        <For each={cells()}>
          {(cell) => (
            <Show when={legal().has(cell.name)}>
              <Show
                when={occupied().has(cell.name)}
                fallback={<circle cx={cell.col + 0.5} cy={cell.row + 0.5} r="0.11" class="board-dot" />}
              >
                <circle cx={cell.col + 0.5} cy={cell.row + 0.5} r="0.4" class="board-capture-ring" />
              </Show>
            </Show>
          )}
        </For>

        <For each={pieces()}>
          {(item) => (
            <g
              transform={`translate(${item.col} ${item.row})`}
              style={{
                opacity: drag()?.from === item.name ? 0.32 : 1,
                cursor: props.interactive ? "grab" : "default",
              }}
              onPointerDown={(event) => handlePieceDown(event, item)}
            >
              <svg x="0.035" y="0.035" width="0.93" height="0.93" viewBox="0 0 45 45">
                <PieceGlyph role={item.piece.role} color={item.piece.color} />
              </svg>
            </g>
          )}
        </For>

        <Show when={dragPiece()}>
          {(item) => (
            <g
              transform={`translate(${(drag()?.x ?? 0) - 0.46} ${(drag()?.y ?? 0) - 0.46})`}
              style={{ opacity: 0.92, "pointer-events": "none" }}
            >
              <svg width="0.93" height="0.93" viewBox="0 0 45 45">
                <PieceGlyph role={item().piece.role} color={item().piece.color} />
              </svg>
            </g>
          )}
        </Show>

        <Arrows arrows={props.arrows ?? []} orientation={props.orientation} />
        <Show when={props.badge}>
          {(badge) => (
            <CategoryBadge
              square={badge().square}
              category={badge().category}
              orientation={props.orientation}
            />
          )}
        </Show>

        <For each={coordFiles()}>
          {(item) => (
            <text x={item.col + 0.5} y={8.19} text-anchor="middle" class="board-coord">
              {item.label}
            </text>
          )}
        </For>
        <For each={coordRanks()}>
          {(item) => (
            <text x={-0.12} y={item.row + 0.56} text-anchor="end" class="board-coord">
              {item.label}
            </text>
          )}
        </For>
      </svg>
    </div>
  );
}
