import { For, Show } from "solid-js";

import { currentLanguage, t, tCategory, tCoach } from "../../i18n";
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
            {t("studio.coachFallback")}
          </div>
        </div>
      }
    >
      {(move) => {
        const meta = () => CATEGORY_META[move().classification];
        const explanation = (): string | null => {
          const tag = move().explanationTag;
          const params = move().explanationParams;
          if (tag && params) {
            const text = tCoach(tag, params);
            if (text) return text;
          }
          // Older analyses only have Russian prose. Keep it for Russian users,
          // but never leak it into the English interface.
          return currentLanguage() === "ru" ? move().explanationText : null;
        };
        return (
          <div class="coach-card">
            <div class="coach-head">
              <span class="badge" style={{ color: meta().color, "border-color": meta().color }}>
                <CategoryGlyph category={move().classification} size={12} /> {tCategory(move().classification)}
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
            <Show when={explanation()}>
              {(text) => <div class="coach-text">{text()}</div>}
            </Show>
            <Show when={!explanation()}>
              <div class="coach-text muted">{t("studio.bestMove", { san: move().bestSan ?? "—" })}</div>
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
                {t("studio.showBestLine")}
              </button>
              <Show when={move().isCritical}>
                <span class="chip accent">{t("studio.criticalMoment")}</span>
              </Show>
            </div>
          </div>
        );
      }}
    </Show>
  );
}
