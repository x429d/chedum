import { Channel, invoke } from "@tauri-apps/api/core";
import { confirm as confirmDialog, open, save } from "@tauri-apps/plugin-dialog";

import { t } from "../i18n";
import type {
  Account,
  AccountPlatform,
  AnalysisProgress,
  AppInfo,
  AppSettings,
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
      okLabel: t("library.delete"),
      cancelLabel: t("library.cancel"),
    });
  },

  confirmStopBatch(message: string): Promise<boolean> {
    return confirmDialog(message, {
      title: "Chenal",
      kind: "warning",
      okLabel: t("library.stopAndProceed"),
      cancelLabel: t("library.cancel"),
    });
  },

  confirmDisconnect(message: string): Promise<boolean> {
    return confirmDialog(message, {
      title: "Chenal",
      kind: "warning",
      okLabel: t("accounts.remove"),
      cancelLabel: t("library.cancel"),
    });
  },

  confirmSyncReset(message: string): Promise<boolean> {
    return confirmDialog(message, {
      title: "Chenal",
      kind: "warning",
      okLabel: t("accounts.syncReset"),
      cancelLabel: t("library.cancel"),
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

  importPgnText(text: string, accountId: number | null): Promise<ImportSummary> {
    return invoke<ImportSummary>("import_pgn_text", { text, accountId });
  },

  async importPgnFiles(accountId: number | null): Promise<ImportSummary | null> {
    const selection = await open({
      title: `${t("library.importPgn")} PGN`,
      multiple: true,
      filters: PGN_FILTER,
    });
    if (selection === null) return null;
    const paths: string[] = Array.isArray(selection) ? selection : [selection];
    if (paths.length === 0) return null;
    return invoke<ImportSummary>("import_pgn_files", { paths, accountId });
  },

  async exportGames(ids: number[]): Promise<string | null> {
    const path = await save({
      title: `${t("library.export")} PGN`,
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

  getHubStats(accountId: number | null): Promise<HubStats> {
    return invoke<HubStats>("get_hub_stats", { accountId });
  },

  listAccounts(): Promise<Account[]> {
    return invoke<Account[]>("list_accounts");
  },

  addAccount(platform: AccountPlatform, username: string): Promise<Account> {
    return invoke<Account>("add_account", { platform, username });
  },

  removeAccount(accountId: number): Promise<void> {
    return invoke<void>("remove_account", { accountId });
  },

  searchOpenings(query: string, limit = 20): Promise<OpeningEntry[]> {
    return invoke<OpeningEntry[]>("search_openings", { query, limit });
  },

  analyzeGame(
    gameId: number,
    depth: number,
    multipv: number,
    parallel: boolean,
    onProgress: (progress: AnalysisProgress) => void,
  ): Promise<GameDetail> {
    return invoke<GameDetail>("analyze_game", {
      gameId,
      depth,
      multipv,
      parallel,
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

  syncAccount(
    accountId: number,
    force: boolean,
    onProgress: (progress: SyncProgress) => void,
  ): Promise<SyncResult> {
    return invoke<SyncResult>("sync_account", {
      accountId,
      force,
      onProgress: channel(onProgress),
    });
  },

  cancelSync(): Promise<void> {
    return invoke<void>("cancel_sync");
  },

  resetSyncHistory(accountId: number): Promise<void> {
    return invoke<void>("reset_sync_history", { accountId });
  },

  getDiagnostics(): Promise<string> {
    return invoke<string>("get_diagnostics");
  },
};
