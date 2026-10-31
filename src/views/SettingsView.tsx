import { AnalysisCard } from "../components/settings/AnalysisCard";
import { AppearanceCard } from "../components/settings/AppearanceCard";
import { EngineCard } from "../components/settings/EngineCard";
import { Icon } from "../components/ui/Icon";
import { t } from "../i18n";
import { accountsStore } from "../stores/accountsStore";
import { ui } from "../stores/uiStore";

export function SettingsView() {
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
                <div class="card-title">{t("accounts.listTitle")}</div>
              </div>
              <Icon name="user" size={16} />
            </div>
            <div class="card-pad" style={{ display: "flex", "flex-direction": "column", gap: "10px" }}>
              <div class="form-hint">{t("accounts.subtitle")}</div>
              <button
                type="button"
                class="btn"
                style={{ "align-self": "flex-start" }}
                onClick={() => ui.setView("accounts")}
              >
                <Icon name="user" size={14} />
                {t("accounts.manage")}
                {accountsStore.accounts().length > 0
                  ? ` (${accountsStore.accounts().length})`
                  : ""}
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
