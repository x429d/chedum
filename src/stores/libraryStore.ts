import { createSignal } from "solid-js";
import { api } from "../services/api";
import type {
  BatchState,
  GameFilter,
  GameListItem,
  ImportSummary,
  SyncProgress,
  SyncResult,
} from "../types/models";
import { EMPTY_BATCH } from "../types/models";
import { toErrorMessage } from "../utils/format";
import { gameStore } from "./gameStore";
import { settingsStore } from "./settingsStore";
import { ui } from "./uiStore";

const [games, setGames] = createSignal<GameListItem[]>([]);
const [total, setTotal] = createSignal(0);
const [loading, setLoading] = createSignal(false);
const [filter, setFilter] = createSignal<GameFilter>({ limit: 200 });
const [selected, setSelected] = createSignal<number[]>([]);
const [selectedGameId, setSelectedGameId] = createSignal<number | null>(null);
const [syncProgress, setSyncProgress] = createSignal<SyncProgress | null>(null);
const [syncResult, setSyncResult] = createSignal<SyncResult | null>(null);
const [syncing, setSyncing] = createSignal(false);
const [importing, setImporting] = createSignal(false);
const [batch, setBatch] = createSignal<BatchState>(EMPTY_BATCH);

const selectedCount = () => selected().length;

let loadSequence = 0;
let searchTimer: number | undefined;
let batchCancelled = false;

async function load(): Promise<void> {
  const sequence = ++loadSequence;
  setLoading(true);
  try {
    const page = await api.listGames(filter());
    if (sequence !== loadSequence) return;
    setGames(page.games);
    setTotal(page.total);
    setSelected((ids) => ids.filter((id) => page.games.some((game) => game.id === id)));
  } catch (error) {
    ui.notify(toErrorMessage(error), "error");
  } finally {
    if (sequence === loadSequence) setLoading(false);
  }
}
function updateFilter(patch: Partial<GameFilter>): void {
  setFilter((current) => ({ ...current, ...patch }));
  void load();
}
function setSearch(value: string): void {
  setFilter((current) => ({ ...current, search: value }));
  if (searchTimer !== undefined) window.clearTimeout(searchTimer);
  searchTimer = window.setTimeout(() => {
    searchTimer = undefined;
    void load();
  }, 260);
}

function resetFilter(): void {
  cancelPendingSearch();
  setFilter({ limit: 200 });
  void load();
}

function cancelPendingSearch(): void {
  if (searchTimer !== undefined) {
    window.clearTimeout(searchTimer);
    searchTimer = undefined;
  }
}

function toggleSelect(id: number): void {
  setSelected((ids) => (ids.includes(id) ? ids.filter((value) => value !== id) : [...ids, id]));
}

function selectAll(): void {
  setSelected(games().map((game) => game.id));
}

function clearSelection(): void {
  setSelected([]);
}

function openGame(id: number): void {
  setSelectedGameId(id);
  ui.setView("analysis");
}

async function syncChesscom(username: string, force: boolean): Promise<void> {
  if (syncing()) return;
  setSyncing(true);
  setSyncResult(null);
  try {
    const result = await api.syncChesscom(username, force, (progress) => setSyncProgress(progress));
    setSyncResult(result);
    ui.notify(
      `Синхронизация: +${result.gamesImported} партий, пропущено ${result.gamesSkipped}`,
      "success",
    );
    if (result.username) {
      await settingsStore.update({ playerUsername: result.username });
    }
    await load();
    maybeAutoAnalyze(result.gamesImported);
  } catch (error) {
    ui.notify(toErrorMessage(error), "error");
  } finally {
    setSyncing(false);
  }
}

async function importFiles(): Promise<void> {
  setImporting(true);
  try {
    const result = await api.importPgnFiles();
    if (result) {
      notifyImport(result);
      await load();
      maybeAutoAnalyze(result.imported);
    }
  } catch (error) {
    ui.notify(toErrorMessage(error), "error");
  } finally {
    setImporting(false);
  }
}

async function importText(text: string): Promise<void> {
  setImporting(true);
  try {
    const result = await api.importPgnText(text);
    notifyImport(result);
    await load();
    maybeAutoAnalyze(result.imported);
  } catch (error) {
    ui.notify(toErrorMessage(error), "error");
  } finally {
    setImporting(false);
  }
}

