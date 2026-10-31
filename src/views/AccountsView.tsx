import { createSignal, For, onMount, Show } from "solid-js";

import { t } from "../i18n";
import { accountsStore } from "../stores/accountsStore";
import { ui } from "../stores/uiStore";
import { syncProgressLabel } from "../utils/progress";
import { AccountCard } from "../components/accounts/AccountCard";
import { AddAccountCard } from "../components/accounts/AddAccountCard";
import { Icon } from "../components/ui/Icon";

export function AccountsView() {
  const [force, setForce] = createSignal(false);
  const lastResultLabel = (): string | null => {
    const result = accountsStore.lastResult();
    if (!result) return null;
    return t("accounts.syncResult", {
      account: `${accountsStore.platformName(result.platform)} @${result.username}`,
      imported: result.gamesImported,
      duplicates: result.duplicates,
      filtered: result.filtered,
      errors: result.errors.length,
    });
  };
  const progressAccount = () => {
    const id = accountsStore.syncingAccountId();
    const account = accountsStore.accounts().find((value) => value.id === id);
    return account ? accountsStore.accountName(account) : null;
  };

  onMount(() => {
    void accountsStore.refresh();
  });

  return (
    <div class="view-scroll">
      <header class="view-header">
        <div>
          <h1 class="view-title">{t("accounts.title")}</h1>
          <div class="view-subtitle">{t("accounts.subtitle")}</div>
        </div>
        <div class="header-actions">
          <label class="chip" style={{ cursor: "pointer" }}>
            <input
              type="checkbox"
              checked={force()}
              onChange={(event) => setForce(event.currentTarget.checked)}
            />
            {t("accounts.syncForce")}
          </label>
          <button
            type="button"
            class="btn"
            disabled={accountsStore.accounts().length === 0}
            onClick={() => {
              void accountsStore.setActive(null).then(() => ui.setView("library"));
            }}
          >
            <Icon name="library" size={14} />
            {t("accounts.allAccounts")}
          </button>
          <button
            type="button"
            class="btn btn-primary"
            disabled={accountsStore.busy() || accountsStore.accounts().length === 0}
            onClick={() => void accountsStore.syncAll(force())}
          >
            <Show
              when={accountsStore.syncingAll()}
              fallback={<Icon name="sync" size={14} />}
            >
              <span class="spinner" style={{ width: "13px", height: "13px" }} />
            </Show>
            {accountsStore.syncingAll() ? t("accounts.syncAllBusy") : t("accounts.syncAll")}
          </button>
          <Show when={accountsStore.busy()}>
            <button
              type="button"
              class="btn btn-danger"
              onClick={() => void accountsStore.cancel()}
            >
              {t("accounts.cancelSync")}
            </button>
          </Show>
        </div>
      </header>

      <div class="accounts-grid">
        <div class="accounts-main">
          <AddAccountCard />

          <div class="card">
            <div class="card-header">
              <div>
                <div class="card-title">{t("accounts.listTitle")}</div>
              </div>
              <Icon name="user" size={16} />
            </div>
            <div class="card-pad">
              <Show
                when={accountsStore.accounts().length > 0}
                fallback={
                  <div class="empty-state" style={{ "min-height": "190px" }}>
                    <div class="empty-icon">
                      <Icon name="user" size={22} />
                    </div>
                    <div style={{ "font-weight": "600", color: "var(--text)" }}>
                      {t("accounts.emptyTitle")}
                    </div>
                    <div style={{ "font-size": "11.5px", "max-width": "360px" }}>
                      {t("accounts.emptyHint")}
                    </div>
                  </div>
                }
              >
                <div class="account-list">
                  <For each={accountsStore.accounts()}>
                    {(account) => <AccountCard account={account} force={force()} />}
                  </For>
                </div>
              </Show>
            </div>
          </div>
        </div>

        <div class="accounts-side">
          <div class="card">
            <div class="card-header">
              <div>
                <div class="card-title">{t("accounts.syncProgress")}</div>
              </div>
              <Icon name="sync" size={16} />
            </div>
            <div class="card-pad">
              <Show
                when={accountsStore.progress()}
                fallback={
                  <div class="form-hint">
                    {accountsStore.accounts().length === 0
                      ? t("accounts.syncIdle")
                      : (lastResultLabel() ?? t("accounts.howStep2"))}
                  </div>
                }
              >
                {(progress) => (
                  <div>
                    <div class="progress-track">
                      <div
                        class="progress-fill"
                        style={{ width: `${progress().percent}%` }}
                      />
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
            </div>
          </div>

          <div class="card">
            <div class="card-header">
              <div>
                <div class="card-title">{t("accounts.howTitle")}</div>
              </div>
              <Icon name="info" size={16} />
            </div>
            <div class="card-pad account-how">
              <div class="account-step">
                <span class="account-step-number">1</span>
                <span>{t("accounts.howStep1")}</span>
              </div>
              <div class="account-step">
                <span class="account-step-number">2</span>
                <span>{t("accounts.howStep2")}</span>
              </div>
              <div class="account-step">
                <span class="account-step-number">3</span>
                <span>{t("accounts.howStep3")}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
