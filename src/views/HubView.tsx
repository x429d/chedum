import { createEffect, createSignal, For, on, onMount, Show } from "solid-js";

import { AccuracyTrend } from "../components/hub/AccuracyTrend";
import { KpiCards } from "../components/hub/KpiCards";
import { OpeningMatrix } from "../components/hub/OpeningMatrix";
import { PhaseWeakness } from "../components/hub/PhaseWeakness";
import { TimeStatsCard } from "../components/hub/TimeStatsCard";
import { Icon } from "../components/ui/Icon";
import { api } from "../services/api";
import { libraryStore } from "../stores/libraryStore";
import { ui } from "../stores/uiStore";
import type { ColorRecord, HubStats } from "../types/models";
import { formatNumber, toErrorMessage } from "../utils/format";

export function HubView() {
  const [stats, setStats] = createSignal<HubStats | null>(null);
  const [loading, setLoading] = createSignal(false);

  const load = async (): Promise<void> => {
    setLoading(true);
    try {
      setStats(await api.getHubStats());
    } catch (error) {
      ui.notify(toErrorMessage(error), "error");
    } finally {
      setLoading(false);
    }
  };

  onMount(() => {
    void load();
  });

  createEffect(
    on(
      () => libraryStore.total(),
      () => {
        if (ui.view() === "hub") void load();
      },
      { defer: true },
    ),
  );

  const colorCards = (): { color: string; record: ColorRecord }[] => {
    const value = stats();
    if (!value) return [];
    return [
      { color: "Белые", record: value.kpis.white },
      { color: "Чёрные", record: value.kpis.black },
    ];
  };

  return (
    <div class="view-scroll">
      <header class="view-header">
        <div>
          <h1 class="view-title">Аналитика игрока</h1>
          <div class="view-subtitle">Результаты, дебюты, фазы и время на ход.</div>
        </div>
        <div class="header-actions">
          <button type="button" class="btn" onClick={() => void load()} disabled={loading()}>
            <Show when={loading()} fallback={<Icon name="refresh" size={14} />}>
              <span class="spinner" style={{ width: "13px", height: "13px" }} />
            </Show>
            Обновить
          </button>
        </div>
      </header>

      <Show
        when={stats()}
        fallback={
          <div class="empty-state" style={{ "min-height": "50vh" }}>
            <div class="empty-icon">
              <Icon name="hub" size={22} />
            </div>
            <div style={{ "font-weight": "600", color: "var(--text)" }}>
              {loading() ? "Считаю статистику…" : "Нет данных"}
            </div>
            <div style={{ "font-size": "12px" }}>
              Добавьте партии и запустите анализ — дашборд соберётся автоматически.
            </div>
          </div>
        }
      >
        {(value) => (
          <Show
            when={value().kpis.totalGames > 0}
            fallback={
              <div class="empty-state" style={{ "min-height": "50vh" }}>
                <div class="empty-icon">
                  <Icon name="library" size={22} />
                </div>
                <div style={{ "font-weight": "600", color: "var(--text)" }}>
                  Пока нет партий с вашим цветом
                </div>
                <div style={{ "font-size": "12px", "max-width": "380px" }}>
                  Укажите свой ник в настройках — он используется для определения цвета в
                  импортированных партиях.
                </div>
                <button
                  type="button"
                  class="btn btn-primary btn-sm"
                  onClick={() => ui.setView("settings")}
                >
                  <Icon name="settings" size={13} />
                  В настройки
                </button>
              </div>
            }
          >
            <KpiCards kpis={value().kpis} />

            <div class="hub-grid">
              <OpeningMatrix openings={value().openings} />
              <PhaseWeakness phases={value().phases} />
            </div>

            <div class="hub-grid">
              <AccuracyTrend points={value().trend} />
              <TimeStatsCard time={value().time} />
            </div>

            <div class="card">
              <div class="card-header">
                <div>
                  <div class="card-title">Результаты по цветам</div>
                  <div class="card-subtitle">Победы, ничьи и точность</div>
                </div>
              </div>
              <div class="card-pad">
                <For each={colorCards()}>
                  {(item) => (
                    <div
                      style={{
                        display: "grid",
                        "grid-template-columns": "70px 1fr auto",
                        gap: "12px",
                        "align-items": "center",
                        padding: "9px 0",
                        "border-bottom": "1px solid rgba(35,42,59,0.55)",
                      }}
                    >
                      <span style={{ "font-weight": "600", "font-size": "12px" }}>{item.color}</span>
                      <div style={{ display: "flex", gap: "8px", "align-items": "center" }}>
                        <span class="badge outcome-win">{item.record.wins}W</span>
                        <span class="badge outcome-draw">{item.record.draws}D</span>
                        <span class="badge outcome-loss">{item.record.losses}L</span>
                        <span class="dim" style={{ "font-size": "10.5px" }}>
                          {item.record.games} партий
                        </span>
                      </div>
                      <span class="mono" style={{ "font-size": "12px" }}>
                        {item.record.accuracy === null
                          ? "—"
                          : `${formatNumber(item.record.accuracy, 1)}%`}
                      </span>
                    </div>
                  )}
                </For>
              </div>
            </div>
          </Show>
        )}
      </Show>
    </div>
  );
}
