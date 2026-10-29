import { For, Show } from "solid-js";

import type { MoveEvaluation } from "../../types/models";
import { CATEGORY_META } from "../../types/models";
import { formatClock, formatEval } from "../../utils/format";
import { CategoryGlyph } from "../ui/CategoryGlyph";
import { Icon } from "../ui/Icon";

export function CoachCard(props: { move: MoveEvaluation | null; onPlayLine: () => void }) {
  return (
    <Show
      when={props.move}
      fallback={
        <div class="coach-card">
          <div class="muted" style={{ "font-size": "11.5px", "line-height": "1.5" }}>
            Выберите ход на доске или в списке — тренер объяснит решение, покажет линии Stockfish и
            причину ошибки.
          </div>
        </div>
      }
    >
      {(move) => {
        const meta = () => CATEGORY_META[move().classification];
        return (
          <div class="coach-card">
            <div class="coach-head">
              <span class="badge" style={{ color: meta().color, "border-color": meta().color }}>
                <CategoryGlyph category={move().classification} size={12} /> {meta().label}
              </span>
              <span class="mono" style={{ "font-size": "11px" }}>
                {move().ply}. {move().playedSan}
              </span>
              <span class="coach-eval">
                {formatEval(move().bestEvalCp, move().bestEvalMate)} →{" "}
                {formatEval(move().evalCp, move().evalMate)}
              </span>
              <span style={{ "margin-left": "auto", "font-size": "10.5px", color: "var(--muted)" }}>
                {formatClock(move().timeSpentSec)}, Δ{" "}
                {move().winProbLoss.toFixed(1)}%
              </span>
            </div>
            <Show when={move().explanationText}>
              <div class="coach-text">{move().explanationText}</div>
            </Show>
            <Show when={!move().explanationText}>
              <div class="coach-text muted">Лучший ход: {move().bestSan ?? "—"}.</div>
            </Show>
            <div class="pv-list">
              <For each={move().pvLines.slice(0, 2)}>
                {(line) => (
                  <div class="pv-line">
                    <span class="pv-rank">{line.rank}</span>
                    <span class="mono" style={{ color: "var(--text)" }}>
                      {formatEval(line.scoreCp, line.scoreMate)}
                    </span>
                    <span class="pv-san">{line.sanMoves.slice(0, 7).join(" ")}</span>
                  </div>
                )}
              </For>
            </div>
            <div style={{ display: "flex", gap: "6px", "margin-top": "10px", "flex-wrap": "wrap" }}>
              <button
                type="button"
                class="btn btn-sm"
                disabled={move().pvLines.length === 0}
                onClick={props.onPlayLine}
              >
                <Icon name="play" size={12} />
                Показать лучшую линию
              </button>
              <Show when={move().isCritical}>
                <span class="chip accent">Критический момент</span>
              </Show>
            </div>
          </div>
        );
      }}
    </Show>
  );
}
