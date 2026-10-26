import { Show } from "solid-js";

import { analysisStore } from "../../stores/analysisStore";
import { gameStore } from "../../stores/gameStore";
import { libraryStore } from "../../stores/libraryStore";
import { settingsStore } from "../../stores/settingsStore";
import type { BoardTheme } from "../../types/models";
import { Board } from "../board/Board";
import { BoardControls } from "../board/BoardControls";
import { EvalBar } from "../board/EvalBar";
import { LivePanel } from "./LivePanel";

const THEMES: BoardTheme[] = ["midnight", "walnut", "slate"];

export function AnalysisBoard(props: { fullscreen: boolean; onFullscreen: () => void }) {
  const cycleTheme = (): void => {
    const current = settingsStore.settings().boardTheme;
    const index = THEMES.indexOf(current);
    const next = THEMES[(index + 1) % THEMES.length] ?? "midnight";
    void settingsStore.update({ boardTheme: next });
  };

  return (
    <div class="board-column">
      <div class="board-stage">
        <EvalBar
          winProbWhite={gameStore.winProbWhite()}
          label={gameStore.evalLabel()}
          orientation={gameStore.orientation()}
        />
        <Board
          fen={gameStore.boardFen()}
          orientation={gameStore.orientation()}
          theme={settingsStore.settings().boardTheme}
          lastMove={gameStore.lastMoveUci()}
          arrows={gameStore.arrows()}
          badge={gameStore.badge()}
          checkSquare={gameStore.checkSquareName()}
          interactive={!gameStore.preview()}
          fullscreen={props.fullscreen}
          onMove={(from, to) => {
            gameStore.enterVariation(from, to);
          }}
        />
      </div>

      <BoardControls
        ply={gameStore.ply()}
        total={gameStore.moves().length}
        theme={settingsStore.settings().boardTheme}
        showArrows={settingsStore.settings().showArrows}
        liveEnabled={analysisStore.liveEnabled()}
        liveDisabled={gameStore.analyzing() || libraryStore.batch().running}
        fullscreen={props.fullscreen}
        onFirst={() => gameStore.first()}
        onPrevious={() => gameStore.previous()}
        onNext={() => gameStore.next()}
        onLast={() => gameStore.last()}
        onFlip={() => gameStore.flip()}
        onTheme={cycleTheme}
        onArrows={() =>
          void settingsStore.update({ showArrows: !settingsStore.settings().showArrows })
        }
        onLive={() => {
          if (!gameStore.analyzing() && !libraryStore.batch().running) analysisStore.toggle();
        }}
        onFullscreen={props.onFullscreen}
      />

      <Show when={gameStore.variation()}>
        {(line) => (
          <div class="card" style={{ padding: "8px 12px", "font-size": "11.5px" }}>
            <span class="muted">Вариант: </span>
            <span class="mono">{line().sans.join(" ")}</span>
            <button
              type="button"
              class="btn btn-ghost btn-sm"
              style={{ "margin-left": "8px" }}
              onClick={() => gameStore.clearVariation()}
            >
              Вернуться к партии
            </button>
          </div>
        )}
      </Show>
      <Show when={gameStore.preview()}>
        <div class="card" style={{ padding: "8px 12px", "font-size": "11.5px" }}>
          <span style={{ color: "var(--accent-bright)" }}>Показ главной линии…</span>
          <button
            type="button"
            class="btn btn-ghost btn-sm"
            style={{ "margin-left": "8px" }}
            onClick={() => gameStore.stopPreview()}
          >
            Остановить
          </button>
        </div>
      </Show>

      <LivePanel
        fen={gameStore.boardFen()}
        depth={settingsStore.settings().analysisDepth}
        multipv={settingsStore.settings().multipv}
      />
    </div>
  );
}
