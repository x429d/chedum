import { Show } from "solid-js";

import { t } from "../../i18n";
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
          <div class="card-title">{t("library.queueTitle")}</div>
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
            {selected() > 0
              ? t("library.queueAnalyzeSelected", { count: selected() })
              : t("library.queueResume", { count: pending() })}
          </button>
          <Show when={selected() > 0 && pending() > 0 && !batch().running}>
            <button
              type="button"
              class="btn btn-sm"
              onClick={() => void libraryStore.analyzePending()}
            >
              {t("library.queueAllNew", { count: pending() })}
            </button>
          </Show>
          <Show when={batch().running}>
            <button
              type="button"
              class="btn btn-danger btn-sm"
              onClick={() => void libraryStore.cancelBatch()}
            >
              <Icon name="pause" size={12} />
              {t("library.queueStop")}
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
                <span>{t("library.elapsed")}</span>
                <strong>{formatDuration(batch().elapsedSec)}</strong>
              </div>
              <div class="queue-metric">
                <span>{t("library.remaining")}</span>
                <strong>
                  {batch().etaSec === null ? "—" : `~${formatDuration(batch().etaSec)}`}
                </strong>
              </div>
              <div class="queue-metric">
                <span>{t("library.done")}</span>
                <strong>{batch().done}</strong>
              </div>
              <div class="queue-metric" classList={{ "is-warn": batch().failed > 0 }}>
                <span>{t("library.errors")}</span>
                <strong>{batch().failed}</strong>
              </div>
            </div>
            <div class="queue-message">{batch().label}</div>
            <Show when={batch().plyTotal > 0}>
              <div class="queue-submessage">
                {batch().ply} / {batch().plyTotal}
                {perGame() === null ? "" : `, ~${formatDuration(perGame())}`}
              </div>
            </Show>
          </div>
        </Show>

        <div class="form-hint">
          {pending() > 0
            ? t("library.queuePending", { count: pending() })
            : t("library.queueAllDone")}
          {libraryStore.queueExcluded() > 0
            ? ` ${t("library.queueExcluded", { count: libraryStore.queueExcluded() })}`
            : ""}
        </div>
      </div>
    </div>
  );
}
