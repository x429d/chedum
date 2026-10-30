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
import { stopBatchWithConfirm } from "./batchStop";
import { createLibraryActions } from "./libraryActions";
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
const [queuePending, setQueuePending] = createSignal(0);
const [queueExcluded, setQueueExcluded] = createSignal(0);
const selectedCount = () => selected().length;
let loadSequence = 0;
let searchTimer: number | undefined;
let autoResumeHandled = false;

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
    void refreshQueue();
  } catch (error) {
    ui.notify(toErrorMessage(error), "error");
  } finally {
    if (sequence === loadSequence) setLoading(false);
  }
}

/// Очередь — производная состояния: партии без анализа текущей конфигурации,
/// у которых меньше трёх неудачных попыток. Считает бэкенд.
async function refreshQueue(): Promise<void> {
  try {
    const plan = await api.listPendingAnalysis(filter());
    setQueuePending(plan.ids.length);
    setQueueExcluded(plan.excluded);
  } catch (error) {
    // Счётчик не критичен для загрузки библиотеки, но молча терять ошибку нельзя:
    // иначе очередь выглядит пустой, хотя партии без анализа есть.
    ui.notify(`Очередь анализа: ${toErrorMessage(error)}`, "error");
  }
}

async function startAutoResume(): Promise<void> {
  if (autoResumeHandled) return;
  autoResumeHandled = true;
  if (!settingsStore.settings().autoResumeAnalysis) return;
  try {
    const plan = await api.listPendingAnalysis({});
    if (plan.ids.length === 0) return;
    ui.notify(`Продолжаю разбор: ${plan.ids.length} партий`, "info");
    await batchController.analyzeIds(plan.ids, "Продолжение разбора", plan.positions);
  } catch (error) {
    ui.notify(toErrorMessage(error), "error");
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

const actions = createLibraryActions({
  games,
  total,
  selected,
  selectedGameId,
  setGames,
  setSelected,
  setSelectedGameId,
  setFilter,
  pageIndex,
  pageSize,
  load,
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
  void batchController.analyzePending(false, true);
}

function notifyImport(result: ImportSummary): void {
  const details = result.errors.length > 0 ? `, ошибок: ${result.errors.length}` : "";
  ui.notify(`Импортировано ${result.imported}, пропущено ${result.skipped}${details}`, "success");
}

function deleteGame(id: number): void {
  void actions.deleteGames([id]);
}

function deleteSelected(): void {
  void actions.deleteGames(selected());
}

export const libraryStore = {
  games,
  total,
  analyzed,
  favorites,
  pendingAnalysis: queuePending,
  queueExcluded,
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
  exportSelection: actions.exportSelection,
  toggleFavorite: actions.toggleFavorite,
  deleteGames: actions.deleteGames,
  deleteGame,
  deleteSelected,
  analyzeSelected: () => batchController.analyzeSelected(),
  analyzePending: () => batchController.analyzePending(),
  analyzeOne: (id: number) => batchController.analyzeOne(id),
  analyzeIds: (ids: number[], label?: string) =>
    batchController.analyzeIds(ids, label ?? "Пакетный анализ"),
  refreshQueue,
  startAutoResume,
  cancelBatch: () => batchController.cancel(),
  stopBatchWithConfirm: (message: string) =>
    stopBatchWithConfirm(() => batch().running, () => batchController.cancel(), message),
};
