import { Chess } from "chessops/chess";
import { makeFen, parseFen } from "chessops/fen";
import { makeSanAndPlay } from "chessops/san";
import type { Color, Move, Square } from "chessops/types";
import {
  makeSquare,
  makeUci,
  parseSquare,
  parseUci,
  squareFile,
  squareRank,
} from "chessops/util";

export const START_FEN = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

export function positionFromFen(fen: string): Chess | null {
  const setup = parseFen(fen);
  if (!setup.isOk) return null;
  const pos = Chess.fromSetup(setup.value);
  return pos.isOk ? pos.value : null;
}

export function legalTargets(fen: string, from: string): string[] {
  const pos = positionFromFen(fen);
  const square = parseSquare(from);
  if (!pos || square === undefined) return [];
  return Array.from(pos.dests(square)).map((sq) => makeSquare(sq));
}

export interface PlayedMove {
  uci: string;
  san: string;
  fenAfter: string;
}

export function playMove(fen: string, fromName: string, toName: string): PlayedMove | null {
  const pos = positionFromFen(fen);
  const from = parseSquare(fromName);
  const to = parseSquare(toName);
  if (!pos || from === undefined || to === undefined) return null;
  const role = pos.board.getRole(from);
  if (!role) return null;
  const lastRank = pos.turn === "white" ? 7 : 0;
  const promotion = role === "pawn" && squareRank(to) === lastRank ? "queen" : undefined;
  const move: Move = promotion ? { from, to, promotion } : { from, to };
  if (!pos.isLegal(move)) return null;
  const san = makeSanAndPlay(pos, move);
  return { uci: makeUci(move), san, fenAfter: makeFen(pos.toSetup()) };
}

export function applyUci(fen: string, uci: string): PlayedMove | null {
  const pos = positionFromFen(fen);
  const move = parseUci(uci);
  if (!pos || !move || !pos.isLegal(move)) return null;
  const san = makeSanAndPlay(pos, move);
  return { uci, san, fenAfter: makeFen(pos.toSetup()) };
}

export function applyLine(fen: string, uciMoves: string[]): string[] {
  const fens: string[] = [fen];
  let current = fen;
  for (const uci of uciMoves) {
    const played = applyUci(current, uci);
    if (!played) break;
    current = played.fenAfter;
    fens.push(current);
  }
  return fens;
}

export function sideToMove(fen: string): Color {
  return fen.split(" ")[1] === "b" ? "black" : "white";
}

export function checkSquare(fen: string): string | null {
  const pos = positionFromFen(fen);
  if (!pos || !pos.isCheck()) return null;
  const king = pos.board.kingOf(pos.turn);
  return king === undefined ? null : makeSquare(king);
}

export function targetSquare(uci: string | null): string | null {
  if (!uci || uci.length < 4) return null;
  return uci.slice(2, 4);
}

export function squareCoords(name: string): { file: number; rank: number } | null {
  const square = parseSquare(name);
  if (square === undefined) return null;
  return { file: squareFile(square), rank: squareRank(square) };
}

export function squareFromCoords(file: number, rank: number): string | null {
  if (file < 0 || file > 7 || rank < 0 || rank > 7) return null;
  return makeSquare((rank * 8 + file) as Square);
}

export function displaySquare(square: string | null): string {
  return square ?? "—";
}
