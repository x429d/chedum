import { Show } from "solid-js";

import { Icon } from "../ui/Icon";
import { settingsStore } from "../../stores/settingsStore";

export function EngineCard() {
  return (
          <div class="card">
            <div class="card-header">
              <div>
                <div class="card-title">Движок</div>
                <div class="card-subtitle">Диагностика Stockfish</div>
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
                  {settingsStore.engine()?.name ?? "Stockfish не инициализирован"}
                </span>
                <button
                  type="button"
                  class="btn btn-sm"
                  style={{ "margin-left": "auto" }}
                  onClick={() => void settingsStore.refreshEngine()}
                >
                  <Icon name="refresh" size={12} />
                  Проверить
                </button>
              </div>
              <div class="form-hint mono" style={{ "word-break": "break-all" }}>
                {settingsStore.engine()?.path ?? "путь не найден"}
              </div>
              <Show when={settingsStore.engine()?.error}>
                {(error) => (
                  <div class="sync-log" style={{ color: "var(--danger)" }}>
                    {error()}
                  </div>
                )}
              </Show>
              <div class="form-hint" style={{ "margin-top": "8px" }}>
                Бинарник поставляется вместе с приложением. Если его нет, установите Stockfish
                через Homebrew — приложение найдёт его автоматически.
              </div>
            </div>
          </div>
  );
}
