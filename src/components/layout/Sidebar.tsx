import { For, Show } from "solid-js";

import { libraryStore } from "../../stores/libraryStore";
import { settingsStore } from "../../stores/settingsStore";
import { ui, type View } from "../../stores/uiStore";
import { Icon, type IconName } from "../ui/Icon";

interface NavItem {
  view: View;
  label: string;
  icon: IconName;
}

const NAV: NavItem[] = [
  { view: "library", label: "Библиотека", icon: "library" },
  { view: "analysis", label: "Студия", icon: "analysis" },
  { view: "hub", label: "Аналитика", icon: "hub" },
  { view: "settings", label: "Настройки", icon: "settings" },
];

export function Sidebar() {
  const navigate = (view: View): void => {
    if (view === "analysis") {
      const id = libraryStore.selectedGameId() ?? libraryStore.games()[0]?.id;
      if (!id) {
        ui.notify("Сначала импортируйте или выберите партию", "info");
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
          <Icon name="board" size={19} />
        </div>
        <div>
          <div class="brand-title">Chenal</div>
          <div class="brand-subtitle">Chess Studio</div>
        </div>
      </div>

      <nav class="nav">
        <For each={NAV}>
          {(item) => (
            <button
              type="button"
              class="nav-item"
              classList={{ active: ui.view() === item.view }}
              onClick={() => navigate(item.view)}
            >
              <Icon name={item.icon} />
              <span>{item.label}</span>
              <Show when={item.view === "library" && libraryStore.total() > 0}>
                <span class="nav-badge">{libraryStore.total()}</span>
              </Show>
            </button>
          )}
        </For>
      </nav>

      <div class="sidebar-footer">
        <div class="engine-row">
          <span class="engine-dot" classList={{ online: engineOnline() }} />
          <span>{engineOnline() ? "Stockfish готов" : "Stockfish не найден"}</span>
        </div>
        <div class="mono" style={{ "font-size": "9.5px", color: "var(--dim)" }}>
          {settingsStore.engine()?.name ?? "local engine"} · v
          {settingsStore.info()?.version ?? "0.1.0"}
        </div>
        <div style={{ color: "var(--dim)", "font-size": "10px", "line-height": "1.4" }}>
          Local-first. Партии и анализ остаются на этом Mac.
        </div>
      </div>
    </aside>
  );
}
