import { createEffect, For, on, onCleanup, Show } from "solid-js";

import { analysisStore } from "../../stores/analysisStore";
import { formatEval } from "../../utils/format";
import { Icon } from "../ui/Icon";

export function LivePanel(props: { fen: string; depth: number; multipv: number }) {
  createEffect(
    on(
      () => [props.fen, analysisStore.liveEnabled(), props.depth, props.multipv] as const,
      (state) => {
        const [fen, enabled, depth, multipv] = state;
        if (!enabled) {
          void analysisStore.stop();
          return;
        }
        void analysisStore.start(fen, depth, multipv);
      },
    ),
  );

  onCleanup(() => {
    void analysisStore.stop();
  });

  return (
    <Show when={analysisStore.liveEnabled()}>
      <div class="live-panel">
        <div class="live-head">
          <div class="engine-row">
            <Icon name="activity" size={14} />
            <span style={{ "font-size": "11px", "font-weight": "600" }}>Live-анализ</span>
            <span class="chip accent" style={{ height: "20px", "font-size": "9.5px" }}>
              depth {analysisStore.liveUpdate()?.depth ?? props.depth}
            </span>
          </div>
          <Show when={analysisStore.liveLoading()}>
            <span class="spinner" />
          </Show>
        </div>
        <Show
          when={(analysisStore.liveUpdate()?.lines.length ?? 0) > 0}
          fallback={
            <div class="sync-log" style={{ "margin-top": "6px" }}>
              Stockfish считает позицию…
            </div>
          }
        >
          <For each={analysisStore.liveUpdate()?.lines ?? []}>
            {(line) => (
              <div class="live-line">
                <span class="live-eval" style={{ color: line.winProb >= 50 ? "#9ccf8f" : "#e79a90" }}>
                  {formatEval(line.cp, line.mate)}
                </span>
                <span class="live-san">
                  {line.sanMoves.slice(0, 6).join(" ") || line.uciMoves.slice(0, 6).join(" ")}
                </span>
              </div>
            )}
          </For>
        </Show>
      </div>
    </Show>
  );
}
