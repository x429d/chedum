import { createSignal } from "solid-js";

import { api } from "../services/api";
import type {
  AppInfo,
  AppSettings,
  BoardTheme,
  EngineStatus,
  PieceSet,
} from "../types/models";
import { toErrorMessage } from "../utils/format";
import { ui } from "./uiStore";

export const DEFAULT_SETTINGS: AppSettings = {
  playerUsername: "",
  analysisDepth: 14,
  multipv: 2,
  boardTheme: "walnut",
  pieceSet: "cburnett",
  showArrows: true,
  autoAnalyze: false,
};

const [settings, setSettings] = createSignal<AppSettings>(DEFAULT_SETTINGS);
const [engine, setEngine] = createSignal<EngineStatus | null>(null);
const [info, setInfo] = createSignal<AppInfo | null>(null);
const [loaded, setLoaded] = createSignal(false);

async function load(): Promise<void> {
  try {
    const [stored, status, appInfo] = await Promise.all([
      api.getSettings(),
      api.getEngineStatus(),
      api.getAppInfo(),
    ]);
    setSettings(stored);
    setEngine(status);
    setInfo(appInfo);
    setLoaded(true);
  } catch (error) {
    ui.notify(toErrorMessage(error), "error");
  }
}

async function refreshEngine(): Promise<void> {
  try {
    setEngine(await api.getEngineStatus());
  } catch (error) {
    ui.notify(toErrorMessage(error), "error");
  }
}

async function save(next: AppSettings): Promise<boolean> {
  try {
    await api.saveSettings(next);
    setSettings(next);
    return true;
  } catch (error) {
    ui.notify(toErrorMessage(error), "error");
    return false;
  }
}

async function update(patch: Partial<AppSettings>): Promise<boolean> {
  return save({ ...settings(), ...patch });
}

async function setBoardTheme(theme: BoardTheme): Promise<boolean> {
  return update({ boardTheme: theme });
}

async function setPieceSet(pieceSet: PieceSet): Promise<boolean> {
  return update({ pieceSet });
}

export const settingsStore = {
  settings,
  engine,
  info,
  loaded,
  load,
  save,
  update,
  setBoardTheme,
  setPieceSet,
  refreshEngine,
};
