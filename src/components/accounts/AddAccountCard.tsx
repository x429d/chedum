import { createSignal, For, Show } from "solid-js";

import { t } from "../../i18n";
import { accountsStore } from "../../stores/accountsStore";
import type { AccountPlatform } from "../../types/models";
import { Icon } from "../ui/Icon";

const PLATFORMS: AccountPlatform[] = ["chesscom", "lichess"];

export function AddAccountCard() {
  const [platform, setPlatform] = createSignal<AccountPlatform>("chesscom");
  const [username, setUsername] = createSignal("");
  const [submitting, setSubmitting] = createSignal(false);

  const submit = async (): Promise<void> => {
    const value = username().trim();
    if (!value || submitting() || accountsStore.busy()) return;
    setSubmitting(true);
    const added = await accountsStore.add(platform(), value);
    setSubmitting(false);
    if (added) setUsername("");
  };

  return (
    <div class="card">
      <div class="card-header">
        <div>
          <div class="card-title">{t("accounts.addTitle")}</div>
        </div>
        <Icon name="plus" size={16} />
      </div>
      <div class="card-pad account-add">
        <div class="form-row">
          <span class="form-label">{t("accounts.platform")}</span>
          <div class="segmented">
            <For each={PLATFORMS}>
              {(value) => (
                <button
                  type="button"
                  classList={{ active: platform() === value }}
                  onClick={() => setPlatform(value)}
                >
                  {t(value === "chesscom" ? "accounts.platformChesscom" : "accounts.platformLichess")}
                </button>
              )}
            </For>
          </div>
        </div>
        <div class="form-row">
          <label class="form-label" for="account-username">
            {t("accounts.usernamePlaceholder")}
          </label>
          <div class="account-add-row">
            <input
              id="account-username"
              class="input"
              value={username()}
              placeholder={t("accounts.usernamePlaceholder")}
              autocomplete="off"
              autocapitalize="off"
              spellcheck={false}
              onInput={(event) => setUsername(event.currentTarget.value)}
              onKeyDown={(event) => {
                if (event.key === "Enter") void submit();
              }}
            />
            <button
              type="button"
              class="btn btn-primary"
              disabled={submitting() || accountsStore.busy() || username().trim().length === 0}
              onClick={() => void submit()}
            >
              <Show when={submitting()} fallback={<Icon name="plus" size={14} />}>
                <span class="spinner" style={{ width: "13px", height: "13px" }} />
              </Show>
              {submitting() ? t("accounts.adding") : t("accounts.addBtn")}
            </button>
          </div>
          <div class="form-hint">{t("accounts.howStep2")}</div>
        </div>
      </div>
    </div>
  );
}
