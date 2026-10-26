import { For, Show } from "solid-js";

import { ui } from "../../stores/uiStore";
import { Icon } from "../ui/Icon";

export function Toasts() {
  return (
    <Show when={ui.toasts().length > 0}>
      <div class="toast-stack">
        <For each={ui.toasts()}>
          {(toast) => (
            <div class="toast" classList={{ [toast.kind]: true }} onClick={() => ui.dismiss(toast.id)}>
              <Icon
                name={toast.kind === "error" ? "alert" : toast.kind === "success" ? "check" : "info"}
                size={15}
              />
              <span>{toast.message}</span>
              <button
                type="button"
                class="btn btn-ghost icon-btn"
                style={{ width: "20px", height: "20px", "margin-left": "auto" }}
                onClick={(event) => {
                  event.stopPropagation();
                  ui.dismiss(toast.id);
                }}
              >
                <Icon name="close" size={12} />
              </button>
            </div>
          )}
        </For>
      </div>
    </Show>
  );
}
