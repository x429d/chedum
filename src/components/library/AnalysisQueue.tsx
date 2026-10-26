import { Show } from "solid-js";

import { libraryStore } from "../../stores/libraryStore";
import { settingsStore } from "../../stores/settingsStore";
import { Icon } from "../ui/Icon";

export function AnalysisQueue() {
  const batch = () => libraryStore.batch();
  const pending = () => libraryStore.games().filter((game) => !game.hasAnalysis).length;
  return (
    <div class="card queue-card">
      <div class="card-header">
        <div>
          <div class="card-title">Очередь анализа</div>
          <div class="card-subtitle">
            depth {settingsStore.settings().analysisDepth} · MultiPV{" "}
            {settingsStore.settings().multipv}
          </div>
        </div>
        <Icon name="cpu" size={16} />
      </div>
      <div class="card-pad">
        <div style={{ display: "flex", gap: "6px", "flex-wrap": "wrap" }}>
          <button
            type="button"
            class="btn btn-primary btn-sm"
            disabled={batch().running}
            onClick={() => libraryStore.analyzeSelected()}
          >
            <Icon name="sparkles" size={12} />
            {libraryStore.selectedCount() > 0
              ? `Анализ (${libraryStore.selectedCount()})`
              : "Анализ новых"}
          </button>
          <Show when={batch().running}>
            <button
              type="button"
              class="btn btn-danger btn-sm"
              onClick={() => void libraryStore.cancelBatch()}
            >
              <Icon name="pause" size={12} />
              Стоп
            </button>
          </Show>
        </div>
        <Show when={batch().running || batch().percent > 0}>
          <div>
            <div class="progress-track">
              <div class="progress-fill" style={{ width: `${batch().percent}%` }} />
            </div>
            <div class="sync-log">{batch().label}</div>
          </div>
        </Show>
        <div class="form-hint">
          {pending() > 0
            ? `Без анализа: ${pending()} партий. Очередь выполняется последовательно, по одной партии.`
            : "Все партии в текущей выборке проанализированы."}
        </div>
      </div>
    </div>
  );
}
