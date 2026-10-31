import type { Chess } from "chessops/chess";
import type { Piece } from "chessops/types";
import { makeSquare, squareFile, squareRank } from "chessops/util";

import type { Color } from "../../types/models";
import { FILES } from "../../utils/boardGeometry";

export interface Cell {
  name: string;
  col: number;
  row: number;
  light: boolean;
}

export interface BoardPiece {
  name: string;
  piece: Piece;
  col: number;
  row: number;
}

export function buildCells(orientation: Color): Cell[] {
  const list: Cell[] = [];
  for (let row = 0; row < 8; row += 1) {
    for (let col = 0; col < 8; col += 1) {
      const file = orientation === "white" ? col : 7 - col;
      const rank = orientation === "white" ? 7 - row : row;
      list.push({
        name: `${FILES[file]}${rank + 1}`,
        col,
        row,
        light: (file + rank) % 2 === 1,
      });
    }
  }
  return list;
}

export function buildPieces(position: Chess | null, orientation: Color): BoardPiece[] {
  if (!position) return [];
  const list: BoardPiece[] = [];
  for (const [square, piece] of position.board) {
    const file = squareFile(square);
    const rank = squareRank(square);
    const col = orientation === "white" ? file : 7 - file;
    const row = orientation === "white" ? 7 - rank : rank;
    list.push({ name: makeSquare(square), piece, col, row });
  }
  return list;
}

export function coordinateFiles(orientation: Color): { col: number; label: string }[] {
  return Array.from({ length: 8 }, (_, col) => ({
    col,
    label: orientation === "white" ? FILES[col] : FILES[7 - col],
  }));
}

export function coordinateRanks(orientation: Color): { row: number; label: string }[] {
  return Array.from({ length: 8 }, (_, row) => ({
    row,
    label: String(orientation === "white" ? 8 - row : row + 1),
  }));
}
