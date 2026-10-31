import { createSignal, For, Show } from "solid-js";

import { t } from "../../i18n";
import { accountsStore } from "../../stores/accountsStore";
import { ui } from "../../stores/uiStore";
import { syncProgressLabel } from "../../utils/progress";
import { Icon } from "../ui/Icon";

export function SyncPanel() {
  const [force, setForce] = createSignal(false);
  const progressAccount = () => {
    const id = accountsStore.syncingAccountId();
    const account = accountsStore.accounts().find((value) => value.id === id);
    return account ? accountsStore.accountName(account) : null;
  };

  return (
    <div class="card">
      <div class="card-header">
        <div>
          <div class="card-title">{t("accounts.title")}</div>
        </div>
        <Icon name="user" size={16} />
      </div>
      <div class="card-pad" style={{ display: "flex", "flex-direction": "column", gap: "9px" }}>
        <Show
          when={accountsStore.accounts().length > 0}
          fallback={
            <>
              <div class="form-hint">{t("accounts.emptyHint")}</div>
              <button
                type="button"
                class="btn btn-primary btn-sm"
                onClick={() => ui.setView("accounts")}
              >
                <Icon name="plus" size={13} />
                {t("accounts.addBtn")}
              </button>
            </>
          }
        >
          <div class="sync-account-list">
            <For each={accountsStore.accounts()}>
              {(account) => (
                <div class="sync-account-row">
                  <span
                    class="sync-account-dot"
                    title={accountsStore.platformName(account.platform)}
                    classList={{
                      active: accountsStore.activeAccountId() === account.id,
                      lichess: account.platform === "lichess",
                    }}
                  />
                  <span
                    class="sync-account-name"
                    title={`${accountsStore.platformName(account.platform)} · @${account.username}`}
                  >
                    @{account.username}
                  </span>
                  <span class="sync-account-count mono">{account.games}</span>
                  <button
                    type="button"
                    class="btn btn-ghost btn-sm icon-btn"
                    title={t("accounts.syncBtn")}
                    aria-label={t("accounts.syncBtn")}
                    disabled={accountsStore.busy()}
                    onClick={() => void accountsStore.syncOne(account.id, force())}
                  >
                    <Show
                      when={accountsStore.syncingAccountId() === account.id}
                      fallback={<Icon name="sync" size={13} />}
                    >
                      <span class="spinner" style={{ width: "11px", height: "11px" }} />
                    </Show>
                  </button>
                </div>
              )}
            </For>
          </div>
          <label class="chip" style={{ cursor: "pointer", "justify-content": "center" }} title={t("accounts.syncForceHint")}>
            <input
              type="checkbox"
              checked={force()}
              onChange={(event) => setForce(event.currentTarget.checked)}
            />
            {t("accounts.syncForce")}
          </label>
          <div style={{ display: "flex", gap: "6px" }}>
            <button
              type="button"
              class="btn btn-sm"
              style={{ flex: "1" }}
              disabled={accountsStore.busy()}
              onClick={() => void accountsStore.syncAll(force())}
            >
              <Show
                when={accountsStore.syncingAll()}
                fallback={<Icon name="download" size={13} />}
              >
                <span class="spinner" style={{ width: "11px", height: "11px" }} />
              </Show>
              {accountsStore.syncingAll() ? t("accounts.syncAllBusy") : t("accounts.syncAll")}
            </button>
            <Show when={accountsStore.busy()}>
              <button
                type="button"
                class="btn btn-danger btn-sm"
                onClick={() => void accountsStore.cancel()}
              >
                {t("accounts.cancelSync")}
              </button>
            </Show>
            <button
              type="button"
              class="btn btn-ghost btn-sm"
              title={t("accounts.manage")}
              aria-label={t("accounts.manage")}
              onClick={() => ui.setView("accounts")}
            >
              <Icon name="settings" size={14} />
            </button>
          </div>
          <Show when={accountsStore.progress()}>
            {(progress) => (
              <div>
                <div class="progress-track">
                  <div class="progress-fill" style={{ width: `${progress().percent}%` }} />
                </div>
                <div class="sync-log">
                  <Show when={progressAccount()}>
                    {(account) => <span>{account()}: </span>}
                  </Show>
                  {syncProgressLabel(progress())}
                </div>
              </div>
            )}
          </Show>
        </Show>
      </div>
    </div>
  );
}
