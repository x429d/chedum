import { createSignal } from "solid-js";

import { t } from "../i18n";
import { api } from "../services/api";
import type { Account, AccountPlatform, SyncProgress, SyncResult } from "../types/models";
import { toErrorCode, toErrorMessage } from "../utils/format";
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
    // After connecting an account show all accounts: new games must not be
    // hidden behind the previous profile filter.
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

function resultMessage(account: Account, result: SyncResult): string {
  return t("accounts.syncResult", {
    account: accountName(account),
    imported: result.gamesImported,
    duplicates: result.duplicates,
    filtered: result.filtered,
    errors: result.errors.length,
  });
}

function resultKind(result: SyncResult): "success" | "info" | "error" {
  if (result.errors.length > 0) return "error";
  return result.gamesImported > 0 ? "success" : "info";
}

function hasSyncDetails(result: SyncResult): boolean {
  return (
    result.gamesImported > 0 ||
    result.duplicates > 0 ||
    result.filtered > 0 ||
    result.errors.length > 0
  );
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
    if (hasSyncDetails(result)) {
      ui.notify(resultMessage(account, result), resultKind(result));
    } else {
      ui.notify(t("accounts.syncEmpty", { account: accountName(account) }), "info");
    }
  } catch (error) {
    if (toErrorCode(error) === "cancelled") {
      ui.notify(t("accounts.syncCancelled"), "info");
    } else {
      ui.notify(`${accountName(account)}: ${toErrorMessage(error)}`, "error");
    }
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
  let duplicates = 0;
  let filtered = 0;
  let errors = 0;
  let firstError: string | null = null;
  let cancelled = false;
  try {
    for (const account of accounts()) {
      setSyncingAccountId(account.id);
      try {
        const result = await api.syncAccount(account.id, force, (update) =>
          setProgress(prefixedProgress(account, update)),
        );
        imported += result.gamesImported;
        duplicates += result.duplicates;
        filtered += result.filtered;
        errors += result.errors.length;
        if (firstError === null && result.errors.length > 0) {
          firstError = result.errors[0] ?? null;
        }
      } catch (error) {
        if (toErrorCode(error) === "cancelled") {
          cancelled = true;
          break;
        }
        errors += 1;
        firstError ??= toErrorMessage(error);
      } finally {
        setSyncingAccountId(null);
      }
    }
    await refresh();
    await libraryStore.refreshAfterSync(imported);
    if (cancelled) {
      ui.notify(t("accounts.syncCancelled"), "info");
    } else if (errors > 0) {
      ui.notify(
        `${t("accounts.syncAllDone", { imported, duplicates, filtered, errors })}: ${firstError ?? ""}`,
        "error",
      );
    } else if (imported > 0 || duplicates > 0 || filtered > 0) {
      ui.notify(t("accounts.syncAllDone", { imported, duplicates, filtered, errors }), "success");
    } else {
      ui.notify(t("accounts.syncAllEmpty"), "info");
    }
  } finally {
    setSyncingAll(false);
    setSyncingAccountId(null);
    setProgress(null);
  }
}

async function cancel(): Promise<void> {
  try {
    await api.cancelSync();
  } catch (error) {
    ui.notify(toErrorMessage(error), "error");
  }
}

async function resetHistory(accountId: number): Promise<void> {
  const account = accounts().find((value) => value.id === accountId);
  try {
    await api.resetSyncHistory(accountId);
    await refresh();
    ui.notify(
      t("accounts.syncResetDone", {
        account: account ? accountName(account) : `#${accountId}`,
      }),
      "success",
    );
  } catch (error) {
    ui.notify(toErrorMessage(error), "error");
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
  cancel,
  resetHistory,
  accountName,
  platformName,
};
