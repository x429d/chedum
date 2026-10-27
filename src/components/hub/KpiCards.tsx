import { For } from "solid-js";

import type { HubKpis } from "../../types/models";
import { formatNumber } from "../../utils/format";

export function KpiCards(props: { kpis: HubKpis }) {
  const cards = () => [
    {
      label: "Партий",
      value: String(props.kpis.totalGames),
      hint: `${props.kpis.analyzedGames} с анализом`,
    },
    {
      label: "Win rate",
      value: `${formatNumber(props.kpis.winRate, 1)}%`,
      hint: `${props.kpis.wins}W · ${props.kpis.draws}D · ${props.kpis.losses}L`,
    },
    {
      label: "Средняя точность",
      value: props.kpis.avgAccuracy === null ? "—" : `${formatNumber(props.kpis.avgAccuracy, 1)}%`,
      hint: "метод Lichess",
    },
    {
      label: "Средний ACPL",
      value: formatNumber(props.kpis.avgAcpl, 1),
      hint: "потеря сантипешек",
    },
    {
      label: "Performance Elo",
      value: props.kpis.avgPerfElo === null ? "—" : String(Math.round(props.kpis.avgPerfElo)),
      hint: "оценка силы игры",
    },
  ];
  return (
    <div class="hub-kpis">
      <For each={cards()}>
        {(card) => (
          <div class="kpi-card">
            <div class="kpi-caption">{card.label}</div>
            <div class="kpi-value">{card.value}</div>
            <div class="kpi-hint">{card.hint}</div>
          </div>
        )}
      </For>
    </div>
  );
}
