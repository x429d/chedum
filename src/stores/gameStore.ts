import { createSignal } from "solid-js";

import { api } from "../services/api";
import type {
  AnalysisProgress,
  ArrowSpec,
  BoardBadge,
  Color,
  GameDetail,
  MoveCategory,
} from "../types/models";
import { applyLine, checkSquare, playMove, START_FEN, targetSquare } from "../utils/chess";
import { buildArrows, groupMoves } from "../utils/moveView";
import { formatEval, toErrorMessage } from "../utils/format";
import { settingsStore } from "./settingsStore";
import { ui } from "./uiStore";

interface VariationState {
  fens: string[];
  sans: string[];
  ucis: string[];
  basePly: number;
}

interface PreviewState {
  fens: string[];
  index: number;
}

/** Позиция доски для undo/redo: вариант и номер хода в партии. */
interface BoardSnapshot {
  variation: VariationState | null;
  ply: number;
}

const [detail, setDetail] = createSignal<GameDetail | null>(null);
const [loading, setLoading] = createSignal(false);
const [ply, setPly] = createSignal(0);
const [orientation, setOrientation] = createSignal<Color>("white");
const [categoryFilter, setCategoryFilter] = createSignal<MoveCategory | null>(null);
const [variation, setVariation] = createSignal<VariationState | null>(null);
const [preview, setPreview] = createSignal<PreviewState | null>(null);
const [analyzing, setAnalyzing] = createSignal(false);
const [progress, setProgress] = createSignal<AnalysisProgress | null>(null);
let previewTimer: number | undefined;
let previewEndTimer: number | undefined;
let analysisCancelled = false;
/** Состояния, снятые через undo: redo возвращает их по одному. */
const undoHistory: BoardSnapshot[] = [];

const moves = () => detail()?.moves ?? [];
const analysis = () => detail()?.analysis ?? null;
const currentMove = () => {
  const index = ply();
  return index > 0 ? moves()[index - 1] ?? null : null;
};
const initialFen = () => {
  const data = detail();
  if (!data) return START_FEN;
  return data.moves[0]?.fenBefore ?? data.game.initialFen ?? START_FEN;
};
const currentFen = () => {
  const line = variation();
  if (line) return line.fens[line.fens.length - 1] ?? initialFen();
  return currentMove()?.fenAfter ?? initialFen();
};
const boardFen = () => {
  const frames = preview();
  return frames ? frames.fens[frames.index] ?? currentFen() : currentFen();
};
const lastMoveUci = () => {
  if (preview()) return null;
  const line = variation();
  if (line) return line.ucis[line.ucis.length - 1] ?? null;
  return currentMove()?.playedUci ?? null;
};
const badge = (): BoardBadge | null => {
  if (preview() || variation()) return null;
  const move = currentMove();
  if (!move) return null;
  const square = targetSquare(move.playedUci);
  return square ? { square, category: move.classification } : null;
};
const checkSquareName = () => checkSquare(boardFen());
const winProbWhite = () => currentMove()?.winProbWhite ?? 50;
const evalLabel = () => {
  const move = currentMove();
  return move ? formatEval(move.evalCp, move.evalMate) : "0.00";
};
const arrows = (): ArrowSpec[] => {
  if (!settingsStore.settings().showArrows || preview() || variation()) return [];
  return buildArrows(currentMove());
};

const movePairs = () => groupMoves(moves());

async function load(id: number): Promise<void> {
  if (analyzing() && detail()?.game.id === id) return;
  if (analyzing()) {
    analysisCancelled = true;
    void api.stopEngine().catch(() => {});
    setAnalyzing(false);
    setProgress(null);
  }
  setLoading(true);
  try {
    const data = await api.getGame(id);
    setDetail(data);
    setPly(0);
    setVariation(null);
    undoHistory.length = 0;
    stopPreview();
    setCategoryFilter(null);
    setOrientation(data.game.userColor === "black" ? "black" : "white");
  } catch (error) {
    ui.notify(toErrorMessage(error), "error");
    setDetail(null);
  } finally {
    setLoading(false);
  }
}

function goTo(index: number): void {
  undoHistory.length = 0;
  stopPreview();
  setVariation(null);
  setPly(Math.max(0, Math.min(index, moves().length)));
}

function next(): void {
  goTo(ply() + 1);
}
function previous(): void {
  goTo(ply() - 1);
}

function first(): void { goTo(0); }
function last(): void { goTo(moves().length); }


function flip(): void {
  setOrientation((color) => (color === "white" ? "black" : "white"));
}
function toggleCategory(category: MoveCategory): void {
  setCategoryFilter((current) => (current === category ? null : category));
}

function stopPreview(): void {
  if (previewTimer !== undefined) {
    window.clearInterval(previewTimer);
    previewTimer = undefined;
  }
  if (previewEndTimer !== undefined) {
    window.clearTimeout(previewEndTimer);
    previewEndTimer = undefined;
  }
  setPreview(null);
}

