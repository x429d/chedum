import { createSignal } from "solid-js";

export type View = "library" | "analysis" | "hub" | "settings";
export type ToastKind = "info" | "success" | "error";

export interface Toast {
  id: number;
  message: string;
  kind: ToastKind;
}

const LIBRARY_SIDEBAR_KEY = "chenal.librarySidebarCollapsed";
const NAV_SIDEBAR_KEY = "chenal.navSidebarCollapsed";

function readFlag(key: string): boolean {
  try {
    return window.localStorage.getItem(key) === "1";
  } catch {
    return false;
  }
}

function writeFlag(key: string, value: boolean): void {
  try {
    window.localStorage.setItem(key, value ? "1" : "0");
  } catch {
    // Storage can be unavailable; the in-memory preference still works.
  }
}

const [view, setView] = createSignal<View>("library");
const [toasts, setToasts] = createSignal<Toast[]>([]);
const [librarySidebarCollapsed, setLibrarySidebarCollapsedSignal] = createSignal(
  readFlag(LIBRARY_SIDEBAR_KEY),
);
const [navSidebarCollapsed, setNavSidebarCollapsedSignal] = createSignal(readFlag(NAV_SIDEBAR_KEY));
let sequence = 0;

/** Both rails are UI-only preferences, so they live in localStorage instead of the database. */
function setLibrarySidebarCollapsed(collapsed: boolean): void {
  setLibrarySidebarCollapsedSignal(collapsed);
  writeFlag(LIBRARY_SIDEBAR_KEY, collapsed);
}

function setNavSidebarCollapsed(collapsed: boolean): void {
  setNavSidebarCollapsedSignal(collapsed);
  writeFlag(NAV_SIDEBAR_KEY, collapsed);
}

function dismiss(id: number): void {
  setToasts((items) => items.filter((item) => item.id !== id));
}

function notify(message: string, kind: ToastKind = "info"): void {
  const id = ++sequence;
  setToasts((items) => [...items, { id, message, kind }]);
  window.setTimeout(() => dismiss(id), 4800);
}

export const ui = {
  view,
  setView,
  toasts,
  notify,
  dismiss,
  librarySidebarCollapsed,
  setLibrarySidebarCollapsed,
  navSidebarCollapsed,
  setNavSidebarCollapsed,
};
