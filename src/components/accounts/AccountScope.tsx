import { createEffect, For, Show } from "solid-js";

import { t } from "../../i18n";
import { accountsStore } from "../../stores/accountsStore";
import { ui } from "../../stores/uiStore";
import { Icon } from "../ui/Icon";

export function AccountScope() {
  let select: HTMLSelectElement | undefined;

  const totalGames = () =>
    accountsStore.accounts().reduce((sum, account) => sum + account.games, 0);

  createEffect(() => {
    const active = accountsStore.activeAccountId();
    const accounts = accountsStore.accounts();
    if (!select) return;
    const next =
      active !== null && accounts.some((account) => account.id === active) ? String(active) : "";
    // For пересоздаёт <option> после синхронизации, и браузер сбрасывает
    // выбранное значение на первый пункт. Возвращаем его после рендера.
    const apply = () => {
      if (select && select.value !== next) select.value = next;
    };
    apply();
    queueMicrotask(apply);
  });

  return (
    <Show
      when={accountsStore.accounts().length > 0}
      fallback={
        <button
          type="button"
          class="btn btn-primary btn-sm"
          onClick={() => ui.setView("accounts")}
        >
          <Icon name="plus" size={13} />
          {t("accounts.addBtn")}
        </button>
      }
    >
      <div class="account-scope">
        <span class="account-scope-label">{t("accounts.scope")}</span>
        <select
          ref={(element) => (select = element)}
          class="select"
          aria-label={t("accounts.scope")}
          value={accountsStore.activeAccountId() ?? ""}
          onChange={(event) => {
            const value = event.currentTarget.value;
            void accountsStore.setActive(value ? Number(value) : null);
          }}
        >
          <option value="">{`${t("accounts.allAccounts")} (${totalGames()})`}</option>
          <For each={accountsStore.accounts()}>
            {(account) => (
              <option value={account.id}>
                {`${accountsStore.platformName(account.platform)} · @${account.username} (${account.games})`}
              </option>
            )}
          </For>
        </select>
        <button
          type="button"
          class="btn btn-ghost btn-sm icon-btn"
          title={t("accounts.manage")}
          aria-label={t("accounts.manage")}
          onClick={() => ui.setView("accounts")}
        >
          <Icon name="settings" size={14} />
        </button>
      </div>
    </Show>
  );
}
