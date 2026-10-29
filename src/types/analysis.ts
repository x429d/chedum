import type { MoveCategory, Phase, TimeFlag } from "./chess";
import type { Game } from "./game";

export interface PvLine {
  rank: number;
  scoreCp: number | null;
  scoreMate: number | null;
  winProb: number;
  uciMoves: string[];
  sanMoves: string[];
}

export interface MoveEvaluation {
  gameId: number;
  ply: number;
  phase: Phase;
  fenBefore: string;
  fenAfter: string;
  playedUci: string;
  playedSan: string;
  bestUci: string | null;
  bestSan: string | null;
  evalCp: number | null;
  evalMate: number | null;
  bestEvalCp: number | null;
  bestEvalMate: number | null;
  winProbWhite: number;
  winProbLoss: number;
  moveAccuracy: number;
  classification: MoveCategory;
  isCritical: boolean;
  isKeyMoment: boolean;
  materialDiffWhite: number;
  clockRemainingSec: number | null;
  timeSpentSec: number | null;
  timeFlag: TimeFlag;
  explanationTag: string | null;
  explanationText: string | null;
  pvLines: PvLine[];
  userComment: string;
}

export interface CategoryCounts {
  white: Record<MoveCategory, number>;
  black: Record<MoveCategory, number>;
}

export interface GameAnalysis {
  gameId: number;
  engineName: string;
  depth: number;
  multipv: number;
  analyzedAt: string;
  whiteAccuracy: number | null;
  blackAccuracy: number | null;
  whiteAcpl: number | null;
  blackAcpl: number | null;
  whitePerfElo: number | null;
  blackPerfElo: number | null;
  whiteOpeningAcc: number | null;
  whiteMiddlegameAcc: number | null;
  whiteEndgameAcc: number | null;
  blackOpeningAcc: number | null;
  blackMiddlegameAcc: number | null;
  blackEndgameAcc: number | null;
  counts: CategoryCounts;
  summaryText: string;
  keyMomentsCount: number;
}

export interface GameDetail {
  game: Game;
  analysis: GameAnalysis | null;
  moves: MoveEvaluation[];
}

export interface AnalysisProgress {
  stage: string;
  ply: number;
  total: number;
  percent: number;
  message: string;
}

export interface EngineLine {
  rank: number;
  cp: number | null;
  mate: number | null;
  winProb: number;
  uciMoves: string[];
  sanMoves: string[];
}

export interface EngineUpdate {
  fen: string;
  depth: number;
  seldepth: number | null;
  nodes: number | null;
  nps: number | null;
  lines: EngineLine[];
  bestMove: string | null;
  bestSan: string | null;
  finished: boolean;
}
