import { For, Show } from "solid-js";

import { libraryStore } from "../../stores/libraryStore";
import type { GameListItem } from "../../types/models";
import {
  formatDate,
  formatNumber,
  outcomeClass,
  outcomeLabel,
  timeClassLabel,
} from "../../utils/format";
import { Icon } from "../ui/Icon";
import { Pagination } from "./Pagination";

function accuracyClass(value: number | null): string {
  if (value === null) return "";
  if (value >= 90) return "accuracy-high";
  if (value >= 75) return "accuracy-mid";
  return "accuracy-low";
}

function Row(props: { game: GameListItem }) {
  const selected = () => libraryStore.selected().includes(props.game.id);
  return (
    <tr onClick={() => libraryStore.openGame(props.game.id)}>
      <td style={{ width: "28px" }} onClick={(event) => event.stopPropagation()}>
        <input
          type="checkbox"
          checked={selected()}
          onChange={() => libraryStore.toggleSelect(props.game.id)}
        />
      </td>
      <td style={{ "white-space": "nowrap" }}>
        <div class="game-title">{formatDate(props.game.playedAt)}</div>
        <div class="game-meta">{timeClassLabel(props.game.timeClass)}</div>
      </td>
      <td>
        <div class="game-title">
          {props.game.whiteUsername}
          <span class="dim" style={{ "font-weight": "400" }}>
            {" "}
            {props.game.whiteElo ?? ""}
          </span>
        </div>
        <div class="game-meta">
          {props.game.blackUsername}
          <span class="dim"> {props.game.blackElo ?? ""}</span>
        </div>
      </td>
      <td>
        <span class={`badge ${outcomeClass(props.game.result, props.game.userColor)}`}>
          {outcomeLabel(props.game.result, props.game.userColor)}
        </span>
      </td>
      <td class="opening-cell">
        <span class="eco-code">{props.game.eco ?? "—"}</span>
        <span class="opening-name" title={props.game.openingName ?? ""}>
          {props.game.openingName ?? "Без дебюта"}
        </span>
      </td>
      <td style={{ "text-align": "right" }}>
        <Show
          when={props.game.userAccuracy !== null}
          fallback={<span class="dim">—</span>}
        >
          <span class={`accuracy-cell ${accuracyClass(props.game.userAccuracy)}`}>
            {formatNumber(props.game.userAccuracy, 1)}%
          </span>
        </Show>
      </td>
      <td class="col-acpl" style={{ "text-align": "right" }}>
        <Show when={props.game.userAcpl !== null} fallback={<span class="dim">—</span>}>
          <span class="mono">{formatNumber(props.game.userAcpl, 1)}</span>
        </Show>
      </td>
      <td style={{ width: "104px" }} onClick={(event) => event.stopPropagation()}>
        <div class="row-actions">
          <button
            type="button"
            class="btn btn-ghost btn-sm icon-btn row-delete"
            title="Удалить"
            aria-label="Удалить партию"
            onClick={() => libraryStore.deleteGame(props.game.id)}
          >
            <Icon name="trash" size={14} />
          </button>
          <button
            type="button"
            class="btn btn-ghost btn-sm icon-btn"
            title={props.game.isFavorite ? "Убрать из избранного" : "В избранное"}
            aria-label={props.game.isFavorite ? "Убрать из избранного" : "В избранное"}
            onClick={() => void libraryStore.toggleFavorite(props.game)}
          >
            <Icon
              name="star"
              size={15}
              class={`star-icon${props.game.isFavorite ? " favorite-on" : ""}`}
            />
          </button>
          <button
            type="button"
            class="btn btn-ghost btn-sm icon-btn"
            title="Анализировать"
            aria-label="Анализировать партию"
            onClick={() => libraryStore.analyzeOne(props.game.id)}
          >
            <Icon name="analyze" size={15} />
          </button>
        </div>
      </td>
    </tr>
  );
}

export function GameTable() {
  const allSelected = () =>
    libraryStore.games().length > 0 &&
    libraryStore.games().every((game) => libraryStore.selected().includes(game.id));

  return (
    <div class="card card-table" style={{ overflow: "hidden" }}>
      <Show when={libraryStore.selectedCount() > 0}>
        <div
          style={{
            display: "flex",
            "align-items": "center",
            gap: "8px",
            padding: "9px 14px",
            "border-bottom": "1px solid var(--border)",
            background: "rgba(20,184,166,0.06)",
            "font-size": "11.5px",
          }}
        >
          <span>Выбрано: {libraryStore.selectedCount()}</span>
          <button type="button" class="btn btn-sm" onClick={() => libraryStore.analyzeSelected()}>
            <Icon name="sparkles" size={12} />
            Анализировать выбранные
          </button>
          <button type="button" class="btn btn-ghost btn-sm" onClick={() => libraryStore.clearSelection()}>
            Снять выбор
          </button>
          <button
            type="button"
            class="btn btn-danger btn-sm"
            style={{ "margin-left": "auto" }}
            onClick={() => libraryStore.deleteSelected()}
          >
            <Icon name="trash" size={12} />
            Удалить выбранные
          </button>
        </div>
      </Show>
      <div class="table-wrap">
        <table class="data-table">
          <thead>
            <tr>
              <th>
                <input
                  type="checkbox"
                  checked={allSelected()}
                  onChange={() =>
                    allSelected() ? libraryStore.clearSelection() : libraryStore.selectAll()
                  }
                />
              </th>
              <th>Дата</th>
              <th>Игроки</th>
              <th>Результат</th>
              <th>Дебют</th>
              <th style={{ "text-align": "right" }}>Точность</th>
              <th class="col-acpl" style={{ "text-align": "right" }}>ACPL</th>
              <th />
            </tr>
          </thead>
          <tbody>
            <Show
              when={libraryStore.games().length > 0}
              fallback={
                <tr>
                  <td colspan="8">
                    <div class="empty-state">
                      <div class="empty-icon">
                        <Icon name="library" size={22} />
                      </div>
                      <div style={{ "font-weight": "600", color: "var(--text)" }}>
                        {libraryStore.loading() ? "Загружаю библиотеку…" : "Пока нет партий"}
                      </div>
                      <div style={{ "font-size": "11.5px", "max-width": "360px" }}>
                        Импортируйте PGN или синхронизируйте публичные партии Chess.com — анализ
                        Stockfish запускается локально.
                      </div>
                      <div style={{ display: "flex", gap: "8px", "margin-top": "8px" }}>
                        <button
                          type="button"
                          class="btn btn-primary btn-sm"
                          onClick={() => void libraryStore.importFiles()}
                        >
                          <Icon name="import" size={13} />
                          Импорт PGN
                        </button>
                      </div>
                    </div>
                  </td>
                </tr>
              }
            >
              <For each={libraryStore.games()}>{(game) => <Row game={game} />}</For>
            </Show>
          </tbody>
        </table>
      </div>
      <Pagination />
    </div>
  );
}
