import { createSignal, Show } from "solid-js";

import { libraryStore } from "../../stores/libraryStore";
import { settingsStore } from "../../stores/settingsStore";
import { Icon } from "../ui/Icon";

export function SyncPanel() {
  const [username, setUsername] = createSignal("");
  const [force, setForce] = createSignal(false);
  const progress = () => libraryStore.syncProgress();
  const result = () => libraryStore.syncResult();
  return (
    <div class="card">
      <div class="card-header">
        <div>
          <div class="card-title">Chess.com</div>
          <div class="card-subtitle">Публичные партии через PubAPI</div>
        </div>
        <Icon name="sync" size={16} />
      </div>
      <div class="card-pad" style={{ display: "flex", "flex-direction": "column", gap: "9px" }}>
        <input
          class="input"
          placeholder={settingsStore.settings().playerUsername || "ваш ник на Chess.com"}
          value={username()}
          onInput={(event) => setUsername(event.currentTarget.value)}
        />
        <label class="chip" style={{ cursor: "pointer", "justify-content": "center" }}>
          <input
            type="checkbox"
            checked={force()}
            onChange={(event) => setForce(event.currentTarget.checked)}
          />
          Перекачать все архивы
        </label>
        <button
          type="button"
          class="btn btn-primary"
          disabled={libraryStore.syncing()}
          onClick={() => void libraryStore.syncChesscom(username(), force())}
        >
          <Show when={libraryStore.syncing()} fallback={<Icon name="download" size={14} />}>
            <span class="spinner" style={{ width: "13px", height: "13px" }} />
          </Show>
          {libraryStore.syncing() ? "Синхронизация…" : "Синхронизировать"}
        </button>
        <Show when={progress()}>
          {(state) => (
            <div>
              <div class="progress-track">
                <div class="progress-fill" style={{ width: `${state().percent}%` }} />
              </div>
              <div class="sync-log">
                {state().message}
                <Show when={state().archivesTotal > 0}>
                  <span>
                    {" "}
                    · архив {state().archivesDone}/{state().archivesTotal}
                  </span>
                </Show>
              </div>
            </div>
          )}
        </Show>
        <Show when={result()}>
          {(summary) => (
            <div class="sync-log" style={{ color: "var(--accent-bright)" }}>
              Готово: +{summary().gamesImported} партий, пропущено {summary().gamesSkipped}.
            </div>
          )}
        </Show>
        <div class="form-hint">
          Инкрементально: завершённые месяцы не скачиваются повторно, текущий архив обновляется.
          Запросы ограничены 1 в секунду.
        </div>
      </div>
    </div>
  );
}
