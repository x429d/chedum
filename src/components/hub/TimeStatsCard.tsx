import { For } from "solid-js";

import { currentLanguage, t } from "../../i18n";
import type { TimeStats } from "../../types/models";
import { formatNumber } from "../../utils/format";

export function TimeStatsCard(props: { time: TimeStats }) {
  const secUnit = () => (currentLanguage() === "ru" ? "с" : "s");
  const tiles = () => [
    { label: t("hub.timeTroubleTile"), value: String(props.time.timeTroubleMoves) },
    { label: t("hub.impulsiveTile"), value: String(props.time.impulsiveErrors) },
    { label: t("hub.overthinkTile"), value: String(props.time.overthinkMoves) },
    {
      label: t("hub.avgTimeTile"),
      value:
        props.time.avgTimeSpent === null ? "—" : `${formatNumber(props.time.avgTimeSpent, 1)} ${secUnit()}`,
    },
    {
      label: t("hub.pressureAccuracyTile"),
      value:
        props.time.pressureAccuracy === null
          ? "—"
          : `${formatNumber(props.time.pressureAccuracy, 1)}%`,
    },
  ];
  return (
    <div class="card">
      <div class="card-header">
        <div>
          <div class="card-title">{t("hub.timeManagement")}</div>
        </div>
      </div>
      <div class="card-pad">
        <div class="time-grid">
          <For each={tiles()}>
            {(tile) => (
              <div class="time-tile">
                <div class="metric-label">{tile.label}</div>
                <div class="metric-value">{tile.value}</div>
              </div>
            )}
          </For>
        </div>
      </div>
    </div>
  );
}
