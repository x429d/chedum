import { createSignal } from "solid-js";

import { t } from "../i18n";
import { api } from "../services/api";
import type { Account, AccountPlatform, SyncProgress, SyncResult } from "../types/models";
import { toErrorMessage } from "../utils/format";
import { libraryStore } from "./libraryStore";
import { settingsStore } from "./settingsStore";
import { ui } from "./uiStore";

const [accounts, setAccounts] = createSignal<Account[]>([]);
const [activeAccountId, setActiveAccountId] = createSignal<number | null>(null);
const [loaded, setLoaded] = createSignal(false);
const [syncingAccountId, setSyncingAccountId] = createSignal<number | null>(null);
const [syncingAll, setSyncingAll] = createSignal(false);
const [progress, setProgress] = createSignal<SyncProgress | null>(null);
const [lastResult, setLastResult] = createSignal<SyncResult | null>(null);

const busy = () => syncingAccountId() !== null || syncingAll();

function platformName(platform: AccountPlatform): string {
  return platform === "chesscom" ? "chess.com" : "lichess";
}

function accountName(account: Account): string {
  return `${platformName(account.platform)} @${account.username}`;
}

function applyActive(accountId: number | null): void {
  setActiveAccountId(accountId);
  libraryStore.setAccount(accountId);
}

async function load(): Promise<void> {
  try {
    const list = await api.listAccounts();
    setAccounts(list);
    const stored = settingsStore.settings().activeAccountId;
    const valid = stored !== null && list.some((account) => account.id === stored) ? stored : null;
    applyActive(valid);
    setLoaded(true);
  } catch (error) {
    ui.notify(toErrorMessage(error), "error");
  }
}

async function refresh(): Promise<void> {
  const list = await api.listAccounts();
  setAccounts(list);
  const current = activeAccountId();
  if (current !== null && !list.some((account) => account.id === current)) {
    applyActive(null);
  }
}

async function setActive(accountId: number | null): Promise<void> {
  applyActive(accountId);
  await settingsStore.update({ activeAccountId: accountId });
  await libraryStore.load();
}

async function add(platform: AccountPlatform, username: string): Promise<boolean> {
  try {
    const account = await api.addAccount(platform, username.trim());
    await load();
    // После подключения показываем все аккаунты: новые партии не должны
    // теряться за фильтром старого профиля.
    await setActive(null);
    ui.notify(t("accounts.added", { account: accountName(account) }), "success");
    void syncOne(account.id, false);
    return true;
  } catch (error) {
    ui.notify(toErrorMessage(error), "error");
    return false;
  }
}

async function remove(accountId: number): Promise<void> {
  const account = accounts().find((value) => value.id === accountId);
  try {
    await api.removeAccount(accountId);
    if (activeAccountId() === accountId) {
      applyActive(null);
      await settingsStore.update({ activeAccountId: null });
    }
    await refresh();
    await libraryStore.load();
    ui.notify(
      t("accounts.removed", { account: account ? accountName(account) : `#${accountId}` }),
      "success",
    );
  } catch (error) {
    ui.notify(toErrorMessage(error), "error");
  }
}

function prefixedProgress(account: Account, update: SyncProgress): SyncProgress {
  return { ...update, message: `${accountName(account)}: ${update.message}` };
}

async function syncOne(accountId: number, force: boolean): Promise<void> {
  if (busy()) return;
  const account = accounts().find((value) => value.id === accountId);
  if (!account) return;
  setSyncingAccountId(accountId);
  setLastResult(null);
  setProgress(null);
  try {
    const result = await api.syncAccount(accountId, force, (update) =>
      setProgress(prefixedProgress(account, update)),
    );
    setLastResult(result);
    await refresh();
    await libraryStore.refreshAfterSync(result.gamesImported);
    if (result.gamesImported > 0) {
      ui.notify(
        t("accounts.syncDone", {
          account: accountName(account),
          imported: result.gamesImported,
          skipped: result.gamesSkipped,
        }),
        "success",
      );
    } else {
      ui.notify(t("accounts.syncEmpty", { account: accountName(account) }), "info");
    }
  } catch (error) {
    ui.notify(`${accountName(account)}: ${toErrorMessage(error)}`, "error");
  } finally {
    setSyncingAccountId(null);
    setProgress(null);
  }
}

async function syncAll(force: boolean): Promise<void> {
  if (busy() || accounts().length === 0) return;
  setSyncingAll(true);
  setLastResult(null);
  setProgress(null);
  let imported = 0;
  let skipped = 0;
  const errors: string[] = [];
  try {
    for (const account of accounts()) {
      setSyncingAccountId(account.id);
      try {
        const result = await api.syncAccount(account.id, force, (update) =>
          setProgress(prefixedProgress(account, update)),
        );
        imported += result.gamesImported;
        skipped += result.gamesSkipped;
      } catch (error) {
        errors.push(`${accountName(account)}: ${toErrorMessage(error)}`);
      } finally {
        setSyncingAccountId(null);
      }
    }
    await refresh();
    await libraryStore.refreshAfterSync(imported);
    if (errors.length > 0) {
      ui.notify(`${t("accounts.syncSomeFailed", { count: errors.length })}: ${errors[0]}`, "error");
    } else if (imported > 0) {
      ui.notify(t("accounts.syncAllDone", { imported, skipped }), "success");
    } else {
      ui.notify(t("accounts.syncAllEmpty"), "info");
    }
  } finally {
    setSyncingAll(false);
    setSyncingAccountId(null);
    setProgress(null);
  }
}

export const accountsStore = {
  accounts,
  activeAccountId,
  loaded,
  syncingAccountId,
  syncingAll,
  progress,
  lastResult,
  busy,
  load,
  refresh,
  setActive,
  add,
  remove,
  syncOne,
  syncAll,
  accountName,
  platformName,
};
