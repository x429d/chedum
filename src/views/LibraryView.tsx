import { createMemo, onCleanup } from "solid-js";

import { libraryStore } from "../stores/libraryStore";
import { formatNumber, pluralRu } from "../utils/format";
import { Icon } from "../components/ui/Icon";
import { AnalysisQueue } from "../components/library/AnalysisQueue";
import { Filters } from "../components/library/Filters";
import { GameTable } from "../components/library/GameTable";
import { ImportExport } from "../components/library/ImportExport";
import { SyncPanel } from "../components/library/SyncPanel";

export function LibraryView() {
  onCleanup(() => libraryStore.cancelPendingSearch());
  const analyzed = createMemo(() => libraryStore.games().filter((game) => game.hasAnalysis).length);
  const favorites = createMemo(() => libraryStore.games().filter((game) => game.isFavorite).length);
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
          <div class="view-kicker">Local library</div>
          <h1 class="view-title">Библиотека партий</h1>
          <div class="view-subtitle">
            {libraryStore.total()}{" "}
            {pluralRu(libraryStore.total(), "партия", "партии", "партий")} · {analyzed()} с
            анализом · всё хранится локально
          </div>
        </div>
        <div class="header-actions">
          <ImportExport />
        </div>
      </header>

      <div class="stats-strip">
        <div class="stat-card">
          <div class="stat-label">Всего партий</div>
          <div class="stat-value">{libraryStore.total()}</div>
          <div class="stat-hint">в локальной SQLite</div>
        </div>
        <div class="stat-card">
          <div class="stat-label">С анализом</div>
          <div class="stat-value">{analyzed()}</div>
          <div class="stat-hint">Stockfish + классификатор</div>
        </div>
        <div class="stat-card">
          <div class="stat-label">Средняя точность</div>
          <div class="stat-value">
            {averageAccuracy() === null ? "—" : `${formatNumber(averageAccuracy(), 1)}%`}
          </div>
          <div class="stat-hint">по видимой выборке</div>
        </div>
        <div class="stat-card">
          <div class="stat-label">Избранные</div>
          <div class="stat-value">{favorites()}</div>
          <div class="stat-hint">отмечены звездой</div>
        </div>
      </div>

      <div class="library-grid">
        <div class="card" style={{ overflow: "hidden" }}>
          <Filters />
          <GameTable />
        </div>

        <div class="side-stack">
          <SyncPanel />
          <AnalysisQueue />
          <div class="card card-pad">
            <div style={{ display: "flex", "align-items": "center", gap: "8px" }}>
              <div class="empty-icon" style={{ width: "32px", height: "32px", "margin": "0" }}>
                <Icon name="sparkles" size={15} />
              </div>
              <div>
                <div class="card-title">Глубокий разбор</div>
                <div class="card-subtitle">11 категорий · Win% · CAPS · Move Coach</div>
              </div>
            </div>
            <div class="form-hint" style={{ "margin-top": "9px" }}>
              Откройте партию, чтобы увидеть двухслойный график, кольца точности, объяснения
              тренера и тренажёр «Retry Mistake».
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
