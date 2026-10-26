import { createEffect, createSignal, on, onCleanup, Show } from "solid-js";

import { AnalysisBoard } from "../components/analytics/AnalysisBoard";
import { AnalysisHeader } from "../components/analytics/AnalysisHeader";
import { EvalChart } from "../components/analytics/EvalChart";
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

  onCleanup(() => {
    gameStore.stopPreview();
    void analysisStore.stop();
  });

  const detail = () => gameStore.detail();
  const analysis = () => gameStore.analysis();
  const startAnalysis = (): void => {
    if (libraryStore.batch().running) {
      ui.notify("Дождитесь завершения пакетного анализа", "info");
      return;
    }
    analysisStore.setLiveEnabled(false);
    void (async () => {
      await analysisStore.stop();
      await gameStore.runAnalysis();
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
              analyzing={gameStore.analyzing()}
              busy={libraryStore.batch().running}
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

            <div class="analysis-layout">
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
                </div>
                <div class="tab-body">
                  <Show
                    when={analysis()}
                    fallback={
                      <div class="empty-state" style={{ padding: "26px 14px" }}>
                        <div class="empty-icon">
                          <Icon name="sparkles" size={20} />
                        </div>
                        <div style={{ "font-weight": "600", color: "var(--text)" }}>
                          Партия ещё не разобрана
                        </div>
                        <div style={{ "font-size": "11.5px", "max-width": "280px" }}>
                          Запустите локальный Stockfish: точность, 11 категорий, фазы и объяснения
                          тренера.
                        </div>
                        <button
                          type="button"
                          class="btn btn-primary btn-sm"
                          disabled={gameStore.analyzing() || libraryStore.batch().running}
                          onClick={startAnalysis}
                        >
                          <Icon name="sparkles" size={12} />
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

            <Show when={gameStore.moves().length > 0 && analysis()}>
              <div class="card chart-card">
                <div class="chart-header">
                  <div>
                    <div class="card-title">Двойная аналитика</div>
                    <div class="card-subtitle">Win% и время на ход · клик переводит доску</div>
                  </div>
                  <div class="chart-legend">
                    <span>
                      <span class="legend-dot" style={{ background: "#2dd4bf" }} />
                      Win% белых
                    </span>
                    <span>
                      <span class="legend-dot" style={{ background: "#ef4444" }} />
                      цейтнот / поспешность
                    </span>
                    <span>
                      <span class="legend-dot" style={{ background: "#475569" }} />
                      время
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

            <Show when={fullscreen()}>
              <button
                type="button"
                class="btn"
                style={{
                  position: "fixed",
                  top: "28px",
                  right: "28px",
                  "z-index": 100,
                  background: "#131722",
                }}
                onClick={() => setFullscreen(false)}
              >
                <Icon name="close" size={14} />
                Закрыть
              </button>
            </Show>
          </div>
        )}
      </Show>
    </div>
  );
}
