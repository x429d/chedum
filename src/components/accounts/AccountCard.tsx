import { Show } from "solid-js";

import { pluralGames, t } from "../../i18n";
import { api } from "../../services/api";
import { accountsStore } from "../../stores/accountsStore";
import { ui } from "../../stores/uiStore";
import type { Account } from "../../types/models";
import { Icon } from "../ui/Icon";

interface Props {
  account: Account;
  force: boolean;
}

function lastSyncLabel(value: string | null): string {
  if (!value) return t("accounts.neverSynced");
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return t("accounts.neverSynced");
  return t("accounts.lastSync", { date: date.toLocaleString() });
}

export function AccountCard(props: Props) {
  const active = () => accountsStore.activeAccountId() === props.account.id;
  const syncing = () => accountsStore.syncingAccountId() === props.account.id;
  const busy = () => accountsStore.busy();

  const showGames = (): void => {
    void accountsStore.setActive(props.account.id).then(() => ui.setView("library"));
  };

  const disconnect = async (): Promise<void> => {
    const confirmed = await api.confirmDisconnect(
      t("accounts.confirmRemove", { account: accountsStore.accountName(props.account) }),
    );
    if (confirmed) await accountsStore.remove(props.account.id);
  };

  const resetHistory = async (): Promise<void> => {
    const confirmed = await api.confirmSyncReset(
      t("accounts.syncResetConfirm", { account: accountsStore.accountName(props.account) }),
    );
    if (confirmed) await accountsStore.resetHistory(props.account.id);
  };

  return (
    <div class="account-card" classList={{ "is-active": active() }}>
      <div
        class="account-platform"
        classList={{ lichess: props.account.platform === "lichess" }}
        title={t(
          props.account.platform === "chesscom"
            ? "accounts.platformChesscom"
            : "accounts.platformLichess",
        )}
      >
        {props.account.platform === "chesscom" ? "cc" : "li"}
      </div>
      <div class="account-main">
        <div class="account-title-row">
          <span class="account-username">@{props.account.username}</span>
          <span class="account-platform-name">
            {t(
              props.account.platform === "chesscom"
                ? "accounts.platformChesscom"
                : "accounts.platformLichess",
            )}
          </span>
          <Show when={active()}>
            <span class="badge account-active-badge">{t("accounts.active")}</span>
          </Show>
        </div>
        <div class="account-meta">
          <span>{pluralGames(props.account.games)}</span>
          <span>
            {props.account.analyzed} {t("accounts.analyzed").toLowerCase()}
          </span>
          <span>{lastSyncLabel(props.account.lastSyncedAt)}</span>
        </div>
      </div>
      <div class="account-actions">
        <button type="button" class="btn btn-sm" onClick={showGames}>
          <Icon name="library" size={13} />
          {t("accounts.showGames")}
        </button>
        <button
          type="button"
          class="btn btn-sm"
          disabled={busy()}
          onClick={() => void accountsStore.syncOne(props.account.id, props.force)}
        >
          <Show when={syncing()} fallback={<Icon name="sync" size={13} />}>
            <span class="spinner" style={{ width: "12px", height: "12px" }} />
          </Show>
          {syncing() ? t("accounts.syncingBtn") : t("accounts.syncBtn")}
        </button>
        <button
          type="button"
          class="btn btn-ghost btn-sm"
          title={t("accounts.syncReset")}
          aria-label={t("accounts.syncReset")}
          disabled={busy()}
          onClick={() => void resetHistory()}
        >
          <Icon name="refresh" size={14} />
        </button>
        <button
          type="button"
          class="btn btn-ghost btn-sm icon-btn"
          title={t("accounts.remove")}
          aria-label={t("accounts.remove")}
          disabled={busy()}
          onClick={() => void disconnect()}
        >
          <Icon name="trash" size={14} />
        </button>
      </div>
    </div>
  );
}
