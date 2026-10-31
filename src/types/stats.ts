import type { BoardTheme, Color, Language, Phase, PieceSet } from "./chess";

export interface EngineStatus {
  available: boolean;
  path: string | null;
  name: string | null;
  error: string | null;
}

export interface AppSettings {
  playerUsername: string;
  activeAccountId: number | null;
  analysisDepth: number;
  multipv: number;
  boardTheme: BoardTheme;
  pieceSet: PieceSet;
  showArrows: boolean;
  autoAnalyze: boolean;
  analysisThreads: number;
  autoResumeAnalysis: boolean;
  language: Language;
}

export interface AppInfo {
  name: string;
  version: string;
  ecoOpenings: number;
  maxAnalysisWorkers: number;
}

export interface OpeningEntry {
  eco: string;
  name: string;
  pgn: string;
}

export interface ColorRecord {
  games: number;
  wins: number;
  draws: number;
  losses: number;
  accuracy: number | null;
}

export interface HubKpis {
  totalGames: number;
  analyzedGames: number;
  wins: number;
  draws: number;
  losses: number;
  winRate: number;
  avgAccuracy: number | null;
  avgAcpl: number | null;
  avgPerfElo: number | null;
  avgPerfEloSpread: number | null;
  white: ColorRecord;
  black: ColorRecord;
}

export interface OpeningStat {
  eco: string;
  name: string;
  games: number;
  analyzed: number;
  wins: number;
  draws: number;
  losses: number;
  winRate: number;
  accuracy: number | null;
  exitEval: number | null;
  exitWinProb: number | null;
  price: number;
  reliable: boolean;
}

export interface PhaseStat {
  phase: Phase;
  accuracy: number | null;
  moves: number;
  games: number;
}

export interface AccuracyPoint {
  gameId: number;
  playedAt: number;
  accuracy: number;
  opponent: string;
  color: Color;
  result: string;
}

export interface TimeStats {
  timeTroubleMoves: number;
  impulsiveErrors: number;
  overthinkMoves: number;
  avgTimeSpent: number | null;
  pressureAccuracy: number | null;
}

export interface AnalysisConfigView {
  engineName: string;
  engineVersion: string;
  depth: number;
  multipv: number;
  classifierVersion: string;
  accuracyMethodVersion: string;
  included: number;
  total: number;
  mismatchedGameIds: number[];
}

export interface HubStats {
  kpis: HubKpis;
  openings: OpeningStat[];
  phases: PhaseStat[];
  trend: AccuracyPoint[];
  time: TimeStats;
  config: AnalysisConfigView | null;
}
