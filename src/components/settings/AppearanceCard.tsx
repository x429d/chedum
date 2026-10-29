import { For } from "solid-js";

import { Icon } from "../ui/Icon";
import { settingsStore } from "../../stores/settingsStore";
import type { BoardTheme } from "../../types/models";
import { PieceSetPicker } from "./PieceSetPicker";

const THEMES: { value: BoardTheme; label: string; className: string }[] = [
  { value: "midnight", label: "Midnight", className: "theme-mini-midnight" },
  { value: "walnut", label: "Walnut", className: "theme-mini-walnut" },
  { value: "slate", label: "Slate", className: "theme-mini-slate" },
];

export function AppearanceCard() {
  return (
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
              </div>
              <Icon name="info" size={16} />
            </div>
            <div class="card-pad">
              <div class="form-hint" style={{ "line-height": "1.7" }}>
                Версия {settingsStore.info()?.version ?? "0.1.0"}. Встроенный словарь ECO:{" "}
                {settingsStore.info()?.ecoOpenings ?? "—"} линий.
              </div>
            </div>
          </div>
    </div>
  );
}
