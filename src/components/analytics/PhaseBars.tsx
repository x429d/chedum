import { For } from "solid-js";

import type { GameAnalysis, Phase, UserColor } from "../../types/models";
import { PHASE_LABELS } from "../../types/models";

export function PhaseBars(props: { analysis: GameAnalysis; userColor: UserColor }) {
  const entries = (): { phase: Phase; value: number | null }[] => {
    const black = props.userColor === "black";
    return [
      { phase: "opening", value: black ? props.analysis.blackOpeningAcc : props.analysis.whiteOpeningAcc },
      {
        phase: "middlegame",
        value: black ? props.analysis.blackMiddlegameAcc : props.analysis.whiteMiddlegameAcc,
      },
      { phase: "endgame", value: black ? props.analysis.blackEndgameAcc : props.analysis.whiteEndgameAcc },
    ];
  };
  return (
    <div>
      <For each={entries()}>
        {(entry) => (
          <div class="phase-row">
            <span class="muted">{PHASE_LABELS[entry.phase]}</span>
            <div class="phase-track">
              <div
                class="phase-fill"
                style={{ width: `${entry.value === null ? 0 : Math.max(2, entry.value)}%` }}
              />
            </div>
            <span class="mono" style={{ "text-align": "right" }}>
              {entry.value === null ? "—" : `${entry.value.toFixed(0)}%`}
            </span>
          </div>
        )}
      </For>
    </div>
  );
}
