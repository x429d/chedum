import type { Color } from "../../types/models";

export function EvalBar(props: { winProbWhite: number; label: string; orientation: Color }) {
  const percent = () => Math.max(2, Math.min(98, props.winProbWhite));
  return (
    <div class="eval-bar" title={`Вероятность победы белых: ${props.winProbWhite.toFixed(1)}%`}>
      <div class="eval-bar-track">
        <div
          class="eval-bar-white"
          style={
            props.orientation === "white"
              ? { height: `${percent()}%`, bottom: "0" }
              : { height: `${percent()}%`, top: "0" }
          }
        />
      </div>
      <span class="eval-bar-label">{props.label}</span>
    </div>
  );
}
