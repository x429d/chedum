import { For } from "solid-js";

import type { ArrowSpec, Color } from "../../types/models";
import { squareCoords } from "../../utils/chess";

function center(name: string, orientation: Color): { x: number; y: number } | null {
  const coords = squareCoords(name);
  if (!coords) return null;
  const col = orientation === "white" ? coords.file : 7 - coords.file;
  const row = orientation === "white" ? 7 - coords.rank : coords.rank;
  return { x: col + 0.5, y: row + 0.5 };
}

export function Arrows(props: { arrows: ArrowSpec[]; orientation: Color }) {
  return (
    <g style={{ "pointer-events": "none" }}>
      <For each={props.arrows}>
        {(arrow) => {
          const from = center(arrow.from, props.orientation);
          const to = center(arrow.to, props.orientation);
          if (!from || !to) return null;
          const dx = to.x - from.x;
          const dy = to.y - from.y;
          const length = Math.hypot(dx, dy);
          if (length < 0.05) return null;
          const ux = dx / length;
          const uy = dy / length;
          const startX = from.x + ux * 0.24;
          const startY = from.y + uy * 0.24;
          const tipX = to.x - ux * 0.06;
          const tipY = to.y - uy * 0.06;
          const baseX = to.x - ux * 0.34;
          const baseY = to.y - uy * 0.34;
          const px = -uy;
          const py = ux;
          const opacity = arrow.opacity ?? 1;
          const width = arrow.width ?? 0.11;
          const head = 0.17;
          return (
            <g opacity={opacity}>
              <line
                x1={startX}
                y1={startY}
                x2={baseX}
                y2={baseY}
                stroke={arrow.color}
                stroke-width={width}
                stroke-linecap="round"
                stroke-dasharray={arrow.dashed ? "0.16 0.12" : undefined}
              />
              <polygon
                points={`${tipX},${tipY} ${baseX + px * head},${baseY + py * head} ${baseX - px * head},${baseY - py * head}`}
                fill={arrow.color}
              />
            </g>
          );
        }}
      </For>
    </g>
  );
}
