import { Show } from "solid-js";

import { libraryStore } from "../../stores/libraryStore";
import { settingsStore } from "../../stores/settingsStore";
import { formatDuration } from "../../utils/format";
import { Icon } from "../ui/Icon";

export function AnalysisQueue() {
  const batch = () => libraryStore.batch();
  const pending = () => libraryStore.pendingAnalysis();
  const selected = () => libraryStore.selectedCount();
  const percent = () => Math.round(batch().percent);
  const processed = () => batch().done + batch().failed;
  const perGame = () => (processed() > 0 ? batch().elapsedSec / processed() : null);
  return (
    <div class="card queue-card">
      <div class="card-header">
        <div>
          <div class="card-title">Очередь анализа</div>
          <div class="card-subtitle">
            depth {settingsStore.settings().analysisDepth}, MultiPV{" "}
            {settingsStore.settings().multipv}
          </div>
        </div>
        <Icon name="cpu" size={16} />
      </div>
      <div class="card-pad">
        <div class="queue-actions">
          <button
            type="button"
            class="btn btn-primary btn-sm"
            disabled={batch().running || (selected() === 0 && pending() === 0)}
            onClick={() => libraryStore.analyzeSelected()}
          >
            <Icon name="analyze" size={12} />
            {selected() > 0 ? `Анализ (${selected()})` : `Продолжить разбор (${pending()})`}
          </button>
          <Show when={selected() > 0 && pending() > 0 && !batch().running}>
            <button
              type="button"
              class="btn btn-sm"
              onClick={() => void libraryStore.analyzePending()}
            >
              Все новые ({pending()})
            </button>
          </Show>
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
          <div class="queue-progress">
            <div class="queue-progress-head">
              <span class="queue-percent">{percent()}%</span>
              <span class="queue-counter">
                {Math.min(processed(), batch().total)} / {batch().total}
              </span>
            </div>
            <div class="progress-track">
              <div class="progress-fill" style={{ width: `${batch().percent}%` }} />
            </div>
            <div class="queue-metrics">
              <div class="queue-metric">
                <span>Прошло</span>
                <strong>{formatDuration(batch().elapsedSec)}</strong>
              </div>
              <div class="queue-metric">
                <span>Осталось</span>
                <strong>
                  {batch().etaSec === null ? "—" : `~${formatDuration(batch().etaSec)}`}
                </strong>
              </div>
              <div class="queue-metric">
                <span>Готово</span>
                <strong>{batch().done}</strong>
              </div>
              <div class="queue-metric" classList={{ "is-warn": batch().failed > 0 }}>
                <span>Ошибок</span>
                <strong>{batch().failed}</strong>
              </div>
            </div>
            <div class="queue-message">{batch().label}</div>
            <Show when={batch().plyTotal > 0}>
              <div class="queue-submessage">
                позиций: {batch().ply} из {batch().plyTotal}
                {perGame() === null ? "" : `, ~${formatDuration(perGame())} на партию`}
              </div>
            </Show>
          </div>
        </Show>

        <div class="form-hint">
          {batch().running
            ? "Партии идут параллельно; остановка завершает текущие и не берёт новые."
            : pending() > 0
              ? `Без анализа текущей конфигурации: ${pending()} по фильтру.`
              : "Все партии текущей конфигурации разобраны."}
          {libraryStore.queueExcluded() > 0
            ? ` Исключено после 3 ошибок: ${libraryStore.queueExcluded()}.`
            : ""}
        </div>
      </div>
    </div>
  );
}
