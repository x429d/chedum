import { createMemo, createSignal, Show } from "solid-js";

import { t } from "../../i18n";
import { analysisStore } from "../../stores/analysisStore";
import { gameStore } from "../../stores/gameStore";
import { libraryStore } from "../../stores/libraryStore";
import { settingsStore } from "../../stores/settingsStore";
import type { BoardTheme } from "../../types/models";
import { sideToMove } from "../../utils/chess";
import { formatEval } from "../../utils/format";
import { Board } from "../board/Board";
import { BoardControls } from "../board/BoardControls";
import { EvalBar } from "../board/EvalBar";
import { EvalChart } from "./EvalChart";
import { LivePanel } from "./LivePanel";

const THEMES: BoardTheme[] = ["midnight", "walnut", "slate"];

export function AnalysisBoard(props: { fullscreen: boolean; onFullscreen: () => void }) {
  /** Оценка live-движка в перспективе белых; UCI отдаёт cp/mate от ходящего. */
  const liveEval = createMemo(() => {
    if (!analysisStore.liveEnabled()) return null;
    const update = analysisStore.liveUpdate();
    if (!update || update.fen !== gameStore.boardFen()) return null;
    const line = update.lines[0];
    if (!line) return null;
    const blackToMove = sideToMove(update.fen) === "black";
    const cp = blackToMove ? (line.cp === null ? null : -line.cp) : line.cp;
    const mate = blackToMove ? (line.mate === null ? null : -line.mate) : line.mate;
    const winProbWhite = blackToMove ? 100 - line.winProb : line.winProb;
    return {
      winProbWhite: Math.max(0, Math.min(100, winProbWhite)),
      label: formatEval(cp, mate),
    };
  });

  const [livePending, setLivePending] = createSignal(false);

  const requestLive = (): void => {
    if (livePending()) return;
    void (async () => {
      if (gameStore.analyzing()) return;
      if (libraryStore.batch().running) {
        setLivePending(true);
        try {
          const confirmed = await libraryStore.stopBatchWithConfirm(
            t("studio.confirmStopBatch"),
          );
          if (!confirmed || gameStore.analyzing()) return;
          analysisStore.setLiveEnabled(true);
        } finally {
          setLivePending(false);
        }
        return;
      }
      analysisStore.toggle();
    })();
  };

  const cycleTheme = (): void => {
    const current = settingsStore.settings().boardTheme;
    const index = THEMES.indexOf(current);
    const next = THEMES[(index + 1) % THEMES.length] ?? "midnight";
    void settingsStore.update({ boardTheme: next });
  };

  return (
    <div class="board-column" classList={{ "is-live": analysisStore.liveEnabled() }}>
      <div class="board-stage">
        <EvalBar
          winProbWhite={liveEval()?.winProbWhite ?? gameStore.winProbWhite()}
          label={liveEval()?.label ?? gameStore.evalLabel()}
          orientation={gameStore.orientation()}
        />
        <Board
          fen={gameStore.boardFen()}
          orientation={gameStore.orientation()}
          theme={settingsStore.settings().boardTheme}
          pieceSet={settingsStore.settings().pieceSet}
          lastMove={gameStore.lastMoveUci()}
          arrows={gameStore.arrows()}
          badge={gameStore.badge()}
          checkSquare={gameStore.checkSquareName()}
          interactive={!gameStore.preview()}
          userArrows={gameStore.preview() ? [] : gameStore.userArrows()}
          onUserArrowsChange={gameStore.updateUserArrows}
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
        liveDisabled={gameStore.analyzing() || livePending()}
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
        onLive={requestLive}
        onFullscreen={props.onFullscreen}
      />

      <Show when={gameStore.variation()}>
        {(line) => (
          <div class="card" style={{ padding: "8px 12px", "font-size": "11.5px" }}>
            <span class="muted">{t("studio.variation")} </span>
            <span class="mono">{line().sans.join(" ")}</span>
            <button
              type="button"
              class="btn btn-ghost btn-sm"
              style={{ "margin-left": "8px" }}
              onClick={() => gameStore.clearVariation()}
            >
              {t("studio.returnToGame")}
            </button>
          </div>
        )}
      </Show>
      <Show when={gameStore.preview()}>
        <div class="card" style={{ padding: "8px 12px", "font-size": "11.5px" }}>
          <span style={{ color: "var(--accent-bright)" }}>{t("studio.showingMainLine")}</span>
          <button
            type="button"
            class="btn btn-ghost btn-sm"
            style={{ "margin-left": "8px" }}
            onClick={() => gameStore.stopPreview()}
          >
            {t("studio.stop")}
          </button>
        </div>
      </Show>

      <LivePanel
        fen={gameStore.boardFen()}
        depth={settingsStore.settings().analysisDepth}
        multipv={settingsStore.settings().multipv}
      />

      <Show when={gameStore.moves().length > 0 && gameStore.analysis()}>
        <div class="card chart-card">
          <div class="chart-header">
            <div>
              <div class="card-title">{t("studio.dualChart")}</div>
            </div>
            <div class="chart-legend">
              <span>
                <span class="legend-dot" style={{ background: "#e0bf7d" }} />
                {t("studio.winProbWhite")}
              </span>
              <span>
                <span class="legend-dot" style={{ background: "#e05c4f" }} />
                {t("studio.timeTroubleOrImpulsive")}
              </span>
              <span>
                <span class="legend-dot" style={{ background: "#4a5344" }} />
                {t("studio.moveTimeLegend")}
              </span>
            </div>
          </div>
          <EvalChart
            moves={gameStore.moves()}
            totalPlies={gameStore.moves().length}
            currentPly={gameStore.ply()}
            onSelect={(ply) => gameStore.goTo(ply)}
          />
        </div>
      </Show>
    </div>
  );
}
