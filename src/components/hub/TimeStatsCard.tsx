import { For } from "solid-js";

import type { TimeStats } from "../../types/models";
import { formatNumber } from "../../utils/format";

export function TimeStatsCard(props: { time: TimeStats }) {
  const tiles = () => [
    { label: "Цейтнот", value: String(props.time.timeTroubleMoves), hint: "ходов" },
    { label: "Поспешные", value: String(props.time.impulsiveErrors), hint: "зевков" },
    { label: "Зависания", value: String(props.time.overthinkMoves), hint: "ходов" },
    {
      label: "Среднее время",
      value:
        props.time.avgTimeSpent === null ? "—" : `${formatNumber(props.time.avgTimeSpent, 1)} с`,
      hint: "на ход",
    },
    {
      label: "Точность в цейтноте",
      value:
        props.time.pressureAccuracy === null
          ? "—"
          : `${formatNumber(props.time.pressureAccuracy, 1)}%`,
      hint: "под давлением",
    },
  ];
  return (
    <div class="card">
      <div class="card-header">
        <div>
          <div class="card-title">Тайм-менеджмент</div>
          <div class="card-subtitle">Психология ошибок и часы</div>
        </div>
      </div>
      <div class="card-pad">
        <div class="time-grid">
          <For each={tiles()}>
            {(tile) => (
              <div class="time-tile">
                <div class="metric-label">{tile.label}</div>
                <div class="metric-value">{tile.value}</div>
                <div class="stat-hint">{tile.hint}</div>
              </div>
            )}
          </For>
        </div>
      </div>
    </div>
  );
}
