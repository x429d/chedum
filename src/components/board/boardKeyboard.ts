import type { Cell } from "./boardModel";

export interface BoardKeyboardDeps {
  interactive: () => boolean;
  cells: () => Cell[];
  focused: () => string;
  setFocused: (square: string) => void;
  selected: () => string | null;
  legal: () => Set<string>;
  ownPieces: () => Set<string>;
  select: (square: string | null) => void;
  playMove: (from: string, to: string) => void;
}

/** Arrow-key navigation over the visual board, with Enter/Space to select
 * or move a piece and Escape to clear the selection. */
export function createBoardKeyboard(deps: BoardKeyboardDeps) {
  return (event: KeyboardEvent): void => {
    if (!deps.interactive()) return;
    const current = deps.cells().find((cell) => cell.name === deps.focused());
    let col = current?.col ?? 4;
    let row = current?.row ?? 4;
    switch (event.key) {
      case "ArrowLeft":
        col = Math.max(0, col - 1);
        break;
      case "ArrowRight":
        col = Math.min(7, col + 1);
        break;
      case "ArrowUp":
        row = Math.max(0, row - 1);
        break;
      case "ArrowDown":
        row = Math.min(7, row + 1);
        break;
      case "Enter":
      case " ": {
        event.preventDefault();
        const from = deps.selected();
        const square = deps.focused();
        if (from && deps.legal().has(square)) {
          deps.playMove(from, square);
        } else if (deps.ownPieces().has(square)) {
          deps.select(square);
        } else {
          deps.select(null);
        }
        return;
      }
      case "Escape":
        event.preventDefault();
        deps.select(null);
        return;
      default:
        return;
    }
    event.preventDefault();
    const next = deps.cells().find((cell) => cell.col === col && cell.row === row);
    if (next) deps.setFocused(next.name);
  };
}
