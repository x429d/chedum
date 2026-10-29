import { onMount, Show } from "solid-js";

import { Sidebar } from "./components/layout/Sidebar";
import { Toasts } from "./components/layout/Toasts";
import { libraryStore } from "./stores/libraryStore";
import { settingsStore } from "./stores/settingsStore";
import { ui } from "./stores/uiStore";
import { AnalysisView } from "./views/AnalysisView";
import { HubView } from "./views/HubView";
import { LibraryView } from "./views/LibraryView";
import { SettingsView } from "./views/SettingsView";

export default function App() {
  onMount(() => {
    void (async () => {
      await settingsStore.load();
      await libraryStore.load();
      await libraryStore.startAutoResume();
    })();
  });

  return (
    <div class="app-shell" classList={{ "nav-collapsed": ui.navSidebarCollapsed() }}>
      <div class="window-drag-bar" data-tauri-drag-region aria-hidden="true" />
      <Sidebar />
      <main class="main">
        <div class="drag-strip" />
        <Show when={ui.view() === "library"}>
          <LibraryView />
        </Show>
        <Show when={ui.view() === "analysis"}>
          <AnalysisView />
        </Show>
        <Show when={ui.view() === "hub"}>
          <HubView />
        </Show>
        <Show when={ui.view() === "settings"}>
          <SettingsView />
        </Show>
      </main>
      <Toasts />
    </div>
  );
}
