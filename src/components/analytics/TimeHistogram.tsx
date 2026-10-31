import { For, Show } from "solid-js";

import type { MoveEvaluation } from "../../types/models";
import { HIST_H, HIST_TOP, ML, MR } from "./evalChartLayout";

export function TimeHistogram(props: {
  moves: MoveEvaluation[];
  total: number;
  currentPly: number;
  isPlyInScope: (ply: number) => boolean;
  histMax: number;
  x: (ply: number) => number;
  plotW: number;
  width: number;
}) {
  const barWidth = () => {
    const step = props.plotW / props.total;
    return Math.max(1, Math.min(5, step * 0.45));
  };

  return (
    <>
      <text x={ML - 7} y={HIST_TOP - 4} text-anchor="end" fill="#868d78" font-size="9">
        t
      </text>
      <line
        x1={ML}
        x2={props.width - MR}
        y1={HIST_TOP + HIST_H}
        y2={HIST_TOP + HIST_H}
        stroke="#293123"
        stroke-width="0.7"
      />
      <For each={props.moves}>
        {(move, index) => {
          const height = () => {
            if (
              !props.isPlyInScope(index() + 1) ||
              move.timeSpentSec === null ||
              move.timeSpentSec === undefined ||
              move.timeSpentSec <= 0
            ) {
              return 0;
            }
            return Math.max(1.5, (move.timeSpentSec / props.histMax) * HIST_H);
          };
          const color = () =>
            move.timeFlag === "time_trouble" || move.timeFlag === "impulsive_error"
              ? "#e05c4f"
              : move.timeFlag === "overthink"
                ? "#e08a3c"
                : index() + 1 === props.currentPly
                  ? "#e0bf7d"
                  : "#4a5344";
          return (
            <Show when={height() > 0}>
              <rect
                x={props.x(index() + 1) - barWidth() / 2}
                y={HIST_TOP + HIST_H - height()}
                width={barWidth()}
                height={height()}
                rx={Math.min(barWidth() / 2, 1.5)}
                fill={color()}
                opacity="0.85"
              />
            </Show>
          );
        }}
      </For>
    </>
  );
}
