import { createEffect, ErrorBoundary, onCleanup, onMount, Show } from "solid-js";

import { ViewError } from "./components/layout/ViewError";
import { Sidebar } from "./components/layout/Sidebar";
import { Toasts } from "./components/layout/Toasts";
import { applyDocumentLanguage, currentLanguage } from "./i18n";
import { accountsStore } from "./stores/accountsStore";
import { libraryStore } from "./stores/libraryStore";
import { settingsStore } from "./stores/settingsStore";
import { ui } from "./stores/uiStore";
import { toErrorMessage } from "./utils/format";
import { AccountsView } from "./views/AccountsView";
import { AnalysisView } from "./views/AnalysisView";
import { HubView } from "./views/HubView";
import { LibraryView } from "./views/LibraryView";
import { SettingsView } from "./views/SettingsView";

export default function App() {
  onMount(() => {
    void (async () => {
      await settingsStore.load();
      await accountsStore.load();
      await libraryStore.load();
      await libraryStore.startAutoResume();
    })();
    const onError = (event: ErrorEvent): void => {
      ui.notify(toErrorMessage(event.error ?? event.message), "error");
    };
    const onRejection = (event: PromiseRejectionEvent): void => {
      ui.notify(toErrorMessage(event.reason), "error");
    };
    window.addEventListener("error", onError);
    window.addEventListener("unhandledrejection", onRejection);
    onCleanup(() => {
      window.removeEventListener("error", onError);
      window.removeEventListener("unhandledrejection", onRejection);
    });
  });

  createEffect(() => {
    currentLanguage();
    applyDocumentLanguage();
  });

  return (
    <div class="app-shell" classList={{ "nav-collapsed": ui.navSidebarCollapsed() }}>
      <div class="window-drag-bar" data-tauri-drag-region aria-hidden="true" />
      <Sidebar />
      <main class="main">
        <div class="drag-strip" />
        <ErrorBoundary fallback={(error, reset) => <ViewError error={error} reset={reset} />}>
          <Show when={ui.view() === "library"}>
            <LibraryView />
          </Show>
          <Show when={ui.view() === "analysis"}>
            <AnalysisView />
          </Show>
          <Show when={ui.view() === "hub"}>
            <HubView />
          </Show>
          <Show when={ui.view() === "accounts"}>
            <AccountsView />
          </Show>
          <Show when={ui.view() === "settings"}>
            <SettingsView />
          </Show>
        </ErrorBoundary>
      </main>
      <Toasts />
    </div>
  );
}
