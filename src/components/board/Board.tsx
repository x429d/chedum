import { createEffect, createMemo, createSignal, For, on, Show } from "solid-js";
import { t } from "../../i18n";
import type { ArrowSpec, BoardBadge, BoardTheme, Color, PieceSet } from "../../types/models";
import { positionFromFen } from "../../utils/chess";
import { pieceImage } from "../../utils/pieceSets";
import { Arrows } from "./Arrows";
import { createBoardKeyboard } from "./boardKeyboard";
import { buildCells, buildPieces, coordinateFiles, coordinateRanks, type Cell } from "./boardModel";
import { CategoryBadge } from "./CategoryBadge";
import { createBoardPointer } from "./interaction";
import { createPieceMotion } from "./pieceMotion";

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
  /** User arrows when their state lives outside the board (in the store). */
  userArrows?: ArrowSpec[];
  onUserArrowsChange?: (arrows: ArrowSpec[]) => void;
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
  /** User arrows live in the store (analysis) or locally in the board (trainer). */
  const arrowsControlled = props.onUserArrowsChange !== undefined;
  const pointer = createBoardPointer({
    board: () => boardRef,
    position,
    orientation: () => props.orientation,
    interactive: () => Boolean(props.interactive),
    arrows: arrowsControlled ? () => props.userArrows ?? [] : undefined,
    onArrowsChange: arrowsControlled
      ? (next) => props.onUserArrowsChange?.(next)
      : undefined,
    onMove: (from, to, animate) => {
      if (!animate) motion.suppressNext();
      props.onMove?.(from, to);
    },
  });
  const { selected, drag, hover, ownPieces, legal, userArrows, drawingArrow } = pointer;
  const [focused, setFocused] = createSignal("e4");
  /** The keyboard cursor is only meaningful while the board is used from the keyboard. */
  const [keyboardCursor, setKeyboardCursor] = createSignal(false);

  createEffect(() => {
    const current = selected();
    if (current) setFocused(current);
  });

  const allArrows = createMemo<ArrowSpec[]>(() => {
    const list = [...(props.arrows ?? []), ...userArrows()];
    const preview = drawingArrow();
    if (preview) list.push({ ...preview, opacity: 0.75 });
    return list;
  });

  const cells = createMemo(() => buildCells(props.orientation));

  const focusedCell = createMemo(() => cells().find((cell) => cell.name === focused()));

  const handleKeyDown = createBoardKeyboard({
    interactive: () => Boolean(props.interactive),
    cells,
    focused,
    setFocused,
    selected,
    legal,
    ownPieces,
    select: pointer.select,
    playMove: pointer.playMove,
  });

  const pieces = createMemo(() => buildPieces(position(), props.orientation));

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

  const hoverCell = createMemo(() => {
    if (!props.interactive) return null;
    const name = hover();
    if (!name) return null;
    return cells().find((cell) => cell.name === name) ?? null;
  });

  /**
   * Highlights live above the checkerboard instead of being merged into the square rects:
   * the board itself only paints one dark plate plus the light squares, so neighbouring
   * squares never leave a hairline of the frame colour between them.
   */
  const highlights = createMemo(() => {
    const list: { cell: Cell; classes: string }[] = [];
    const add = (name: string | null | undefined, classes: string) => {
      if (!name) return;
      const cell = cells().find((entry) => entry.name === name);
      if (cell) list.push({ cell, classes });
    };
    if (props.lastMove) {
      add(props.lastMove.slice(0, 2), "board-sq-last");
      add(props.lastMove.slice(2, 4), "board-sq-last");
    }
    add(selected(), "board-sq-selected");
    add(props.checkSquare, "board-sq-check");
    const hovered = hoverCell();
    if (hovered) {
      list.push({
        cell: hovered,
        classes: hovered.light ? "board-sq-hover on-light" : "board-sq-hover",
      });
    }
    return list;
  });

  const boardCursor = (): string => {
    if (drawingArrow()) return "crosshair";
    const cell = hoverCell();
    if (!cell) return "default";
    if (selected() && legal().has(cell.name)) return "pointer";
    if (ownPieces().has(cell.name)) return "grab";
    return "default";
  };

  const coordFiles = createMemo(() => coordinateFiles(props.orientation));
  const coordRanks = createMemo(() => coordinateRanks(props.orientation));

  return (
    <div
      class={`board-wrap theme-${props.theme}`}
      classList={{ "board-embed": props.embedded }}
      style={props.embedded ? { width: "100%", "max-width": "none" } : undefined}
      onContextMenu={(event) => {
        // The macOS WebView opens its own context menu on right-click ("Reload page").
        event.preventDefault();
        event.stopPropagation();
      }}
    >
      <svg
        ref={boardRef}
        class="board-svg"
        viewBox="-0.25 -0.25 8.5 8.5"
        role="grid"
        aria-label={t("board.label")}
        tabIndex={props.interactive ? 0 : undefined}
        style={{ cursor: boardCursor() }}
        onKeyDown={(event) => {
          setKeyboardCursor(true);
          handleKeyDown(event);
        }}
        onFocus={() => {
          const current = selected();
          if (current) setFocused(current);
          setKeyboardCursor(Boolean(boardRef?.matches(":focus-visible")));
        }}
        onBlur={() => {
          setKeyboardCursor(false);
          pointer.clearHover();
        }}
        onPointerDown={(event) => {
          setKeyboardCursor(false);
          pointer.handleDown(event);
        }}
        onPointerMove={pointer.handleMove}
        onPointerUp={pointer.handleUp}
        onPointerCancel={pointer.handleCancel}
        onPointerLeave={pointer.clearHover}
      >
        <title>{t("board.focusHint")}</title>
        <rect x="-0.25" y="-0.25" width="8.5" height="8.5" fill="var(--board-frame, #10150e)" />
        <rect x="0" y="0" width="8" height="8" class="board-sq-dark" />
        <For each={cells()}>
          {(cell) => (
            <Show when={cell.light}>
              <rect x={cell.col} y={cell.row} width="1" height="1" class="board-sq-light" />
            </Show>
          )}
        </For>

        <For each={highlights()}>
          {(item) => (
            <rect x={item.cell.col} y={item.cell.row} width="1" height="1" class={item.classes} />
          )}
        </For>

        <Show when={props.interactive && keyboardCursor() ? focusedCell() : null}>
          {(cell) => (
            <rect
              x={cell().col}
              y={cell().row}
              width="1"
              height="1"
              class="board-sq-focus"
              aria-hidden="true"
            />
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

        <Arrows arrows={allArrows()} orientation={props.orientation} />
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
