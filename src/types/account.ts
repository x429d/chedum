export type AccountPlatform = "chesscom" | "lichess";

export interface Account {
  id: number;
  platform: AccountPlatform;
  username: string;
  createdAt: string;
  lastSyncedAt: string | null;
  games: number;
  analyzed: number;
}