function maybeAutoAnalyze(imported: number): void {
  if (imported <= 0 || !settingsStore.settings().autoAnalyze) return;
  const ids = games()
    .filter((game) => !game.hasAnalysis)
    .map((game) => game.id);
  if (ids.length > 0) void analyzeIds(ids, "Автоанализ");
}

function notifyImport(result: ImportSummary): void {
  const details = result.errors.length > 0 ? `, ошибок: ${result.errors.length}` : "";
  ui.notify(`Импортировано ${result.imported}, пропущено ${result.skipped}${details}`, "success");
}

async function exportSelection(): Promise<void> {
  const ids = selected();
  if (ids.length === 0 && total() === 0) {
    ui.notify("Нет партий для экспорта", "info");
    return;
  }
  try {
    const path = await api.exportGames(ids);
    if (path) ui.notify(`PGN сохранён: ${path}`, "success");
  } catch (error) {
    ui.notify(toErrorMessage(error), "error");
  }
}

async function toggleFavorite(game: GameListItem): Promise<void> {
  try {
    await api.toggleFavorite(game.id, !game.isFavorite);
    setGames((items) =>
      items.map((item) => (item.id === game.id ? { ...item, isFavorite: !item.isFavorite } : item)),
    );
  } catch (error) {
    ui.notify(toErrorMessage(error), "error");
  }
}

async function deleteGame(id: number): Promise<void> {
  if (!window.confirm("Удалить партию вместе с анализом?")) return;
  try {
    await api.deleteGame(id);
    setGames((items) => items.filter((game) => game.id !== id));
    setSelected((ids) => ids.filter((value) => value !== id));
    if (selectedGameId() === id) {
      setSelectedGameId(null);
      ui.setView("library");
    }
    ui.notify("Партия удалена", "success");
  } catch (error) {
    ui.notify(toErrorMessage(error), "error");
  }
}

async function analyzeIds(ids: number[], label: string): Promise<void> {
  if (ids.length === 0 || batch().running) return;
  if (gameStore.analyzing()) {
    ui.notify("Дождитесь завершения анализа партии", "info");
    return;
  }
  batchCancelled = false;
  const settings = settingsStore.settings();
  setBatch({ running: true, current: 0, total: ids.length, percent: 0, label });
  for (let index = 0; index < ids.length; index += 1) {
    if (batchCancelled) break;
    setBatch((current) => ({
      ...current,
      current: index,
      label: `${label}: ${index + 1} из ${ids.length}`,
    }));
    try {
      await api.analyzeGame(ids[index], settings.analysisDepth, settings.multipv, (progress) => {
        setBatch((current) => ({
          ...current,
          percent: ((index + progress.percent / 100) / ids.length) * 100,
          label: progress.message,
        }));
      });
    } catch (error) {
      if (!batchCancelled) ui.notify(toErrorMessage(error), "error");
    }
  }
  const cancelled = batchCancelled;
  setBatch((current) => ({
    ...current,
    running: false,
    percent: cancelled ? current.percent : 100,
    label: cancelled ? "Очередь остановлена" : "Анализ завершён",
  }));
  if (!cancelled) ui.notify(`Готово: ${ids.length} партий`, "success");
  await load();
  await settingsStore.refreshEngine();
}

function analyzeSelected(): void {
  const ids =
    selected().length > 0
      ? selected()
      : games()
          .filter((game) => !game.hasAnalysis)
          .map((game) => game.id);
  if (ids.length === 0) {
    ui.notify("Нет партий, ожидающих анализа", "info");
    return;
  }
  void analyzeIds(ids, "Пакетный анализ");
}

function analyzeOne(id: number): void {
  void analyzeIds([id], "Анализ партии");
}

async function cancelBatch(): Promise<void> {
  batchCancelled = true;
  try {
    await api.stopEngine();
  } catch {
    // движок мог уже завершиться
  }
  setBatch((current) => ({ ...current, label: "Останавливаю очередь…" }));
}

export const libraryStore = {
  games,
  total,
  loading,
  filter,
  selected,
  selectedCount,
  selectedGameId,
  syncProgress,
  syncResult,
  syncing,
  importing,
  batch,
  load,
  updateFilter,
  setSearch,
  resetFilter,
  cancelPendingSearch,
  toggleSelect,
  selectAll,
  clearSelection,
  openGame,
  setSelectedGameId,
  syncChesscom,
  importFiles,
  importText,
  exportSelection,
  toggleFavorite,
  deleteGame,
  analyzeSelected,
  analyzeOne,
  cancelBatch,
};
