import { createSignal } from "solid-js";

export type View = "library" | "analysis" | "hub" | "settings";
export type ToastKind = "info" | "success" | "error";

export interface Toast {
  id: number;
  message: string;
  kind: ToastKind;
}

const [view, setView] = createSignal<View>("library");
const [toasts, setToasts] = createSignal<Toast[]>([]);
let sequence = 0;

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
};
