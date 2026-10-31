import { For } from "solid-js";

import { pluralGames, t } from "../../i18n";
import type { HubKpis } from "../../types/models";
import { formatNumber } from "../../utils/format";

export function KpiCards(props: { kpis: HubKpis }) {
  const cards = () => [
    {
      label: t("hub.games"),
      value: String(props.kpis.totalGames),
      hint: pluralGames(props.kpis.totalGames),
    },
    {
      label: t("hub.winRate"),
      value: `${formatNumber(props.kpis.winRate, 1)}%`,
      hint: `${props.kpis.wins}W · ${props.kpis.draws}D · ${props.kpis.losses}L`,
    },
    {
      label: t("hub.avgAccuracy"),
      value: props.kpis.avgAccuracy === null ? "—" : `${formatNumber(props.kpis.avgAccuracy, 1)}%`,
      hint: pluralGames(props.kpis.analyzedGames),
    },
    {
      label: t("hub.avgAcpl"),
      value: formatNumber(props.kpis.avgAcpl, 1),
      hint: pluralGames(props.kpis.analyzedGames),
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
