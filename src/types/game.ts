import type { AccountPlatform } from "./account";
import type { UserColor } from "./chess";

export type GameSource = "chesscom" | "lichess" | "manual";
export type TimeClass = "bullet" | "blitz" | "rapid" | "classical";

export interface Game {
  id: number;
  externalId: string;
  source: GameSource;
  accountId: number | null;
  pgn: string;
  initialFen: string | null;
  whiteUsername: string;
  blackUsername: string;
  whiteElo: number | null;
  blackElo: number | null;
  userColor: UserColor;
  result: string;
  termination: string | null;
  timeControl: string | null;
  timeClass: TimeClass | null;
  eco: string | null;
  openingName: string | null;
  playedAt: number;
  totalPlies: number;
  notes: string;
  isFavorite: boolean;
}

export interface GameListItem {
  id: number;
  source: GameSource;
  accountId: number | null;
  accountUsername: string | null;
  accountPlatform: AccountPlatform | null;
  whiteUsername: string;
  blackUsername: string;
  whiteElo: number | null;
  blackElo: number | null;
  userColor: UserColor;
  result: string;
  timeClass: TimeClass | null;
  timeControl: string | null;
  eco: string | null;
  openingName: string | null;
  playedAt: number;
  totalPlies: number;
  isFavorite: boolean;
  notes: string;
  hasAnalysis: boolean;
  userAccuracy: number | null;
  userAcpl: number | null;
  keyMomentsCount: number | null;
  engineName: string | null;
  analysisDepth: number | null;
  accuracyMethod: string | null;
  analysisAttempts: number;
  lastAnalysisError: string | null;
}

export interface GameFilter {
  search?: string | null;
  accountId?: number | null;
  color?: UserColor | null;
  result?: string | null;
  timeClass?: TimeClass | null;
  eco?: string | null;
  dateFrom?: number | null;
  dateTo?: number | null;
  analyzedOnly?: boolean;
  favoritesOnly?: boolean;
  hasNotes?: boolean;
  limit?: number;
  offset?: number;
  sort?: string | null;
}

export interface PagedGames {
  games: GameListItem[];
  total: number;
  analyzed: number;
  favorites: number;
  limit: number;
  offset: number;
}

export interface SyncProgress {
  phase: string;
  message: string;
  archivesDone: number;
  archivesTotal: number;
  gamesImported: number;
  percent: number;
}

export interface SyncResult {
  accountId: number;
  platform: AccountPlatform;
  username: string;
  archivesProcessed: number;
  gamesImported: number;
  duplicates: number;
  filtered: number;
  errors: string[];
}

export interface ImportSummary {
  imported: number;
  skipped: number;
  errors: string[];
}

export interface PendingAnalysis {
  ids: number[];
  positions: number;
  excluded: number;
}

export interface BatchState {
  running: boolean;
  /** Index of the game currently being processed (zero-based). */
  current: number;
  total: number;
  percent: number;
  label: string;
  /** Position within the current game and its total count. */
  ply: number;
  plyTotal: number;
  /** Successfully analyzed and failed games in the current run. */
  done: number;
  failed: number;
  elapsedSec: number;
  /** Estimated remaining time in seconds; null until at least one game is done. */
  etaSec: number | null;
}

export const EMPTY_BATCH: BatchState = {
  running: false,
  current: 0,
  total: 0,
  percent: 0,
  label: "",
  ply: 0,
  plyTotal: 0,
  done: 0,
  failed: 0,
  elapsedSec: 0,
  etaSec: null,
};
