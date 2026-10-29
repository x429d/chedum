import { Channel, invoke } from "@tauri-apps/api/core";
import { confirm as confirmDialog, open, save } from "@tauri-apps/plugin-dialog";

import type {
  AnalysisProgress,
  AppInfo,
  AppSettings,
  ArchiveView,
  EngineStatus,
  EngineUpdate,
  GameDetail,
  GameFilter,
  HubStats,
  ImportSummary,
  OpeningEntry,
  PagedGames,
  PendingAnalysis,
  SyncProgress,
  SyncResult,
} from "../types/models";

const PGN_FILTER = [{ name: "PGN", extensions: ["pgn"] }];

function channel<T>(handler: (message: T) => void): Channel<T> {
  const pipe = new Channel<T>();
  pipe.onmessage = handler;
  return pipe;
}

export const api = {
  listGames(filter: GameFilter): Promise<PagedGames> {
    return invoke<PagedGames>("list_games", { filter });
  },

  listPendingAnalysis(filter: GameFilter): Promise<PendingAnalysis> {
    return invoke<PendingAnalysis>("list_pending_analysis", { filter });
  },

  getGame(gameId: number): Promise<GameDetail> {
    return invoke<GameDetail>("get_game", { gameId });
  },

  deleteGames(ids: number[]): Promise<number> {
    return invoke<number>("delete_games", { ids });
  },

  confirmDelete(message: string): Promise<boolean> {
    return confirmDialog(message, {
      title: "Chenal",
      kind: "warning",
      okLabel: "Удалить",
      cancelLabel: "Отмена",
    });
  },

  confirmStopBatch(message: string): Promise<boolean> {
    return confirmDialog(message, {
      title: "Chenal",
      kind: "warning",
      okLabel: "Остановить и продолжить",
      cancelLabel: "Отмена",
    });
  },

  toggleFavorite(gameId: number, favorite: boolean): Promise<void> {
    return invoke<void>("toggle_favorite", { gameId, favorite });
  },

  updateNotes(gameId: number, notes: string): Promise<void> {
    return invoke<void>("update_notes", { gameId, notes });
  },

  saveUserComment(gameId: number, ply: number, comment: string): Promise<void> {
    return invoke<void>("save_user_comment", { gameId, ply, comment });
  },

  importPgnText(text: string): Promise<ImportSummary> {
    return invoke<ImportSummary>("import_pgn_text", { text });
  },

  async importPgnFiles(): Promise<ImportSummary | null> {
    const selection = await open({
      title: "Импорт PGN",
      multiple: true,
      filters: PGN_FILTER,
    });
    if (selection === null) return null;
    const paths: string[] = Array.isArray(selection) ? selection : [selection];
    if (paths.length === 0) return null;
    return invoke<ImportSummary>("import_pgn_files", { paths });
  },

  async exportGames(ids: number[]): Promise<string | null> {
    const path = await save({
      title: "Экспорт PGN",
      defaultPath: "chenal-export.pgn",
      filters: PGN_FILTER,
    });
    if (!path) return null;
    await invoke<void>("export_games_to_file", { ids, path });
    return path;
  },

  getSettings(): Promise<AppSettings> {
    return invoke<AppSettings>("get_settings");
  },

  saveSettings(settings: AppSettings): Promise<void> {
    return invoke<void>("save_settings", { settings });
  },

  getEngineStatus(): Promise<EngineStatus> {
    return invoke<EngineStatus>("get_engine_status");
  },

  getAppInfo(): Promise<AppInfo> {
    return invoke<AppInfo>("get_app_info");
  },

  getHubStats(): Promise<HubStats> {
    return invoke<HubStats>("get_hub_stats");
  },

  getSyncStatus(username: string): Promise<ArchiveView[]> {
    return invoke<ArchiveView[]>("get_sync_status", { username });
  },

  searchOpenings(query: string, limit = 20): Promise<OpeningEntry[]> {
    return invoke<OpeningEntry[]>("search_openings", { query, limit });
  },

  analyzeGame(
    gameId: number,
    depth: number,
    multipv: number,
    onProgress: (progress: AnalysisProgress) => void,
  ): Promise<GameDetail> {
    return invoke<GameDetail>("analyze_game", {
      gameId,
      depth,
      multipv,
      onProgress: channel(onProgress),
    });
  },

  liveAnalyze(
    fen: string,
    depth: number,
    multipv: number,
    onUpdate: (update: EngineUpdate) => void,
  ): { promise: Promise<EngineUpdate>; stop: () => Promise<void> } {
    const promise = invoke<EngineUpdate>("analyze_position", {
      fen,
      depth,
      multipv,
      onProgress: channel(onUpdate),
    });
    return { promise, stop: () => invoke<void>("stop_engine") };
  },

  stopEngine(): Promise<void> {
    return invoke<void>("stop_engine");
  },

  syncChesscom(
    username: string,
    force: boolean,
    onProgress: (progress: SyncProgress) => void,
  ): Promise<SyncResult> {
    return invoke<SyncResult>("sync_chesscom", {
      username,
      force,
      onProgress: channel(onProgress),
    });
  },
};
