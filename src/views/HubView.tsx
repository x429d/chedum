import { createEffect, createSignal, For, on, onMount, Show } from "solid-js";

import { AccountScope } from "../components/accounts/AccountScope";
import { AccuracyTrend } from "../components/hub/AccuracyTrend";
import { KpiCards } from "../components/hub/KpiCards";
import { OpeningMatrix } from "../components/hub/OpeningMatrix";
import { PhaseWeakness } from "../components/hub/PhaseWeakness";
import { RatingPlatforms } from "../components/hub/RatingPlatforms";
import { TimeStatsCard } from "../components/hub/TimeStatsCard";
import { Masonry } from "../components/layout/Masonry";
import { Icon } from "../components/ui/Icon";
import { pluralGames, t, tSide } from "../i18n";
import { api } from "../services/api";
import { accountsStore } from "../stores/accountsStore";
import { dataVersion } from "../stores/dataVersion";
import { libraryStore } from "../stores/libraryStore";
import { settingsStore } from "../stores/settingsStore";
import { ui } from "../stores/uiStore";
import type { ColorRecord, HubStats } from "../types/models";
import { formatNumber, toErrorMessage } from "../utils/format";

export function HubView() {
  const [stats, setStats] = createSignal<HubStats | null>(null);
  const [loading, setLoading] = createSignal(false);
  const [reanalyzing, setReanalyzing] = createSignal(false);

  const load = async (): Promise<void> => {
    setLoading(true);
    try {
      setStats(await api.getHubStats(accountsStore.activeAccountId()));
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
      () => [accountsStore.activeAccountId(), libraryStore.total(), libraryStore.analyzed(), dataVersion()] as const,
      () => {
        if (ui.view() === "hub") void load();
      },
      { defer: true },
    ),
  );

  const colorCards = (): { color: string; record: ColorRecord }[] => {
    const value = stats();
    if (!value) return [];
    const cap = (s: string) => s.charAt(0).toUpperCase() + s.slice(1);
    return [
      { color: cap(tSide("white")), record: value.kpis.white },
      { color: cap(tSide("black")), record: value.kpis.black },
    ];
  };

  return (
    <div class="view-scroll">
      <header class="view-header">
        <div>
          <h1 class="view-title">{t("hub.title")}</h1>
        </div>
        <div class="header-actions">
          <AccountScope />
          <button type="button" class="btn" onClick={() => void load()} disabled={loading()}>
            <Show when={loading()} fallback={<Icon name="refresh" size={14} />}>
              <span class="spinner" style={{ width: "13px", height: "13px" }} />
            </Show>
            {t("hub.refresh")}
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
              {loading() ? t("hub.calculating") : t("hub.noData")}
            </div>
            <div style={{ "font-size": "12px" }}>
              {t("hub.noDataHint")}
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
                  {t("hub.noUserGames")}
                </div>
                <div style={{ "font-size": "12px", "max-width": "380px" }}>
                  {t("hub.noUserGamesHint")}
                </div>
                <button
                  type="button"
                  class="btn btn-primary btn-sm"
                  onClick={() => ui.setView("accounts")}
                >
                  <Icon name="user" size={13} />
                  {t("accounts.manage")}
                </button>
              </div>
            }
          >
            <Show when={value().config}>
              {(config) => (
                <div class="config-strip">
                  <div>
                    <strong>
                      {t("hub.configLabel", {
                        engine: config().engineName,
                        depth: config().depth,
                        multipv: config().multipv,
                      })}
                    </strong>
                    <span class="dim">
                      {" "}
                      — {config().included} / {config().total}
                    </span>
                  </div>
                  <Show when={config().upgradeGameIds.length > 0}>
                    <button
                      type="button"
                      class="btn btn-sm"
                      disabled={reanalyzing() || libraryStore.batch().running}
                      onClick={() => {
                        const ids = config().upgradeGameIds;
                        setReanalyzing(true);
                        void libraryStore
                          .analyzeIds(ids, t("library.batchAnalysisLabel"))
                          .then(() => load())
                          .finally(() => setReanalyzing(false));
                      }}
                    >
                      <Icon name="analyze" size={12} />
                      {reanalyzing()
                        ? t("hub.reanalyzing")
                        : t("hub.reanalyzeAtDepth", {
                            depth: settingsStore.settings().analysisDepth,
                            count: config().upgradeGameIds.length,
                          })}
                    </button>
                  </Show>
                  <Show when={config().mismatchedGameIds.length > 0 && config().upgradeGameIds.length === 0}>
                    <span class="dim" style={{ "font-size": "10.5px" }}>
                      {t("hub.reanalyzeNotNeeded")}
                    </span>
                  </Show>
                </div>
              )}
            </Show>

            <KpiCards kpis={value().kpis} />

            <Masonry>
              <RatingPlatforms ratings={value().ratings} />

              <div class="card">
                <div class="card-header">
                  <div>
                    <div class="card-title">{t("hub.colorResults")}</div>
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
                          "border-bottom": "1px solid var(--border)",
                        }}
                      >
                        <span style={{ "font-weight": "600", "font-size": "12px" }}>{item.color}</span>
                        <div style={{ display: "flex", gap: "8px", "align-items": "center" }}>
                          <span class="badge outcome-win">{t("hub.winsCount", { n: item.record.wins })}</span>
                          <span class="badge outcome-draw">{t("hub.drawsCount", { n: item.record.draws })}</span>
                          <span class="badge outcome-loss">{t("hub.lossesCount", { n: item.record.losses })}</span>
                          <span class="dim" style={{ "font-size": "10.5px" }}>
                            {pluralGames(item.record.games)}
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

              <TimeStatsCard time={value().time} />
              <OpeningMatrix openings={value().openings} />
              <PhaseWeakness phases={value().phases} />
              <AccuracyTrend points={value().trend} />
            </Masonry>
          </Show>
        )}
      </Show>
    </div>
  );
}
