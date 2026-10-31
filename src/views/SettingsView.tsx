import { createEffect, createSignal, on } from "solid-js";

import { AnalysisCard } from "../components/settings/AnalysisCard";
import { AppearanceCard } from "../components/settings/AppearanceCard";
import { EngineCard } from "../components/settings/EngineCard";
import { Icon } from "../components/ui/Icon";
import { t } from "../i18n";
import { settingsStore } from "../stores/settingsStore";
import { ui } from "../stores/uiStore";

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
          <h1 class="view-title">{t("settings.title")}</h1>
        </div>
      </header>

      <div class="settings-grid">
        <div style={{ display: "flex", "flex-direction": "column", gap: "16px" }}>
          <div class="card">
            <div class="card-header">
              <div>
                <div class="card-title">{t("settings.profileTitle")}</div>
              </div>
              <Icon name="user" size={16} />
            </div>
            <div class="card-pad">
              <div class="form-row">
                <label class="form-label" for="player-username">
                  {t("settings.usernameLabel")}
                </label>
                <input
                  id="player-username"
                  class="input"
                  value={username()}
                  placeholder={t("settings.usernamePlaceholder")}
                  onInput={(event) => setUsername(event.currentTarget.value)}
                />
              </div>
              <button
                type="button"
                class="btn btn-primary"
                onClick={() => {
                  void settingsStore
                    .update({ playerUsername: username().trim() })
                    .then((saved) => {
                      if (saved) ui.notify(t("settings.profileSaved"), "success");
                    });
                }}
              >
                <Icon name="check" size={14} />
                {t("settings.saveProfile")}
              </button>
            </div>
          </div>
          <AnalysisCard />
          <EngineCard />
        </div>

        <AppearanceCard />
      </div>
    </div>
  );
}
