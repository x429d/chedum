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
                    stroke="#293123"
                    stroke-width="0.7"
                  />
                  <text
                    x={ML - 6}
                    y={MT + (1 - value / 100) * PH + 3}
                    text-anchor="end"
                    fill="#868d78"
                    font-size="9"
                  >
                    {value}
                  </text>
                </>
              )}
            </For>
            <path d={path()} fill="none" stroke="#e0bf7d" stroke-width="1.8" />
            <For each={points()}>
              {(item) => (
                <circle
                  cx={item.x}
                  cy={item.y}
                  r="4"
                  fill={
                    item.point.result === "win"
                      ? "#9ccf8f"
                      : item.point.result === "loss"
                        ? "#e79a90"
                        : "#d9c47f"
                  }
                  stroke="#0c110b"
                  stroke-width="1.4"
                >
                  <title>
                    {formatDate(item.point.playedAt)}, {item.point.opponent},{" "}
                    {item.point.accuracy.toFixed(1)}%
                  </title>
                </circle>
              )}
            </For>
          </svg>
          <div class="chart-legend" style={{ "margin-top": "4px" }}>
            <span>
              <span class="legend-dot" style={{ background: "#9ccf8f" }} />
              победа
            </span>
            <span>
              <span class="legend-dot" style={{ background: "#d9c47f" }} />
              ничья
            </span>
            <span>
              <span class="legend-dot" style={{ background: "#e79a90" }} />
              поражение
            </span>
          </div>
        </Show>
      </div>
    </div>
  );
}
