import type { UserColor } from "./chess";

export type GameSource = "chesscom" | "manual";
export type TimeClass = "bullet" | "blitz" | "rapid" | "classical";

export interface Game {
  id: number;
  externalId: string;
  source: GameSource;
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
}

export interface GameFilter {
  search?: string | null;
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
  username: string;
  archivesProcessed: number;
  gamesImported: number;
  gamesSkipped: number;
}

export interface ImportSummary {
  imported: number;
  skipped: number;
  errors: string[];
}

export interface BatchState {
  running: boolean;
  current: number;
  total: number;
  percent: number;
  label: string;
}

export const EMPTY_BATCH: BatchState = {
  running: false,
  current: 0,
  total: 0,
  percent: 0,
  label: "",
};
