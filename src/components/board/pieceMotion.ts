import { createEffect, createSignal, on, onCleanup } from "solid-js";

import type { Chess } from "chessops/chess";
import type { Piece, Role } from "chessops/types";
import { makeSquare } from "chessops/util";

import type { Color } from "../../types/models";

const FILES = "abcdefgh";
const FADE_MS = 190;
const MAX_CHANGES_TO_ANIMATE = 3;

export interface PieceMotionState {
  role: Role;
  color: Color;
  square: string;
  col: number;
  row: number;
  duration: number;
  fading: boolean;
  entering: boolean;
}

export interface TrackedPiece {
  id: number;
  state: () => PieceMotionState;
  update: (patch: Partial<PieceMotionState>) => void;
}

export interface PieceMotionOptions {
  fen: () => string;
  orientation: () => Color;
  position: () => Chess | null;
}

interface Grid {
  col: number;
  row: number;
}

function gridFor(square: string, orientation: Color): Grid {
  const file = Math.max(0, FILES.indexOf(square[0] ?? ""));
  const rank = Math.max(0, Number(square[1]) - 1);
  return orientation === "white"
    ? { col: file, row: 7 - rank }
    : { col: 7 - file, row: rank };
}

function moveDuration(distance: number): number {
  return Math.round(Math.min(340, 140 + distance * 26));
}

/**
 * Keeps stable piece identities: when a piece changes squares it keeps the
 * same DOM node, so CSS animates the transform between cells.
 */
export function createPieceMotion(options: PieceMotionOptions): {
  pieces: () => TrackedPiece[];
  suppressNext: () => void;
} {
  let nextId = 1;
  let lastOrientation = options.orientation();
  let suppress = false;
  let suppressTimer: number | undefined;
  const fadeTimers = new Set<number>();

  const makePiece = (
    piece: Piece,
    square: string,
    orientation: Color,
    entering: boolean,
  ): TrackedPiece => {
    const grid = gridFor(square, orientation);
    const [state, setState] = createSignal<PieceMotionState>({
      role: piece.role,
      color: piece.color,
      square,
      col: grid.col,
      row: grid.row,
      duration: 0,
      fading: false,
      entering,
    });
    return {
      id: nextId++,
      state,
      update: (patch) => setState((prev) => ({ ...prev, ...patch })),
    };
  };

  const initialPieces = (): TrackedPiece[] => {
    const pos = options.position();
    if (!pos) return [];
    const list: TrackedPiece[] = [];
    for (const [square, piece] of pos.board) {
      list.push(makePiece(piece, makeSquare(square), options.orientation(), true));
    }
    return list;
  };

  const [pieces, setPieces] = createSignal<TrackedPiece[]>(initialPieces());

  const sync = (): void => {
    const pos = options.position();
    const orientation = options.orientation();
    const orientationChanged = orientation !== lastOrientation;
    lastOrientation = orientation;

    if (!pos) {
      setPieces([]);
      return;
    }

    const target = new Map<string, Piece>();
    for (const [square, piece] of pos.board) {
      target.set(makeSquare(square), piece);
    }

    const active = pieces().filter((item) => !item.state().fading);
    const bySquare = new Map<string, TrackedPiece>();
    for (const item of active) bySquare.set(item.state().square, item);

    const used = new Set<number>();
    const settled = new Set<string>();
    for (const [square, piece] of target) {
      const item = bySquare.get(square);
      if (!item) continue;
      const prev = item.state();
      if (prev.role !== piece.role || prev.color !== piece.color) continue;
      const grid = gridFor(square, orientation);
      item.update({ col: grid.col, row: grid.row, duration: 0 });
      used.add(item.id);
      settled.add(square);
    }

    const targetEntries = [...target.entries()].filter(([square]) => !settled.has(square));
    const grids = new Map<string, Grid>();
    for (const [square] of targetEntries) grids.set(square, gridFor(square, orientation));
    const free = active.filter((item) => !used.has(item.id));

    interface Candidate {
      item: TrackedPiece;
      square: string;
      piece: Piece;
      distance: number;
    }

    const matches: Candidate[] = [];
    const matchedSquares = new Set<string>();

    // Match pieces that kept their role first, then promotions (same color).
    for (const promotion of [false, true]) {
      const candidates: Candidate[] = [];
      for (const [square, piece] of targetEntries) {
        if (matchedSquares.has(square)) continue;
        const grid = grids.get(square);
        if (!grid) continue;
        for (const item of free) {
          if (used.has(item.id)) continue;
          const prev = item.state();
          if (prev.color !== piece.color) continue;
          if ((prev.role !== piece.role) !== promotion) continue;
          const distance = Math.abs(prev.col - grid.col) + Math.abs(prev.row - grid.row);
          candidates.push({ item, square, piece, distance });
        }
      }
      candidates.sort((a, b) => a.distance - b.distance);
      for (const candidate of candidates) {
        if (used.has(candidate.item.id) || matchedSquares.has(candidate.square)) continue;
        used.add(candidate.item.id);
        matchedSquares.add(candidate.square);
        matches.push(candidate);
      }
    }

    const newcomers = targetEntries.filter(([square]) => !matchedSquares.has(square));
    const captured = free.filter((item) => !used.has(item.id));
    const animate =
      active.length > 0 &&
      !orientationChanged &&
      !suppress &&
      matches.length + newcomers.length + captured.length <= MAX_CHANGES_TO_ANIMATE;
    suppress = false;

    for (const match of matches) {
      const grid = grids.get(match.square);
      if (!grid) continue;
      const prev = match.item.state();
      const distance = Math.max(Math.abs(prev.col - grid.col), Math.abs(prev.row - grid.row));
      match.item.update({
        role: match.piece.role,
        color: match.piece.color,
        square: match.square,
        col: grid.col,
        row: grid.row,
        duration: animate ? moveDuration(distance) : 0,
        fading: false,
      });
    }

    const added = newcomers.map(([square, piece]) =>
      makePiece(piece, square, orientation, active.length === 0),
    );

    const dropped = new Set<number>();
    if (animate) {
      for (const item of captured) {
        item.update({ fading: true });
        const id = item.id;
        const timer = window.setTimeout(() => {
          fadeTimers.delete(timer);
          setPieces((prev) => prev.filter((entry) => entry.id !== id));
        }, FADE_MS);
        fadeTimers.add(timer);
      }
    } else {
      for (const item of captured) dropped.add(item.id);
    }

    setPieces((prev) => [...prev.filter((entry) => !dropped.has(entry.id)), ...added]);
  };

  createEffect(on(() => [options.fen(), options.orientation()] as const, sync, { defer: true }));

  onCleanup(() => {
    for (const timer of fadeTimers) clearTimeout(timer);
    if (suppressTimer !== undefined) clearTimeout(suppressTimer);
  });

  const suppressNext = (): void => {
    suppress = true;
    if (suppressTimer !== undefined) clearTimeout(suppressTimer);
    suppressTimer = window.setTimeout(() => {
      suppress = false;
      suppressTimer = undefined;
    }, 0);
  };

  return { pieces, suppressNext };
}
