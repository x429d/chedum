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
import { createBatchController } from "./libraryBatch";
import { settingsStore } from "./settingsStore";
import { ui } from "./uiStore";
const [games, setGames] = createSignal<GameListItem[]>([]);
const [total, setTotal] = createSignal(0);
const [analyzed, setAnalyzed] = createSignal(0);
const [favorites, setFavorites] = createSignal(0);
const [loading, setLoading] = createSignal(false);
const [filter, setFilter] = createSignal<GameFilter>({ limit: 25, offset: 0 });
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

async function load(): Promise<void> {
  const sequence = ++loadSequence;
  setLoading(true);
  try {
    const page = await api.listGames(filter());
    if (sequence !== loadSequence) return;
    setGames(page.games);
    setTotal(page.total);
    setAnalyzed(page.analyzed);
    setFavorites(page.favorites);
    setSelected((ids) => ids.filter((id) => page.games.some((game) => game.id === id)));
  } catch (error) {
    ui.notify(toErrorMessage(error), "error");
  } finally {
    if (sequence === loadSequence) setLoading(false);
  }
}
function updateFilter(patch: Partial<GameFilter>): void {
  setFilter((current) => ({ ...current, ...patch, offset: 0 }));
  void load();
}
function setSearch(value: string): void {
  setFilter((current) => ({ ...current, search: value, offset: 0 }));
  if (searchTimer !== undefined) window.clearTimeout(searchTimer);
  searchTimer = window.setTimeout(() => {
    searchTimer = undefined;
    void load();
  }, 260);
}

function resetFilter(): void {
  cancelPendingSearch();
  setFilter({ limit: pageSize(), offset: 0 });
  void load();
}

function cancelPendingSearch(): void {
  if (searchTimer !== undefined) {
    window.clearTimeout(searchTimer);
    searchTimer = undefined;
  }
}

const pageSize = () => filter().limit ?? 25;
const pageIndex = () => Math.floor((filter().offset ?? 0) / pageSize());
const pageCount = () => Math.max(1, Math.ceil(total() / pageSize()));

function setPage(page: number): void {
  const target = Math.max(0, Math.min(page, pageCount() - 1));
  setFilter((current) => ({ ...current, offset: target * pageSize() }));
  void load();
}

function setPageSize(size: number): void {
  const limit = Math.max(1, Math.min(100, Math.round(size)));
  setFilter((current) => ({ ...current, limit, offset: 0 }));
  void load();
}

const batchController = createBatchController({
  batch,
  setBatch,
  filter,
  games,
  selected,
  reload: load,
});

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
    setFilter((current) => ({ ...current, offset: 0 }));
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
      setFilter((current) => ({ ...current, offset: 0 }));
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
    setFilter((current) => ({ ...current, offset: 0 }));
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
  void batchController.analyzePending(false);
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

async function deleteGames(ids: number[]): Promise<void> {
  const unique = [...new Set(ids)];
  if (unique.length === 0) return;
  const message =
    unique.length === 1
      ? "Удалить партию вместе с анализом?"
      : `Удалить ${unique.length} партий вместе с анализом?`;
  let confirmed = false;
  try {
    confirmed = await api.confirmDelete(message);
  } catch (error) {
    ui.notify(toErrorMessage(error), "error");
    return;
  }
  if (!confirmed) return;
  try {
    const deleted = await api.deleteGames(unique);
    if (deleted === 0) {
      ui.notify("Партии уже удалены", "info");
      await load();
      return;
    }
    setGames((items) => items.filter((game) => !unique.includes(game.id)));
    setSelected((items) => items.filter((id) => !unique.includes(id)));
    const current = selectedGameId();
    if (current !== null && unique.includes(current)) {
      setSelectedGameId(null);
      ui.setView("library");
    }
    ui.notify(
      unique.length === 1 ? "Партия удалена" : `Удалено партий: ${deleted}`,
      "success",
    );
    if (games().length === 0 && pageIndex() > 0) {
      setFilter((currentFilter) => ({
        ...currentFilter,
        offset: Math.max(0, (currentFilter.offset ?? 0) - pageSize()),
      }));
    }
    await load();
  } catch (error) {
    ui.notify(toErrorMessage(error), "error");
  }
}

function deleteGame(id: number): void {
  void deleteGames([id]);
}

function deleteSelected(): void {
  void deleteGames(selected());
}

export const libraryStore = {
  games,
  total,
  analyzed,
  favorites,
  pendingAnalysis: () => Math.max(0, total() - analyzed()),
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
  pageSize,
  pageIndex,
  pageCount,
  setPage,
  setPageSize,
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
  deleteGames,
  deleteGame,
  deleteSelected,
  analyzeSelected: () => batchController.analyzeSelected(),
  analyzePending: () => batchController.analyzePending(),
  analyzeOne: (id: number) => batchController.analyzeOne(id),
  cancelBatch: () => batchController.cancel(),
};
