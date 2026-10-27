import { createEffect, createMemo, For, on, Show } from "solid-js";
import type { Piece } from "chessops/types";
import { makeSquare, squareFile, squareRank } from "chessops/util";
import type { ArrowSpec, BoardBadge, BoardTheme, Color, PieceSet } from "../../types/models";
import { FILES } from "../../utils/boardGeometry";
import { positionFromFen } from "../../utils/chess";
import { pieceImage } from "../../utils/pieceSets";
import { Arrows } from "./Arrows";
import { CategoryBadge } from "./CategoryBadge";
import { createBoardPointer } from "./interaction";
import { createPieceMotion } from "./pieceMotion";

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

export interface BoardProps {
  fen: string;
  orientation: Color;
  theme: BoardTheme;
  pieceSet: PieceSet;
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
  const position = createMemo(() => positionFromFen(props.fen));
  const motion = createPieceMotion({
    fen: () => props.fen,
    orientation: () => props.orientation,
    position,
  });
  const pointer = createBoardPointer({
    board: () => boardRef,
    position,
    orientation: () => props.orientation,
    interactive: () => Boolean(props.interactive),
    onMove: (from, to, animate) => {
      if (!animate) motion.suppressNext();
      props.onMove?.(from, to);
    },
  });
  const { selected, drag, hover, ownPieces, legal } = pointer;

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

  createEffect(
    on(
      () => props.fen,
      () => {
        pointer.reset();
      },
      { defer: true },
    ),
  );

  const dragPiece = createMemo(() => {
    const current = drag();
    if (!current) return null;
    const item = motion.pieces().find((entry) => !entry.state().fading && entry.state().square === current.from);
    return item?.state() ?? null;
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

  const hoverCell = createMemo(() => {
    if (!props.interactive) return null;
    const name = hover();
    if (!name) return null;
    return cells().find((cell) => cell.name === name) ?? null;
  });

  const squareCursor = (cell: Cell): string => {
    if (!props.interactive) return "default";
    if (selected() && legal().has(cell.name)) return "pointer";
    if (ownPieces().has(cell.name)) return "grab";
    return "default";
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
        onPointerDown={pointer.handleDown}
        onPointerMove={pointer.handleMove}
        onPointerUp={pointer.handleUp}
        onPointerCancel={pointer.handleCancel}
        onPointerLeave={pointer.clearHover}
      >
        <rect x="-0.25" y="-0.25" width="8.5" height="8.5" fill="var(--board-frame, #10141d)" />
        <For each={cells()}>
          {(cell) => (
            <rect
              x={cell.col}
              y={cell.row}
              width="1"
              height="1"
              class={squareClass(cell)}
              style={{ cursor: squareCursor(cell) }}
            />
          )}
        </For>

        <Show when={hoverCell()}>
          {(cell) => (
            <rect x={cell().col} y={cell().row} width="1" height="1" class="board-sq-hover" />
          )}
        </Show>

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

        <For each={motion.pieces()}>
          {(item) => (
            <g
              classList={{ "board-piece-enter": item.state().entering, "board-piece-fading": item.state().fading }}
              style={{
                transform: `translate(${item.state().col}px, ${item.state().row}px)`,
                transition: `transform ${item.state().duration}ms cubic-bezier(0.2, 0.7, 0.3, 1), opacity 170ms ease-out`,
                opacity: item.state().fading ? 0 : drag()?.from === item.state().square ? 0.32 : 1,
                "pointer-events": "none",
              }}
            >
              <image
                href={pieceImage(props.pieceSet, item.state().color, item.state().role)}
                x="0.035"
                y="0.035"
                width="0.93"
                height="0.93"
                preserveAspectRatio="xMidYMid meet"
              />
            </g>
          )}
        </For>

        <Show when={dragPiece()}>
          {(item) => (
            <g
              style={{
                transform: `translate(${(drag()?.x ?? 0) - 0.46}px, ${(drag()?.y ?? 0) - 0.46}px)`,
                opacity: 0.92,
                "pointer-events": "none",
              }}
            >
              <image
                href={pieceImage(props.pieceSet, item().color, item().role)}
                width="0.93"
                height="0.93"
                preserveAspectRatio="xMidYMid meet"
              />
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
