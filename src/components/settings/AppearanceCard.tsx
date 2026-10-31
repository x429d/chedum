import { For } from "solid-js";

import { currentLanguage, SUPPORTED_LANGUAGES, t } from "../../i18n";
import { settingsStore } from "../../stores/settingsStore";
import type { BoardTheme, Language } from "../../types/models";
import { Icon } from "../ui/Icon";
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
            <div class="card-title">{t("settings.language")}</div>
          </div>
        </div>
        <div class="card-pad">
          <div class="segmented">
            <For each={SUPPORTED_LANGUAGES}>
              {(lang) => (
                <button
                  type="button"
                  aria-pressed={currentLanguage() === lang.code}
                  classList={{ active: currentLanguage() === lang.code }}
                  onClick={() => void settingsStore.setLanguage(lang.code as Language)}
                >
                  {lang.name}
                </button>
              )}
            </For>
          </div>
        </div>
      </div>

      <div class="card">
        <div class="card-header">
          <div>
            <div class="card-title">{t("settings.appearanceTitle")}</div>
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
                  aria-pressed={settingsStore.settings().boardTheme === theme.value}
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
                {t("settings.showArrows")}
              </div>
            </div>
            <button
              type="button"
              class="switch"
              role="switch"
              aria-checked={settingsStore.settings().showArrows}
              aria-label={t("settings.showArrows")}
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
    </div>
  );
}
