import { Show } from "solid-js";

import { t } from "../../i18n";
import { settingsStore } from "../../stores/settingsStore";
import { Icon } from "../ui/Icon";

export function EngineCard() {
  return (
    <div class="card">
      <div class="card-header">
        <div>
          <div class="card-title">{t("settings.engineTitle")}</div>
        </div>
        <Icon name="activity" size={16} />
      </div>
      <div class="card-pad">
        <div class="engine-row" style={{ "margin-bottom": "8px" }}>
          <span
            class="engine-dot"
            classList={{
              online:
                Boolean(settingsStore.engine()?.available) && !settingsStore.engine()?.error,
            }}
          />
          <span style={{ "font-size": "12px", "font-weight": "600" }}>
            {settingsStore.engine()?.name ?? t("settings.engineNotInit")}
          </span>
          <button
            type="button"
            class="btn btn-sm"
            style={{ "margin-left": "auto" }}
            onClick={() => void settingsStore.refreshEngine()}
          >
            <Icon name="refresh" size={12} />
            {t("settings.checkEngine")}
          </button>
        </div>
        <div class="form-hint mono" style={{ "word-break": "break-all" }}>
          {settingsStore.engine()?.path ?? t("settings.pathNotFound")}
        </div>
        <Show when={settingsStore.engine()?.error}>
          {(error) => (
            <div class="sync-log" style={{ color: "var(--danger)" }}>
              {error()}
            </div>
          )}
        </Show>
      </div>
    </div>
  );
}
