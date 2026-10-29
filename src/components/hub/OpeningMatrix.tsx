import { createMemo, createSignal, For, Show } from "solid-js";

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
          <div class="card-title">Матрица репертуара</div>
          <div class="card-subtitle">Дебюты ECO, точность и оценка выхода</div>
        </div>
        <div class="segmented" style={{ "font-size": "10px" }}>
          <button
            type="button"
            classList={{ active: sort() === "games" }}
            title="Сортировать по числу партий"
            onClick={() => setSort("games")}
          >
            Партии
          </button>
          <button
            type="button"
            classList={{ active: sort() === "price" }}
            title="Число партий × средняя потеря Win%"
            onClick={() => setSort("price")}
          >
            По цене
          </button>
        </div>
      </div>
      <div class="card-pad">
        <Show
          when={props.openings.length > 0}
          fallback={<div class="empty-state" style={{ padding: "20px" }}>Нет данных по дебютам.</div>}
        >
          <div class="opening-row" style={{ "font-size": "10px", color: "var(--dim)" }}>
            <span>ECO</span>
            <span>Дебют</span>
            <span title="Все партии этого дебюта">Партии</span>
            <span title="Из них разобрано движком">Разобрано</span>
            <span>Победы</span>
            <span>Точность</span>
          </div>
          <For each={ordered()}>
            {(opening) => (
              <div
                class="opening-row"
                classList={{ unreliable: !opening.reliable }}
                title={`Цена: ${formatNumber(opening.price, 1)} Win% × партии`}
              >
                <span class="eco-code">{opening.eco}</span>
                <span class="opening-name-cell" title={opening.name}>
                  {opening.name || "Без названия"}
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
                      <span
                        class="dim"
                        style={{ "font-size": "10px" }}
                        title={
                          opening.accuracy === null
                            ? `Разобрано ${opening.analyzed} из ${opening.games} — данных нет`
                            : `${formatNumber(opening.accuracy, 1)}% по ${opening.analyzed} партиям — мало данных`
                        }
                      >
                        мало данных
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
