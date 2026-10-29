import { For } from "solid-js";

import type { HubKpis } from "../../types/models";
import { formatNumber, pluralRu } from "../../utils/format";

export function KpiCards(props: { kpis: HubKpis }) {
  const cards = () => [
    {
      label: "Партий",
      value: String(props.kpis.totalGames),
      hint: `${props.kpis.analyzedGames} с анализом`,
    },
    {
      label: "Доля побед",
      value: `${formatNumber(props.kpis.winRate, 1)}%`,
      hint: `${props.kpis.wins} ${pluralRu(props.kpis.wins, "победа", "победы", "побед")}, ${props.kpis.draws} ${pluralRu(props.kpis.draws, "ничья", "ничьи", "ничьих")}, ${props.kpis.losses} ${pluralRu(props.kpis.losses, "поражение", "поражения", "поражений")}`,
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
      value:
        props.kpis.avgPerfElo === null
          ? "—"
          : `${Math.round(props.kpis.avgPerfElo)} ± ${Math.round(props.kpis.avgPerfEloSpread ?? 0)}`,
      hint:
        props.kpis.avgPerfElo === null
          ? "нужно ≥ 20 разобранных партий"
          : "оценка по точности, экспериментально",
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
