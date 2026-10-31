import { Show } from "solid-js";

import { t, tCategory } from "../../i18n";
import type { MoveEvaluation } from "../../types/models";
import { CATEGORY_META } from "../../types/models";
import { formatClock, formatEval, formatNumber } from "../../utils/format";

export interface ChartHover {
  ply: number;
  left: number;
  move: MoveEvaluation | null;
  win: number;
  label: string;
}

export function EvalChartTooltip(props: { state: ChartHover }) {
  return (
    <div
      class="chart-tooltip"
      style={{
        left: `${Math.max(12, Math.min(88, props.state.left))}%`,
        top: "6px",
        transform: "translateX(-50%)",
      }}
    >
      <Show when={props.state.move} fallback={<span class="muted">{t("studio.startPosition")}</span>}>
        {(move) => (
          <div style={{ display: "flex", "flex-direction": "column", gap: "2px" }}>
            <div style={{ display: "flex", gap: "6px", "align-items": "center" }}>
              <span class="mono">{props.state.label}</span>
              <span style={{ "font-weight": "700" }}>{move().playedSan}</span>
              <span style={{ color: CATEGORY_META[move().classification].color }}>
                {tCategory(move().classification)}
              </span>
            </div>
            <div class="mono" style={{ color: "var(--muted)" }}>
              {formatEval(move().evalCp, move().evalMate)}, Win{" "}
              {formatNumber(move().winProbWhite, 1)}%, {formatClock(move().timeSpentSec)}
            </div>
          </div>
        )}
      </Show>
    </div>
  );
}
