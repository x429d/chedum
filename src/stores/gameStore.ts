import { createSignal } from "solid-js";

import { t } from "../i18n";
import { api } from "../services/api";
import type {
  AnalysisProgress,
  ArrowSpec,
  Color,
  GameDetail,
  MoveCategory,
  SquareHighlight,
} from "../types/models";
import { checkSquare, playMove } from "../utils/chess";
import { toErrorMessage } from "../utils/format";
import { bumpDataVersion } from "./dataVersion";
import { createGameHistory, type VariationState } from "./gameHistory";
import { createPreviewController, type PreviewState } from "./gamePreview";
import { persistComment, persistNotes } from "./gamePersistence";
import {
  arrowsFor,
  badgeFor,
  boardFenFor,
  currentFenFor,
  evalLabelFor,
  initialFenFor,
  lastMoveUciFor,
  movePairsFor,
} from "./gameSelectors";
import { settingsStore } from "./settingsStore";
import { ui } from "./uiStore";

const [detail, setDetail] = createSignal<GameDetail | null>(null);
const [loading, setLoading] = createSignal(false);
const [ply, setPly] = createSignal(0);
const [orientation, setOrientation] = createSignal<Color>("white");
const [categoryFilter, setCategoryFilter] = createSignal<MoveCategory | null>(null);
const [variation, setVariation] = createSignal<VariationState | null>(null);
const [preview, setPreview] = createSignal<PreviewState | null>(null);
const [userArrows, setUserArrows] = createSignal<ArrowSpec[]>([]);
const [userHighlights, setUserHighlights] = createSignal<SquareHighlight[]>([]);
const [analyzing, setAnalyzing] = createSignal(false);
const [progress, setProgress] = createSignal<AnalysisProgress | null>(null);
let analysisCancelled = false;

const previewController = createPreviewController(setPreview);
const stopPreview = previewController.stop;
const playPreview = previewController.play;
const history = createGameHistory({
  variation,
  ply,
  arrows: userArrows,
  highlights: userHighlights,
  setVariation,
  setPly,
  setArrows: setUserArrows,
  setHighlights: setUserHighlights,
  stopPreview,
});

const moves = () => detail()?.moves ?? [];
const analysis = () => detail()?.analysis ?? null;
const currentMove = () => {
  const index = ply();
  return index > 0 ? moves()[index - 1] ?? null : null;
};
const initialFen = () => initialFenFor(detail());
const currentFen = () => currentFenFor(initialFen(), currentMove(), variation());
const boardFen = () => boardFenFor(currentFen(), preview());
const lastMoveUci = () => lastMoveUciFor(currentMove(), variation(), preview());
const badge = () => badgeFor(currentMove(), variation(), preview());
const checkSquareName = () => checkSquare(boardFen());
const winProbWhite = () => currentMove()?.winProbWhite ?? 50;
const evalLabel = () => evalLabelFor(currentMove());
const arrows = (): ArrowSpec[] =>
  arrowsFor(currentMove(), settingsStore.settings().showArrows, variation(), preview());
const movePairs = () => movePairsFor(moves());

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
    setUserArrows([]);
    setUserHighlights([]);
    history.reset();
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
  const target = Math.max(0, Math.min(index, moves().length));
  if (target === ply() && !variation()) return;
  history.record();
  stopPreview();
  setVariation(null);
  setPly(target);
  setUserArrows([]);
  setUserHighlights([]);
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
  history.record();
  const existing = variation();
  const fens = existing ? [...existing.fens, played.fenAfter] : [baseFen, played.fenAfter];
  const sans = existing ? [...existing.sans, played.san] : [played.san];
  const ucis = existing ? [...existing.ucis, played.uci] : [played.uci];
  setVariation({ fens, sans, ucis, basePly: ply() });
  setUserArrows([]);
  setUserHighlights([]);
  return true;
}

function clearVariation(): void {
  if (!variation()) return;
  history.record();
  setVariation(null);
  setUserArrows([]);
  setUserHighlights([]);
}

/** User arrows on the board are a regular action and are undone too. */
function updateUserArrows(next: ArrowSpec[]): void {
  const current = userArrows();
  if (
    current.length === next.length &&
    current.every((arrow, index) => {
      const other = next[index];
      return (
        other !== undefined &&
        arrow.from === other.from &&
        arrow.to === other.to &&
        arrow.color === other.color
      );
    })
  ) {
    return;
  }
  history.record();
  setUserArrows(next);
}

/** User square highlights are a regular action and are undone too. */
function updateUserHighlights(next: SquareHighlight[]): void {
  const current = userHighlights();
  if (
    current.length === next.length &&
    current.every((mark, index) => {
      const other = next[index];
      return other !== undefined && mark.square === other.square && mark.color === other.color;
    })
  ) {
    return;
  }
  history.record();
  setUserHighlights(next);
}

/** A left click on the board removes every user arrow and square mark in one undo step. */
function clearAnnotations(): void {
  if (userArrows().length === 0 && userHighlights().length === 0) return;
  history.record();
  setUserArrows([]);
  setUserHighlights([]);
}

function undo(): void {
  history.undo();
}

function redo(): void {
  history.redo();
}

async function saveComment(index: number, comment: string): Promise<void> {
  const data = detail();
  if (!data) return;
  try {
    setDetail(await persistComment(data, index, comment));
  } catch (error) {
    ui.notify(toErrorMessage(error), "error");
  }
}

async function saveNotes(notes: string): Promise<void> {
  const data = detail();
  if (!data) return;
  try {
    setDetail(await persistNotes(data, notes));
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
      // Single-game screen: the rest of the engine pool is free, so positions
      // are analyzed in parallel.
      true,
      (update) => {
        if (detail()?.game.id === gameId) setProgress(update);
      },
    );
    if (detail()?.game.id !== gameId) return;
    setDetail(result);
    setPly(0);
    setVariation(null);
    setUserArrows([]);
    setUserHighlights([]);
    history.reset();
    bumpDataVersion();
    ui.notify(t("library.analysisFinished"), "success");
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
  userArrows,
  userHighlights,
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
  updateUserArrows,
  updateUserHighlights,
  clearAnnotations,
  saveComment,
  saveNotes,
  runAnalysis,
  cancelAnalysis,
};
