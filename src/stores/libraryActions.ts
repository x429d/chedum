import type { Setter } from "solid-js";

import { api } from "../services/api";
import type { GameFilter, GameListItem } from "../types/models";
import { toErrorMessage } from "../utils/format";
import { ui } from "./uiStore";

export interface LibraryActionsDeps {
  games: () => GameListItem[];
  total: () => number;
  selected: () => number[];
  selectedGameId: () => number | null;
  filter: () => GameFilter;
  setGames: Setter<GameListItem[]>;
  setFavorites: Setter<number>;
  setSelected: Setter<number[]>;
  setSelectedGameId: (id: number | null) => void;
  setFilter: Setter<GameFilter>;
  pageIndex: () => number;
  pageSize: () => number;
  load: () => Promise<void>;
}

/// Действия над библиотекой, которым нужны только сигналы стора.
export function createLibraryActions(deps: LibraryActionsDeps) {
  async function exportSelection(): Promise<void> {
    const ids = deps.selected();
    if (ids.length === 0 && deps.total() === 0) {
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
    const favorite = !game.isFavorite;
    try {
      await api.toggleFavorite(game.id, favorite);
      // Счётчик избранного приходит с бэкендом только при полной загрузке страницы,
      // поэтому обновляем его локально вместе со строкой таблицы.
      deps.setGames((items) =>
        items.map((item) => (item.id === game.id ? { ...item, isFavorite: favorite } : item)),
      );
      deps.setFavorites((count) => Math.max(0, count + (favorite ? 1 : -1)));
      // В режиме «только избранное» снятая звезда убирает партию из выборки:
      // перезагружаем страницу, чтобы не оставлять лишнюю строку и не сбить total.
      if (deps.filter().favoritesOnly) await deps.load();
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
        await deps.load();
        return;
      }
      deps.setGames((items) => items.filter((game) => !unique.includes(game.id)));
      deps.setSelected((items) => items.filter((id) => !unique.includes(id)));
      const current = deps.selectedGameId();
      if (current !== null && unique.includes(current)) {
        deps.setSelectedGameId(null);
        ui.setView("library");
      }
      ui.notify(
        unique.length === 1 ? "Партия удалена" : `Удалено партий: ${deleted}`,
        "success",
      );
      if (deps.games().length === 0 && deps.pageIndex() > 0) {
        deps.setFilter((currentFilter) => ({
          ...currentFilter,
          offset: Math.max(0, (currentFilter.offset ?? 0) - deps.pageSize()),
        }));
      }
      await deps.load();
    } catch (error) {
      ui.notify(toErrorMessage(error), "error");
    }
  }

  return { exportSelection, toggleFavorite, deleteGames };
}
