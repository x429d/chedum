import { createMemo, createSignal, For, Show } from "solid-js";

import { t } from "../../i18n";
import type { OpeningStat } from "../../types/models";
import { formatEvalPawns, formatNumber } from "../../utils/format";

type SortMode = "games" | "price";

const VISIBLE_ROWS = 12;

export function OpeningMatrix(props: { openings: OpeningStat[] }) {
  const [sort, setSort] = createSignal<SortMode>("games");
  const ordered = createMemo(() => {
    const list = [...props.openings];
    list.sort((left, right) =>
      sort() === "price" ? right.price - left.price : right.games - left.games,
    );
    return list.slice(0, VISIBLE_ROWS);
  });
  return (
    <div class="card">
      <div class="card-header">
        <div>
          <div class="card-title">{t("hub.openingMatrix")}</div>
        </div>
        <div class="segmented" style={{ "font-size": "10px" }}>
          <button
            type="button"
            aria-pressed={sort() === "games"}
            classList={{ active: sort() === "games" }}
            onClick={() => setSort("games")}
          >
            {t("hub.sortByGames")}
          </button>
          <button
            type="button"
            aria-pressed={sort() === "price"}
            classList={{ active: sort() === "price" }}
            onClick={() => setSort("price")}
          >
            {t("hub.sortByPrice")}
          </button>
        </div>
      </div>
      <div class="card-pad">
        <Show
          when={props.openings.length > 0}
          fallback={<div class="empty-state" style={{ padding: "20px" }}>{t("hub.noOpeningData")}</div>}
        >
          <div class="opening-row" style={{ "font-size": "10px", color: "var(--dim)" }}>
            <span>{t("hub.eco")}</span>
            <span>{t("hub.openingCol")}</span>
            <span>{t("hub.gamesCol")}</span>
            <span>{t("hub.analyzedCol")}</span>
            <span>{t("hub.winRate")}</span>
            <span>{t("hub.accuracyCol")}</span>
          </div>
          <For each={ordered()}>
            {(opening) => (
              <div
                class="opening-row"
                classList={{ unreliable: !opening.reliable }}
              >
                <span class="eco-code">{opening.eco}</span>
                <span class="opening-name-cell" title={opening.name}>
                  {opening.name || t("library.noOpening")}
                </span>
                <span class="mono">{opening.games}</span>
                <span class="mono">{opening.analyzed}</span>
                <span>
                  <div class="mini-bar">
                    <div
                      class="mini-bar-fill"
                      style={{ width: `${opening.winRate}%`, background: "#c6a15b" }}
                    />
                  </div>
                  <span class="dim" style={{ "font-size": "9px" }}>
                    {formatNumber(opening.winRate, 0)}%
                  </span>
                </span>
                <span class="mono" style={{ "text-align": "right" }}>
                  <Show
                    when={opening.reliable}
                    fallback={
                      <span class="dim" style={{ "font-size": "10px" }}>
                        {t("hub.notEnoughData")}
                      </span>
                    }
                  >
                    {opening.accuracy === null
                      ? "—"
                      : `${formatNumber(opening.accuracy, 0)}%`}
                  </Show>
                  <span class="dim" style={{ display: "block", "font-size": "9px" }}>
                    {formatEvalPawns(opening.exitEval)}
                  </span>
                </span>
              </div>
            )}
          </For>
        </Show>
      </div>
    </div>
  );
}
