import { t } from "../i18n";
import type { AnalysisProgress, SyncProgress } from "../types/models";

/** Localized label for a sync progress event. The backend only sends a
 * stable phase plus counters; it never sends user-visible prose. */
export function syncProgressLabel(progress: SyncProgress): string {
  switch (progress.phase) {
    case "archives":
      return t("sync.progressArchives");
    case "archive":
      return t("sync.progressArchive", {
        done: progress.archivesDone,
        total: progress.archivesTotal,
        imported: progress.gamesImported,
      });
    case "download":
      return t("sync.progressDownload");
    case "parse":
      return t("sync.progressParse");
    case "import":
      return t("sync.progressImport");
    case "done":
      return progress.gamesImported > 0
        ? t("sync.progressDone", { imported: progress.gamesImported })
        : t("sync.progressEmpty");
    default:
      return progress.message;
  }
}

export function analysisProgressLabel(progress: AnalysisProgress): string {
  switch (progress.stage) {
    case "prepare":
      return t("progress.prepare");
    case "engine":
      return t("progress.engine", { done: progress.ply, total: progress.total });
    case "aggregate":
      return t("progress.aggregate");
    case "save":
      return t("progress.save");
    case "done":
      return t("progress.done");
    default:
      return progress.message;
  }
}
