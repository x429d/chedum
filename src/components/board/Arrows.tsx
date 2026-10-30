import { For } from "solid-js";

import type { ArrowSpec, Color } from "../../types/models";
import { squareCoords } from "../../utils/chess";
import { arrowGeometry } from "./arrowGeometry";

/** Outline that separates arrows from the board and from each other. */
const ARROW_BORDER = "#0c110b";
/** Outline thickness in board cells. */
const BORDER_WIDTH = 0.06;

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
          const geometry = arrowGeometry(from, to);
          if (!geometry) return null;
          const opacity = arrow.opacity ?? 1;
          const width = arrow.width ?? 0.12;
          const border = width + BORDER_WIDTH;
          const dash = arrow.dashed ? "0.16 0.12" : undefined;
          return (
            <g opacity={opacity}>
              <g fill="none" stroke={ARROW_BORDER} stroke-linejoin="round" stroke-linecap="round">
                <path d={geometry.shaft} stroke-width={border} stroke-dasharray={dash} />
              </g>
              <polygon
                points={geometry.head}
                fill={ARROW_BORDER}
                stroke={ARROW_BORDER}
                stroke-width={BORDER_WIDTH}
                stroke-linejoin="round"
              />
              <path
                d={geometry.shaft}
                fill="none"
                stroke={arrow.color}
                stroke-width={width}
                stroke-linecap="round"
                stroke-dasharray={dash}
              />
              <polygon points={geometry.head} fill={arrow.color} />
            </g>
          );
        }}
      </For>
    </g>
  );
}