function playPreview(fen: string, uciMoves: string[]): void {
  stopPreview();
  const fens = applyLine(fen, uciMoves.slice(0, 4));
  if (fens.length < 2) return;
  let index = 0;
  setPreview({ fens, index });
  previewTimer = window.setInterval(() => {
    index += 1;
    if (index >= fens.length) {
      if (previewTimer !== undefined) window.clearInterval(previewTimer);
      previewTimer = undefined;
      previewEndTimer = window.setTimeout(() => {
        previewEndTimer = undefined;
        setPreview(null);
      }, 950);
      return;
    }
    setPreview({ fens, index });
  }, 680);
}

function playBestLine(): void {
  const move = currentMove();
  const line = move?.pvLines[0];
  if (!move || !line) return;
  playPreview(move.fenBefore, line.uciMoves);
}

function enterVariation(from: string, to: string): boolean {
  stopPreview();
  const baseFen = currentFen();
  const played = playMove(baseFen, from, to);
  if (!played) return false;
  undoHistory.length = 0;
  const existing = variation();
  const fens = existing ? [...existing.fens, played.fenAfter] : [baseFen, played.fenAfter];
  const sans = existing ? [...existing.sans, played.san] : [played.san];
  const ucis = existing ? [...existing.ucis, played.uci] : [played.uci];
  setVariation({ fens, sans, ucis, basePly: ply() });
  return true;
}

function clearVariation(): void {
  undoHistory.length = 0;
  setVariation(null);
}

function snapshot(): BoardSnapshot {
  return { variation: variation(), ply: ply() };
}

function applySnapshot(state: BoardSnapshot): void {
  stopPreview();
  setVariation(state.variation);
  setPly(state.ply);
}

/** Позиция на шаг назад: последний ход варианта либо предыдущий ход партии. */
function previousSnapshot(): BoardSnapshot | null {
  const line = variation();
  if (line && line.sans.length > 0) {
    const rest = {
      ...line,
      fens: line.fens.slice(0, -1),
      sans: line.sans.slice(0, -1),
      ucis: line.ucis.slice(0, -1),
    };
    return { variation: rest.sans.length > 0 ? rest : null, ply: ply() };
  }
  if (ply() > 0) return { variation: null, ply: ply() - 1 };
  return null;
}

function undo(): void {
  const next = previousSnapshot();
  if (!next) return;
  undoHistory.push(snapshot());
  applySnapshot(next);
}

function redo(): void {
  const next = undoHistory.pop();
  if (!next) return;
  applySnapshot(next);
}

async function saveComment(index: number, comment: string): Promise<void> {
  const data = detail();
  if (!data) return;
  try {
    await api.saveUserComment(data.game.id, index, comment);
    setDetail({
      ...data,
      moves: data.moves.map((move) =>
        move.ply === index ? { ...move, userComment: comment } : move,
      ),
    });
  } catch (error) {
    ui.notify(toErrorMessage(error), "error");
  }
}

async function saveNotes(notes: string): Promise<void> {
  const data = detail();
  if (!data) return;
  try {
    await api.updateNotes(data.game.id, notes);
    setDetail({ ...data, game: { ...data.game, notes } });
  } catch (error) {
    ui.notify(toErrorMessage(error), "error");
  }
}

async function runAnalysis(): Promise<void> {
  const data = detail();
  if (!data || analyzing()) return;
  analysisCancelled = false;
  setAnalyzing(true);
  setProgress(null);
  const gameId = data.game.id;
  try {
    const settings = settingsStore.settings();
    const result = await api.analyzeGame(
      gameId,
      settings.analysisDepth,
      settings.multipv,
      (update) => {
        if (detail()?.game.id === gameId) setProgress(update);
      },
    );
    if (detail()?.game.id !== gameId) return;
    setDetail(result);
    setPly(0);
    setVariation(null);
    undoHistory.length = 0;
    ui.notify("Анализ завершён", "success");
    await settingsStore.refreshEngine();
  } catch (error) {
    if (!analysisCancelled) ui.notify(toErrorMessage(error), "error");
  } finally {
    setAnalyzing(false);
    setProgress(null);
  }
}

async function cancelAnalysis(): Promise<void> {
  analysisCancelled = true;
  await api.stopEngine().catch(() => {});
  setAnalyzing(false);
}

export const gameStore = {
  detail,
  loading,
  ply,
  orientation,
  categoryFilter,
  variation,
  preview,
  analyzing,
  progress,
  moves,
  analysis,
  currentMove,
  currentFen,
  boardFen,
  lastMoveUci,
  badge,
  checkSquareName,
  winProbWhite,
  evalLabel,
  arrows,
  movePairs,
  initialFen,
  load,
  goTo,
  next,
  previous,
  first,
  last,
  flip,
  undo,
  redo,
  toggleCategory,
  setCategoryFilter,
  stopPreview,
  playPreview,
  playBestLine,
  enterVariation,
  clearVariation,
  saveComment,
  saveNotes,
  runAnalysis,
  cancelAnalysis,
};
