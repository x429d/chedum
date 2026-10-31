import { For, Show } from "solid-js";

import { t } from "../../i18n";
import { libraryStore } from "../../stores/libraryStore";
import { ui } from "../../stores/uiStore";
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

/// Плавная шкала: 0 % — красный, 50 % — янтарный, 100 % — зелёный.
function accuracyColor(value: number | null): string | undefined {
  if (value === null) return undefined;
  const clamped = Math.max(0, Math.min(100, value));
  const hue = clamped <= 50 ? (clamped / 50) * 42 : 42 + ((clamped - 50) / 50) * 68;
  return `hsl(${hue.toFixed(1)} 58% 65%)`;
}

function accuracyHint(game: GameListItem): string {
  if (!game.hasAnalysis || !game.engineName) return t("studio.notAnalyzedTitle");
  return `${game.engineName} · depth ${game.analysisDepth ?? "—"}`;
}

function sourceLabel(game: GameListItem): string | null {
  if (game.source === "manual") return null;
  return t(
    game.source === "chesscom" ? "accounts.platformChesscom" : "accounts.platformLichess",
  );
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
        <div class="game-meta">
          {timeClassLabel(props.game.timeClass)}
          <Show when={sourceLabel(props.game)}>
            {(label) => (
              <span
                class={`source-chip source-${props.game.source}`}
                title={props.game.accountUsername ? `@${props.game.accountUsername}` : label()}
              >
                {label()}
              </span>
            )}
          </Show>
          <Show when={props.game.analysisAttempts >= 3}>
            <span
              style={{ color: "var(--danger)", "margin-left": "5px" }}
              title={`Analysis failed ${props.game.analysisAttempts} times: ${props.game.lastAnalysisError ?? ""}`}
              aria-label="Analysis failed 3 times"
            >
              ⚠
            </span>
          </Show>
        </div>
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
          {props.game.openingName ?? t("library.noOpening")}
        </span>
      </td>
      <td style={{ "text-align": "right" }} title={accuracyHint(props.game)}>
        <Show
          when={props.game.userAccuracy !== null}
          fallback={<span class="dim">—</span>}
        >
          <span
            class="accuracy-cell"
            style={{ color: accuracyColor(props.game.userAccuracy) }}
          >
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
            title={t("library.deleteGame")}
            aria-label={t("library.deleteGame")}
            onClick={() => libraryStore.deleteGame(props.game.id)}
          >
            <Icon name="trash" size={14} />
          </button>
          <button
            type="button"
            class="btn btn-ghost btn-sm icon-btn"
            title={props.game.isFavorite ? t("library.removeFromFavorites") : t("library.addToFavorites")}
            aria-label={props.game.isFavorite ? t("library.removeFromFavorites") : t("library.addToFavorites")}
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
            title={t("library.analyzeGame")}
            aria-label={t("library.analyzeGame")}
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
            background: "rgba(198,161,91,0.06)",
            "font-size": "11.5px",
          }}
        >
          <span>{t("library.selectedCount", { count: libraryStore.selectedCount() })}</span>
          <button type="button" class="btn btn-sm" onClick={() => libraryStore.analyzeSelected()}>
            <Icon name="analyze" size={12} />
            {t("library.analyzeSelected")}
          </button>
          <button type="button" class="btn btn-ghost btn-sm" onClick={() => libraryStore.clearSelection()}>
            {t("library.deselect")}
          </button>
          <button
            type="button"
            class="btn btn-danger btn-sm"
            style={{ "margin-left": "auto" }}
            onClick={() => libraryStore.deleteSelected()}
          >
            <Icon name="trash" size={12} />
            {t("library.deleteSelected")}
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
              <th>{t("library.date")}</th>
              <th>{t("library.players")}</th>
              <th>{t("library.result")}</th>
              <th>{t("library.opening")}</th>
              <th style={{ "text-align": "right" }}>{t("library.accuracy")}</th>
              <th class="col-acpl" style={{ "text-align": "right" }}>{t("library.acpl")}</th>
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
                        {t("library.emptyTitle")}
                      </div>
                      <div style={{ "font-size": "11.5px", "max-width": "360px" }}>
                        {t("library.emptyHint")}
                      </div>
                      <div style={{ display: "flex", gap: "8px", "margin-top": "8px" }}>
                        <button
                          type="button"
                          class="btn btn-primary btn-sm"
                          onClick={() => ui.setView("accounts")}
                        >
                          <Icon name="user" size={13} />
                          {t("accounts.addBtn")}
                        </button>
                        <button
                          type="button"
                          class="btn btn-sm"
                          onClick={() => void libraryStore.importFiles()}
                        >
                          <Icon name="import" size={13} />
                          {t("library.importPgn")}
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
