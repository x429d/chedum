import { createMemo, onCleanup, onMount, Show } from "solid-js";

import { AnalysisQueue } from "../components/library/AnalysisQueue";
import { Filters } from "../components/library/Filters";
import { GameTable } from "../components/library/GameTable";
import { ImportExport } from "../components/library/ImportExport";
import { SyncPanel } from "../components/library/SyncPanel";
import { Icon } from "../components/ui/Icon";
import { pluralGames, t } from "../i18n";
import { libraryStore } from "../stores/libraryStore";
import { ui } from "../stores/uiStore";
import { formatNumber } from "../utils/format";

type SideSection = "sync" | "queue";

export function LibraryView() {
  onMount(() => void libraryStore.load());
  onCleanup(() => libraryStore.cancelPendingSearch());
  let sideSync: HTMLDivElement | undefined;
  let sideQueue: HTMLDivElement | undefined;
  const expandSidebar = (section?: SideSection): void => {
    ui.setLibrarySidebarCollapsed(false);
    if (!section) return;
    requestAnimationFrame(() => {
      const target = section === "sync" ? sideSync : sideQueue;
      target?.scrollIntoView({ block: "nearest", behavior: "smooth" });
    });
  };
  const averageAccuracy = createMemo(() => {
    const values = libraryStore
      .games()
      .map((game) => game.userAccuracy)
      .filter((value): value is number => value !== null && Number.isFinite(value));
    if (values.length === 0) return null;
    return values.reduce((sum, value) => sum + value, 0) / values.length;
  });
  return (
    <div class="view-scroll">
      <header class="view-header">
        <div>
          <h1 class="view-title">{t("library.title")}</h1>
          <div class="view-subtitle">
            {pluralGames(libraryStore.total())}, {libraryStore.analyzed()} {t("library.analyzed").toLowerCase()}
          </div>
        </div>
        <div class="header-actions">
          <ImportExport />
        </div>
      </header>

      <div class="stats-strip">
        <div class="stat-card">
          <div class="stat-label">{t("library.totalGames")}</div>
          <div class="stat-value">{libraryStore.total()}</div>
        </div>
        <div class="stat-card">
          <div class="stat-label">{t("library.analyzed")}</div>
          <div class="stat-value">{libraryStore.analyzed()}</div>
        </div>
        <div class="stat-card">
          <div class="stat-label">{t("library.avgAccuracy")}</div>
          <div class="stat-value">
            {averageAccuracy() === null ? "—" : `${formatNumber(averageAccuracy(), 1)}%`}
          </div>
        </div>
        <div class="stat-card">
          <div class="stat-label">{t("library.favorites")}</div>
          <div class="stat-value">{libraryStore.favorites()}</div>
        </div>
      </div>

      <div
        class="library-grid"
        classList={{ "side-collapsed": ui.librarySidebarCollapsed() }}
      >
        <div class="card" style={{ overflow: "hidden" }}>
          <Filters />
          <GameTable />
        </div>

        <div class="side-stack" classList={{ collapsed: ui.librarySidebarCollapsed() }}>
          <div class="side-rail">
            <button
              type="button"
              class="rail-btn"
              title={t("library.expandPanels")}
              aria-label={t("library.expandPanels")}
              onClick={() => expandSidebar()}
            >
              <Icon name="chevrons-left" size={16} />
            </button>
            <div class="rail-divider" />
            <button
              type="button"
              class="rail-btn"
              title={t("library.syncTitle")}
              aria-label={t("library.syncTitle")}
              onClick={() => expandSidebar("sync")}
            >
              <Icon name="sync" size={16} />
              <Show when={libraryStore.syncing()}>
                <span class="rail-badge" />
              </Show>
            </button>
            <button
              type="button"
              class="rail-btn"
              title={t("library.queueTitle")}
              aria-label={t("library.queueTitle")}
              onClick={() => expandSidebar("queue")}
            >
              <Icon name="cpu" size={16} />
              <Show when={libraryStore.batch().running}>
                <span class="rail-badge" />
              </Show>
            </button>
          </div>

          <div class="side-body">
            <div class="side-head">
              <span>{t("library.panels")}</span>
              <button
                type="button"
                class="rail-btn rail-btn-sm"
                title={t("library.collapsePanels")}
                aria-label={t("library.collapsePanels")}
                aria-expanded="true"
                onClick={() => ui.setLibrarySidebarCollapsed(true)}
              >
                <Icon name="chevrons-right" size={15} />
              </button>
            </div>
            <div class="side-section" ref={(el) => (sideSync = el)}>
              <SyncPanel />
            </div>
            <div class="side-section" ref={(el) => (sideQueue = el)}>
              <AnalysisQueue />
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
