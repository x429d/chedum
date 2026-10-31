import { createMemo, For } from "solid-js";

import { currentLanguage, t } from "../../i18n";
import type { MoveEvaluation, UserColor } from "../../types/models";
import { formatNumber } from "../../utils/format";

export function TimeSummary(props: { moves: MoveEvaluation[]; userColor: UserColor }) {
  const userMoves = createMemo(() => {
    const color = props.userColor;
    if (color === "none") return props.moves;
    return props.moves.filter((move) =>
      color === "white" ? move.ply % 2 === 1 : move.ply % 2 === 0,
    );
  });
  const stats = createMemo(() => {
    const list = userMoves();
    const trouble = list.filter((move) => move.timeFlag === "time_trouble").length;
    const impulsive = list.filter((move) => move.timeFlag === "impulsive_error").length;
    const overthink = list.filter((move) => move.timeFlag === "overthink").length;
    const times = list
      .map((move) => move.timeSpentSec)
      .filter((value): value is number => value !== null && Number.isFinite(value));
    const average = times.length > 0 ? times.reduce((sum, value) => sum + value, 0) / times.length : null;
    const pressure = list.filter(
      (move) => move.timeFlag === "time_trouble" || move.timeFlag === "impulsive_error",
    );
    const pressureAccuracy =
      pressure.length > 0
        ? pressure.reduce((sum, move) => sum + move.moveAccuracy, 0) / pressure.length
        : null;
    return { trouble, impulsive, overthink, average, pressureAccuracy };
  });
  const secUnit = () => (currentLanguage() === "ru" ? "с" : "s");
  const tiles = createMemo(() => [
    { label: t("hub.timeTroubleTile"), value: String(stats().trouble) },
    { label: t("hub.impulsiveTile"), value: String(stats().impulsive) },
    { label: t("hub.overthinkTile"), value: String(stats().overthink) },
    {
      label: t("hub.avgTimeTile"),
      value: stats().average === null ? "—" : `${formatNumber(stats().average, 1)} ${secUnit()}`,
    },
    {
      label: t("hub.pressureAccuracyTile"),
      value: stats().pressureAccuracy === null ? "—" : `${formatNumber(stats().pressureAccuracy, 1)}%`,
    },
  ]);
  return (
    <div class="metric-tiles">
      <For each={tiles()}>
        {(tile) => (
          <div class="metric-tile">
            <div class="metric-label">{tile.label}</div>
            <div class="metric-value">{tile.value}</div>
          </div>
        )}
      </For>
    </div>
  );
}
