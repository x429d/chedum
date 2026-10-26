import { createMemo, For } from "solid-js";

import type { PhaseStat } from "../../types/models";
import { PHASE_LABELS } from "../../types/models";
import { formatNumber } from "../../utils/format";

export function PhaseWeakness(props: { phases: PhaseStat[] }) {
  const weakest = createMemo(() => {
    const known = props.phases.filter((phase) => phase.accuracy !== null);
    return known.length === 0
      ? null
      : known.reduce((low, phase) =>
          (phase.accuracy ?? 0) < (low.accuracy ?? 0) ? phase : low,
        ).phase;
  });
  return (
    <div class="card">
      <div class="card-header">
        <div>
          <div class="card-title">Уязвимые фазы</div>
          <div class="card-subtitle">Точность по стадиям партии</div>
        </div>
      </div>
      <div class="card-pad">
        <For each={props.phases}>
          {(phase) => (
            <div class="phase-weak-row">
              <span
                style={{ width: "82px", "font-size": "11px" }}
                classList={{ "muted": weakest() !== phase.phase }}
              >
                {PHASE_LABELS[phase.phase]}
                <span class="dim" style={{ display: "block", "font-size": "9px" }}>
                  {phase.moves} ходов
                </span>
              </span>
              <div class="phase-track" style={{ flex: "1" }}>
                <div
                  class="phase-fill"
                  style={{
                    width: `${phase.accuracy === null ? 0 : Math.max(2, phase.accuracy)}%`,
                    background: weakest() === phase.phase ? "#f97316" : undefined,
                  }}
                />
              </div>
              <span
                class="mono"
                style={{
                  "font-size": "11px",
                  color: weakest() === phase.phase ? "#f97316" : "var(--text)",
                }}
              >
                {phase.accuracy === null ? "—" : `${formatNumber(phase.accuracy, 0)}%`}
              </span>
            </div>
          )}
        </For>
      </div>
    </div>
  );
}
