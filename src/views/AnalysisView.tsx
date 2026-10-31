import { createEffect, createSignal, on, onCleanup, onMount, Show } from "solid-js";

import { AnalysisBoard } from "../components/analytics/AnalysisBoard";
import { AnalysisHeader } from "../components/analytics/AnalysisHeader";
import { KeyMomentsTab } from "../components/analytics/KeyMomentsTab";
import { MovesTab } from "../components/analytics/MovesTab";
import { ReportTab } from "../components/analytics/ReportTab";
import { Icon } from "../components/ui/Icon";
import { t } from "../i18n";
import { analysisStore } from "../stores/analysisStore";
import { gameStore } from "../stores/gameStore";
import { libraryStore } from "../stores/libraryStore";
import { ui } from "../stores/uiStore";
import { analysisProgressLabel } from "../utils/progress";

type Tab = "report" | "moves" | "moments";

export function AnalysisView() {
  const [tab, setTab] = createSignal<Tab>("report");
  const [fullscreen, setFullscreen] = createSignal(false);
  const [startingAnalysis, setStartingAnalysis] = createSignal(false);
  const progressLabel = (): string => {
    const progress = gameStore.progress();
    return progress ? analysisProgressLabel(progress) : t("studio.startingStockfish");
  };

  createEffect(
    on(
      () => libraryStore.selectedGameId(),
      (id) => {
        if (id !== null) void gameStore.load(id);
      },
    ),
  );

  createEffect(
    on(
      () => libraryStore.batch().running,
      (running) => {
        if (running) {
          analysisStore.setLiveEnabled(false);
          void analysisStore.stop();
        }
      },
    ),
  );

  createEffect(
    on(fullscreen, (active) => {
      if (!active) return;
      const handler = (event: KeyboardEvent): void => {
        if (event.key === "Escape") setFullscreen(false);
      };
      window.addEventListener("keydown", handler);
      onCleanup(() => window.removeEventListener("keydown", handler));
    }),
  );

  onMount(() => {
    const handler = (event: KeyboardEvent): void => {
      if (event.defaultPrevented) return;
      const target = event.target;
      if (target instanceof HTMLElement) {
        const tag = target.tagName;
        if (
          tag === "INPUT" ||
          tag === "TEXTAREA" ||
          tag === "SELECT" ||
          target.isContentEditable
        ) {
          return;
        }
      }
      // Cmd/Ctrl+Z undoes a move; Cmd/Ctrl+Shift+Z and Ctrl+Y redo it.
      if (event.metaKey || event.ctrlKey) {
        if (event.altKey) return;
        const key = event.key.toLowerCase();
        // Non-Latin layouts can report a different `key`, so check `code` too.
        const undoKey = key === "z" || event.code === "KeyZ";
        const redoKey = key === "y" || event.code === "KeyY";
        if (undoKey) {
          event.preventDefault();
          if (event.shiftKey) gameStore.redo();
          else gameStore.undo();
        } else if (redoKey) {
          event.preventDefault();
          gameStore.redo();
        }
        return;
      }
      if (event.altKey) return;
      if (event.key === "ArrowLeft") {
        event.preventDefault();
        gameStore.previous();
      } else if (event.key === "ArrowRight") {
        event.preventDefault();
        gameStore.next();
      } else if (event.key === "Home") {
        event.preventDefault();
        gameStore.first();
      } else if (event.key === "End") {
        event.preventDefault();
        gameStore.last();
      }
    };
    window.addEventListener("keydown", handler);
    onCleanup(() => window.removeEventListener("keydown", handler));
  });

  onCleanup(() => {
    gameStore.stopPreview();
    void analysisStore.stop();
  });

  const detail = () => gameStore.detail();
  const analysis = () => gameStore.analysis();
  const startAnalysis = (): void => {
    if (startingAnalysis()) return;
    setStartingAnalysis(true);
    void (async () => {
      try {
        const confirmed = await libraryStore.stopBatchWithConfirm(
          t("studio.confirmStopBatch"),
        );
        if (!confirmed) return;
        analysisStore.setLiveEnabled(false);
        await analysisStore.stop();
        await gameStore.runAnalysis();
      } finally {
        setStartingAnalysis(false);
      }
    })();
  };

  return (
    <div class="view-scroll">
      <Show
        when={detail()}
        fallback={
          <div class="empty-state" style={{ "min-height": "60vh" }}>
            <div class="empty-icon">
              <Icon name="analysis" size={22} />
            </div>
            <div style={{ "font-weight": "600", color: "var(--text)" }}>
              {gameStore.loading() ? t("studio.loadingGame") : t("studio.emptyTitle")}
            </div>
            <div style={{ "font-size": "12px" }}>
              {t("studio.emptyHint")}
            </div>
            <button
              type="button"
              class="btn btn-primary btn-sm"
              onClick={() => ui.setView("library")}
            >
              <Icon name="library" size={13} />
              {t("studio.toLibraryBtn")}
            </button>
          </div>
        }
      >
        {(data) => (
          <div class="fade-in">
            <AnalysisHeader
              game={data().game}
              analysis={analysis()}
              analyzing={gameStore.analyzing() || startingAnalysis()}
              onAnalyze={startAnalysis}
              onBack={() => ui.setView("library")}
            />

            <Show when={gameStore.analyzing()}>
              <div class="card" style={{ padding: "10px 14px", "margin-bottom": "12px" }}>
                <div class="progress-track">
                  <div
                    class="progress-fill"
                    style={{ width: `${gameStore.progress()?.percent ?? 0}%` }}
                  />
                </div>
                <div style={{ display: "flex", "align-items": "center", gap: "8px" }}>
                  <div class="sync-log" style={{ flex: "1" }}>
                    {progressLabel()}
                  </div>
                  <button
                    type="button"
                    class="btn btn-ghost btn-sm"
                    onClick={() => void gameStore.cancelAnalysis()}
                  >
                    {t("studio.stop")}
                  </button>
                </div>
              </div>
            </Show>

            <div class="analysis-layout" classList={{ "is-fullscreen": fullscreen() }}>
              <AnalysisBoard
                fullscreen={fullscreen()}
                onFullscreen={() => setFullscreen((value) => !value)}
              />

              <div class="card analysis-panel">
                <div class="tabs">
                  <button
                    type="button"
                    class="tab"
                    classList={{ active: tab() === "report" }}
                    onClick={() => setTab("report")}
                  >
                    {t("studio.reportTab")}
                  </button>
                  <button
                    type="button"
                    class="tab"
                    classList={{ active: tab() === "moves" }}
                    onClick={() => setTab("moves")}
                  >
                    {t("studio.movesTab")}
                  </button>
                  <button
                    type="button"
                    class="tab"
                    classList={{ active: tab() === "moments" }}
                    onClick={() => setTab("moments")}
                  >
                    {t("studio.momentsTab")}
                  </button>
                  <Show when={fullscreen()}>\n                    <button
                      type="button"
                      class="btn btn-ghost btn-sm"
                      style={{ "flex": "none", "margin-left": "auto" }}
                      title={t("studio.closeFullscreen")}
                      onClick={() => setFullscreen(false)}
                    >
                      <Icon name="close" size={13} />
                      {t("studio.closeFullscreen")}
                    </button>
                  </Show>
                </div>
                <div class="tab-body">
                  <Show
                    when={analysis()}
                    fallback={
                      <div class="empty-state" style={{ padding: "26px 14px" }}>
                        <div class="empty-icon">
                          <Icon name="analyze" size={20} />
                        </div>
                        <div style={{ "font-weight": "600", color: "var(--text)" }}>
                          {t("studio.notAnalyzedTitle")}
                        </div>
                        <div style={{ "font-size": "11.5px", "max-width": "280px" }}>
                          {t("studio.notAnalyzedHint")}
                        </div>
                        <button
                          type="button"
                          class="btn btn-primary btn-sm"
                          disabled={gameStore.analyzing() || startingAnalysis()}
                          onClick={startAnalysis}
                        >
                          <Icon name="analyze" size={12} />
                          {t("studio.analyze")}
                        </button>
                      </div>
                    }
                  >
                    {(value) => (
                      <>
                        <Show when={tab() === "report"}>
                          <ReportTab
                            analysis={value()}
                            game={data().game}
                            moves={gameStore.moves()}
                          />
                        </Show>
                        <Show when={tab() === "moves"}>
                          <MovesTab />
                        </Show>
                        <Show when={tab() === "moments"}>
                          <KeyMomentsTab />
                        </Show>
                      </>
                    )}
                  </Show>
                </div>
              </div>
            </div>
          </div>
        )}
      </Show>
    </div>
  );
}
