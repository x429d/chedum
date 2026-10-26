import { createMemo, For, Show } from "solid-js";

import type { AccuracyPoint } from "../../types/models";
import { formatDate } from "../../utils/format";

const W = 1000;
const H = 190;
const ML = 34;
const MR = 16;
const MT = 14;
const MB = 24;
const PW = W - ML - MR;
const PH = H - MT - MB;

export function AccuracyTrend(props: { points: AccuracyPoint[] }) {
  const points = createMemo(() =>
    props.points.map((point, index) => ({
      point,
      x: ML + (props.points.length <= 1 ? PW / 2 : (index / (props.points.length - 1)) * PW),
      y: MT + (1 - Math.max(0, Math.min(100, point.accuracy)) / 100) * PH,
    })),
  );
  const path = createMemo(() =>
    points()
      .map((item, index) => `${index === 0 ? "M" : "L"} ${item.x.toFixed(1)} ${item.y.toFixed(1)}`)
      .join(" "),
  );
  return (
    <div class="card">
      <div class="card-header">
        <div>
          <div class="card-title">Тренд точности</div>
          <div class="card-subtitle">От партии к партии</div>
        </div>
        <span class="chip">{props.points.length} точек</span>
      </div>
      <div class="card-pad">
        <Show
          when={props.points.length > 0}
          fallback={<div class="empty-state" style={{ padding: "20px" }}>Пока нет проанализированных партий.</div>}
        >
          <svg class="trend-svg" viewBox={`0 0 ${W} ${H}`}>
            <For each={[50, 75, 90]}>
              {(value) => (
                <>
                  <line
                    x1={ML}
                    x2={W - MR}
                    y1={MT + (1 - value / 100) * PH}
                    y2={MT + (1 - value / 100) * PH}
                    stroke="#232a3b"
                    stroke-width="0.7"
                  />
                  <text
                    x={ML - 6}
                    y={MT + (1 - value / 100) * PH + 3}
                    text-anchor="end"
                    fill="#6b7280"
                    font-size="9"
                  >
                    {value}
                  </text>
                </>
              )}
            </For>
            <path d={path()} fill="none" stroke="#2dd4bf" stroke-width="1.8" />
            <For each={points()}>
              {(item) => (
                <circle
                  cx={item.x}
                  cy={item.y}
                  r="4"
                  fill={
                    item.point.result === "win"
                      ? "#4ade80"
                      : item.point.result === "loss"
                        ? "#f87171"
                        : "#facc15"
                  }
                  stroke="#0b0e14"
                  stroke-width="1.4"
                >
                  <title>
                    {formatDate(item.point.playedAt)} · {item.point.opponent} ·{" "}
                    {item.point.accuracy.toFixed(1)}%
                  </title>
                </circle>
              )}
            </For>
          </svg>
          <div class="chart-legend" style={{ "margin-top": "4px" }}>
            <span>
              <span class="legend-dot" style={{ background: "#4ade80" }} />
              победа
            </span>
            <span>
              <span class="legend-dot" style={{ background: "#facc15" }} />
              ничья
            </span>
            <span>
              <span class="legend-dot" style={{ background: "#f87171" }} />
              поражение
            </span>
          </div>
        </Show>
      </div>
    </div>
  );
}
