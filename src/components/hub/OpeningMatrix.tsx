import { For, Show } from "solid-js";

import type { OpeningStat } from "../../types/models";
import { formatEvalPawns, formatNumber } from "../../utils/format";

export function OpeningMatrix(props: { openings: OpeningStat[] }) {
  return (
    <div class="card">
      <div class="card-header">
        <div>
          <div class="card-title">Матрица репертуара</div>
          <div class="card-subtitle">Дебюты ECO, точность и оценка выхода</div>
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
            <span>Партии</span>
            <span>Победы</span>
            <span>Точность</span>
          </div>
          <For each={props.openings}>
            {(opening) => (
              <div class="opening-row">
                <span class="eco-code">{opening.eco}</span>
                <span class="opening-name-cell" title={opening.name}>
                  {opening.name || "Без названия"}
                </span>
                <span class="mono">{opening.games}</span>
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
                  {opening.accuracy === null ? "—" : `${formatNumber(opening.accuracy, 0)}%`}
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
