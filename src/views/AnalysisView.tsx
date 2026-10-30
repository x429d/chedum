import { createEffect, createSignal, on, onCleanup, onMount, Show } from "solid-js";

import { AnalysisBoard } from "../components/analytics/AnalysisBoard";
import { AnalysisHeader } from "../components/analytics/AnalysisHeader";
import { KeyMomentsTab } from "../components/analytics/KeyMomentsTab";
import { MovesTab } from "../components/analytics/MovesTab";
import { ReportTab } from "../components/analytics/ReportTab";
import { Icon } from "../components/ui/Icon";
import { analysisStore } from "../stores/analysisStore";
import { gameStore } from "../stores/gameStore";
import { libraryStore } from "../stores/libraryStore";
import { ui } from "../stores/uiStore";

type Tab = "report" | "moves" | "moments";

export function AnalysisView() {
  const [tab, setTab] = createSignal<Tab>("report");
  const [fullscreen, setFullscreen] = createSignal(false);
  const [startingAnalysis, setStartingAnalysis] = createSignal(false);

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
      // Cmd/Ctrl+Z — отменить ход, Cmd/Ctrl+Shift+Z и Ctrl+Y — вернуть.
      if (event.metaKey || event.ctrlKey) {
        if (event.altKey) return;
        const key = event.key.toLowerCase();
        // На нелатинской раскладке key может быть другим, поэтому смотрим и code.
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
          "Остановить текущий пакетный анализ и переанализировать эту партию?",
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
              {gameStore.loading() ? "Открываю партию…" : "Партия не выбрана"}
            </div>
            <div style={{ "font-size": "12px" }}>
              Выберите партию в библиотеке или запустите демо-анализ.
            </div>
            <button
              type="button"
              class="btn btn-primary btn-sm"
              onClick={() => ui.setView("library")}
            >
              <Icon name="library" size={13} />
              В библиотеку
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
                    {gameStore.progress()?.message ?? "Запускаю Stockfish…"}
                  </div>
                  <button
                    type="button"
                    class="btn btn-ghost btn-sm"
                    onClick={() => void gameStore.cancelAnalysis()}
                  >
                    Остановить
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
                    Отчёт
                  </button>
                  <button
                    type="button"
                    class="tab"
                    classList={{ active: tab() === "moves" }}
                    onClick={() => setTab("moves")}
                  >
                    Ходы и тренер
                  </button>
                  <button
                    type="button"
                    class="tab"
                    classList={{ active: tab() === "moments" }}
                    onClick={() => setTab("moments")}
                  >
                    Моменты и Retry
                  </button>
                  <Show when={fullscreen()}>
                    <button
                      type="button"
                      class="btn btn-ghost btn-sm"
                      style={{ "flex": "none", "margin-left": "auto" }}
                      title="Выйти из полноэкранного режима"
                      onClick={() => setFullscreen(false)}
                    >
                      <Icon name="close" size={13} />
                      Закрыть
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
                          Партия ещё не разобрана
                        </div>
                        <div style={{ "font-size": "11.5px", "max-width": "280px" }}>
                          Запустите Stockfish, чтобы получить разбор партии.
                        </div>
                        <button
                          type="button"
                          class="btn btn-primary btn-sm"
                          disabled={gameStore.analyzing() || startingAnalysis()}
                          onClick={startAnalysis}
                        >
                          <Icon name="analyze" size={12} />
                          Анализировать
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
