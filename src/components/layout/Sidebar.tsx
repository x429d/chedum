import { For, Show } from "solid-js";

import { t } from "../../i18n";
import { accountsStore } from "../../stores/accountsStore";
import { libraryStore } from "../../stores/libraryStore";
import { settingsStore } from "../../stores/settingsStore";
import { ui, type View } from "../../stores/uiStore";
import { Icon, type IconName } from "../ui/Icon";
import { BrandMark } from "./BrandMark";

interface NavConfig {
  view: View;
  labelKey: "nav.library" | "nav.studio" | "nav.analytics" | "nav.accounts" | "nav.settings";
  icon: IconName;
}

const NAV: NavConfig[] = [
  { view: "library", labelKey: "nav.library", icon: "library" },
  { view: "analysis", labelKey: "nav.studio", icon: "analysis" },
  { view: "hub", labelKey: "nav.analytics", icon: "hub" },
  { view: "accounts", labelKey: "nav.accounts", icon: "user" },
  { view: "settings", labelKey: "nav.settings", icon: "settings" },
];

export function Sidebar() {
  const navigate = (view: View): void => {
    if (view === "analysis") {
      const id = libraryStore.selectedGameId() ?? libraryStore.games()[0]?.id;
      if (!id) {
        ui.notify(t("nav.selectGamePrompt"), "info");
        return;
      }
      libraryStore.openGame(id);
      return;
    }
    ui.setView(view);
  };

  const engineOnline = (): boolean =>
    Boolean(settingsStore.engine()?.available) && !settingsStore.engine()?.error;

  return (
    <aside class="sidebar">
      <div class="brand">
        <div class="brand-mark">
          <BrandMark size={21} />
        </div>
        <div class="brand-text">
          <div class="brand-title">Chenal</div>
        </div>
        <Show when={!ui.navSidebarCollapsed()}>
          <button
            type="button"
            class="sidebar-toggle"
            title={t("nav.collapse")}
            aria-label={t("nav.collapse")}
            aria-expanded="true"
            onClick={() => ui.setNavSidebarCollapsed(true)}
          >
            <Icon name="chevrons-left" size={15} />
          </button>
        </Show>
      </div>

      <Show when={ui.navSidebarCollapsed()}>
        <button
          type="button"
          class="sidebar-toggle sidebar-expand"
          title={t("nav.expand")}
          aria-label={t("nav.expand")}
          onClick={() => ui.setNavSidebarCollapsed(false)}
        >
          <Icon name="chevrons-right" size={15} />
        </button>
      </Show>

      <nav class="nav">
        <For each={NAV}>
          {(item) => {
            const label = () => t(item.labelKey);
            return (
              <button
                type="button"
                class="nav-item"
                classList={{ active: ui.view() === item.view }}
                title={label()}
                aria-label={label()}
                aria-current={ui.view() === item.view ? "page" : undefined}
                onClick={() => navigate(item.view)}
              >
                <Icon name={item.icon} />
                <span class="nav-label">{label()}</span>
                <Show when={item.view === "library" && libraryStore.total() > 0}>
                  <span class="nav-badge">{libraryStore.total()}</span>
                </Show>
                <Show when={item.view === "accounts" && accountsStore.accounts().length > 0}>
                  <span class="nav-badge">{accountsStore.accounts().length}</span>
                </Show>
              </button>
            );
          }}
        </For>
      </nav>

      <div class="sidebar-footer">
        <div class="engine-row">
          <span class="engine-dot" classList={{ online: engineOnline() }} />
          <span>{engineOnline() ? t("nav.engineReady") : t("nav.engineNotFound")}</span>
        </div>
      </div>
    </aside>
  );
}
