import { createMemo, For, Show } from "solid-js";

import type { MoveEvaluation } from "../../types/models";
import { CATEGORY_META } from "../../types/models";
import { formatEval } from "../../utils/format";
import { CategoryGlyph } from "../ui/CategoryGlyph";

export function KeyMoments(props: { moves: MoveEvaluation[]; onSelect: (ply: number) => void }) {
  const moments = createMemo(() =>
    props.moves.filter((move) => move.isCritical || move.classification === "brilliant" ||
      move.classification === "great" || move.classification === "miss" ||
      move.classification === "mistake" || move.classification === "blunder"),
  );
  return (
    <Show
      when={moments().length > 0}
      fallback={<div class="empty-state" style={{ padding: "24px" }}>Ключевых моментов нет.</div>}
    >
      <div>
        <div class="card-title" style={{ "margin-bottom": "8px" }}>
          Ключевые моменты
        </div>
        <For each={moments()}>
          {(move) => {
            const meta = CATEGORY_META[move.classification];
            return (
              <button type="button" class="key-moment" onClick={() => props.onSelect(move.ply)}>
                <span class="category-symbol" style={{ background: meta.color }}>
                  <CategoryGlyph category={move.classification} size={12} />
                </span>
                <span class="mono" style={{ "font-size": "10.5px", color: "var(--muted)" }}>
                  {move.ply}.
                </span>
                <span style={{ "font-weight": "700", "font-size": "12px" }}>{move.playedSan}</span>
                <span class="mono" style={{ "font-size": "10px", color: meta.color }}>
                  {formatEval(move.evalCp, move.evalMate)}
                </span>
                <span class="dim" style={{ "font-size": "10px", "margin-left": "auto" }}>
                  Δ {move.winProbLoss.toFixed(1)}%
                </span>
              </button>
            );
          }}
        </For>
      </div>
    </Show>
  );
}
