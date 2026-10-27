import { createEffect, createSignal, For, on, Show } from "solid-js";

import { PieceSetPicker } from "../components/settings/PieceSetPicker";
import { Icon } from "../components/ui/Icon";
import { settingsStore } from "../stores/settingsStore";
import { ui } from "../stores/uiStore";
import type { BoardTheme } from "../types/models";

const THEMES: { value: BoardTheme; label: string; className: string }[] = [
  { value: "midnight", label: "Midnight", className: "theme-mini-midnight" },
  { value: "walnut", label: "Walnut", className: "theme-mini-walnut" },
  { value: "slate", label: "Slate", className: "theme-mini-slate" },
];

const DEPTHS = [8, 10, 12, 14, 16, 18, 20, 22];

export function SettingsView() {
  const [username, setUsername] = createSignal(settingsStore.settings().playerUsername);

  createEffect(
    on(
      () => settingsStore.settings().playerUsername,
      (value) => setUsername(value),
    ),
  );

  return (
    <div class="view-scroll">
      <header class="view-header">
        <div>
          <div class="view-kicker">Preferences</div>
          <h1 class="view-title">Настройки</h1>
          <div class="view-subtitle">
            Профиль, локальный движок и оформление студии. Всё сохраняется на этом Mac.
          </div>
        </div>
      </header>

      <div class="settings-grid">
        <div style={{ display: "flex", "flex-direction": "column", gap: "16px" }}>
          <div class="card">
            <div class="card-header">
              <div>
                <div class="card-title">Профиль игрока</div>
                <div class="card-subtitle">Ник используется для синхронизации и цвета</div>
              </div>
              <Icon name="user" size={16} />
            </div>
            <div class="card-pad">
              <div class="form-row">
                <label class="form-label" for="player-username">
                  Ник на Chess.com
                </label>
                <input
                  id="player-username"
                  class="input"
                  value={username()}
                  placeholder="например, chenal_player"
                  onInput={(event) => setUsername(event.currentTarget.value)}
                />
                <div class="form-hint">
                  По этому нику определяется ваш цвет в импортированных PGN и выполняется
                  синхронизация публичных архивов.
                </div>
              </div>
              <button
                type="button"
                class="btn btn-primary"
                onClick={() => {
                  void settingsStore
                    .update({ playerUsername: username().trim() })
                    .then((saved) => {
                      if (saved) ui.notify("Профиль сохранён", "success");
                    });
                }}
              >
                <Icon name="check" size={14} />
                Сохранить профиль
              </button>
            </div>
          </div>

          <div class="card">
            <div class="card-header">
              <div>
                <div class="card-title">Локальный анализ</div>
                <div class="card-subtitle">Stockfish запускается как дочерний процесс</div>
              </div>
              <Icon name="cpu" size={16} />
            </div>
            <div class="card-pad">
              <div class="form-row">
                <label class="form-label" for="analysis-depth">
                  Глубина анализа
                </label>
                <select
                  id="analysis-depth"
                  class="select"
                  value={String(settingsStore.settings().analysisDepth)}
                  onChange={(event) =>
                    void settingsStore.update({
                      analysisDepth: Number(event.currentTarget.value),
                    })
                  }
                >
                  <For each={DEPTHS}>
                    {(depth) => <option value={String(depth)}>{depth}</option>}
                  </For>
                </select>
                <div class="form-hint">
                  Больше глубина — точнее классификация, но дольше разбор. 14 — хороший баланс для
                  рапида и блица.
                </div>
              </div>
              <div class="form-row">
                <span class="form-label">MultiPV</span>
                <div class="segmented">
                  <For each={[1, 2, 3]}>
                    {(value) => (
                      <button
                        type="button"
                        classList={{ active: settingsStore.settings().multipv === value }}
                        onClick={() => void settingsStore.update({ multipv: value })}
                      >
                        {value}
                      </button>
                    )}
                  </For>
                </div>
                <div class="form-hint">
                  Две линии нужны для поиска критических позиций и Brilliant/Great. Три — для
                  более богатого дерева вариантов.
                </div>
              </div>
              <div class="toggle-row">
                <div>
                  <div style={{ "font-size": "12px", "font-weight": "600" }}>
                    Автоанализ после импорта
                  </div>
                  <div class="form-hint">Ставить новые партии в очередь автоматически.</div>
                </div>
                <button
                  type="button"
                  class="switch"
                  classList={{ on: settingsStore.settings().autoAnalyze }}
                  onClick={() =>
                    void settingsStore.update({
                      autoAnalyze: !settingsStore.settings().autoAnalyze,
                    })
                  }
                />
              </div>
            </div>
          </div>

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
                  <div class="sync-log" style={{ color: "#f87171" }}>
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
        </div>

        <div style={{ display: "flex", "flex-direction": "column", gap: "16px" }}>
          <div class="card">
            <div class="card-header">
              <div>
                <div class="card-title">Оформление</div>
                <div class="card-subtitle">Тема доски и стрелки</div>
              </div>
              <Icon name="board" size={16} />
            </div>
            <div class="card-pad">
              <div class="theme-preview-row">
                <For each={THEMES}>
                  {(theme) => (
                    <button
                      type="button"
                      class="theme-preview"
                      classList={{
                        selected: settingsStore.settings().boardTheme === theme.value,
                      }}
                      onClick={() => void settingsStore.setBoardTheme(theme.value)}
                    >
                      <div class={`theme-mini-board ${theme.className}`}>
                        <For each={Array.from({ length: 16 }, (_, index) => index)}>
                          {(index) => (
                            <span
                              classList={{
                                light: (Math.floor(index / 4) + (index % 4)) % 2 === 0,
                              }}
                            />
                          )}
                        </For>
                      </div>
                      {theme.label}
                    </button>
                  )}
                </For>
              </div>
              <div class="toggle-row" style={{ "margin-top": "8px" }}>
                <div>
                  <div style={{ "font-size": "12px", "font-weight": "600" }}>
                    Стрелки лучших линий
                  </div>
                  <div class="form-hint">MultiPV 1, MultiPV 2, ошибка и угроза.</div>
                </div>
                <button
                  type="button"
                  class="switch"
                  classList={{ on: settingsStore.settings().showArrows }}
                  onClick={() =>
                    void settingsStore.update({
                      showArrows: !settingsStore.settings().showArrows,
                    })
                  }
                />
              </div>
            </div>
          </div>

          <PieceSetPicker />

          <div class="card">
            <div class="card-header">
              <div>
                <div class="card-title">О приложении</div>
                <div class="card-subtitle">Chenal · native chess analytics</div>
              </div>
              <Icon name="info" size={16} />
            </div>
            <div class="card-pad">
              <div class="form-hint" style={{ "line-height": "1.7" }}>
                Версия {settingsStore.info()?.version ?? "0.1.0"} · встроенный словарь ECO:{" "}
                {settingsStore.info()?.ecoOpenings ?? "—"} линий.
                <br />
                Local-first: партии, оценки и заметки хранятся в SQLite (WAL). Stockfish работает
                офлайн, интернет нужен только для Chess.com PubAPI.
              </div>
              <div style={{ display: "flex", gap: "6px", "flex-wrap": "wrap", "margin-top": "10px" }}>
                <span class="chip accent">Stockfish</span>
                <span class="chip">SQLite WAL</span>
                <span class="chip">SolidJS</span>
                <span class="chip">Tauri v2</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
