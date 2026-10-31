import { createMemo } from "solid-js";

import { settingsStore } from "../stores/settingsStore";
import type { Color, MoveCategory, Phase, TimeClass, UserColor } from "../types/models";
import { en } from "./en";
import { ru } from "./ru";
import type { TranslationSchema } from "./types";

export interface LanguageOption {
  code: string;
  name: string;
}

export const SUPPORTED_LANGUAGES: LanguageOption[] = [
  { code: "en", name: "English" },
  { code: "ru", name: "Русский" },
];

const TRANSLATIONS: Record<string, TranslationSchema> = {
  en,
  ru,
};

export function detectSystemLanguage(): string {
  if (typeof navigator !== "undefined" && typeof navigator.language === "string") {
    const lang = navigator.language.slice(0, 2).toLowerCase();
    if (lang in TRANSLATIONS) return lang;
  }
  return "en";
}

export const currentLanguage = createMemo((): string => {
  const pref = settingsStore.settings().language;
  if (pref && pref in TRANSLATIONS) return pref;
  return detectSystemLanguage();
});

type PathKeys<T> = T extends object
  ? { [K in keyof T]: `${Exclude<K, symbol>}${PathKeys<T[K]> extends never ? "" : `.${PathKeys<T[K]>}`}` }[keyof T]
  : never;

export type TranslationKey = PathKeys<TranslationSchema>;

function getNestedValue(obj: Record<string, unknown>, path: string): string | undefined {
  const parts = path.split(".");
  let current: unknown = obj;
  for (const part of parts) {
    if (!current || typeof current !== "object") return undefined;
    current = (current as Record<string, unknown>)[part];
  }
  return typeof current === "string" ? current : undefined;
}

export function t(key: TranslationKey, params?: Record<string, string | number>): string {
  const lang = currentLanguage();
  const dict = TRANSLATIONS[lang] ?? TRANSLATIONS.en;
  let text = getNestedValue(dict as unknown as Record<string, unknown>, key);
  if (text === undefined) {
    text = getNestedValue(TRANSLATIONS.en as unknown as Record<string, unknown>, key) ?? key;
  }
  if (!params) return text;
  return Object.entries(params).reduce(
    (acc, [placeholder, val]) => acc.replace(new RegExp(`\\{${placeholder}\\}`, "g"), String(val)),
    text,
  );
}

export function tCategory(category: MoveCategory): string {
  return t(`categories.${category}` as TranslationKey);
}

export function tPhase(phase: Phase): string {
  return t(`phases.${phase}` as TranslationKey);
}

export function tOutcome(result: string, color: UserColor): string {
  let outcome = "draw";
  if (result === "1/2-1/2") outcome = "draw";
  else if (color === "white") outcome = result === "1-0" ? "win" : result === "0-1" ? "loss" : result;
  else if (color === "black") outcome = result === "0-1" ? "win" : result === "1-0" ? "loss" : result;
  else return result;

  if (outcome === "win" || outcome === "loss" || outcome === "draw") {
    return t(`outcomes.${outcome}` as TranslationKey);
  }
  return result;
}

export function tTimeClass(timeClass: TimeClass | null): string {
  if (!timeClass) return "—";
  return t(`timeClasses.${timeClass}` as TranslationKey);
}

export function tSide(side: Color | UserColor): string {
  if (side === "white" || side === "black") {
    return t(`sides.${side}` as TranslationKey);
  }
  return "—";
}

export function pluralGames(count: number): string {
  if (currentLanguage() === "ru") {
    const mod10 = count % 10;
    const mod100 = count % 100;
    if (mod10 === 1 && mod100 !== 11) return `${count} партия`;
    if (mod10 >= 2 && mod10 <= 4 && (mod100 < 10 || mod100 >= 20)) return `${count} партии`;
    return `${count} партий`;
  }
  return count === 1 ? "1 game" : `${count} games`;
}

export function formatDateLocale(timestamp: number): string {
  const date = new Date(timestamp * 1000);
  const locale = currentLanguage() === "ru" ? "ru-RU" : "en-US";
  return new Intl.DateTimeFormat(locale, {
    day: "2-digit",
    month: "short",
    year: "numeric",
  }).format(date);
}

export function formatDurationLocale(totalSeconds: number | null): string {
  if (totalSeconds === null || !Number.isFinite(totalSeconds) || totalSeconds <= 0) {
    return currentLanguage() === "ru" ? "0 с" : "0 s";
  }
  const seconds = Math.round(totalSeconds);
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const rest = seconds % 60;
  const isRu = currentLanguage() === "ru";
  const hUnit = isRu ? "ч" : "h";
  const mUnit = isRu ? "мин" : "m";
  const sUnit = isRu ? "с" : "s";

  if (hours > 0) return `${hours} ${hUnit} ${String(minutes).padStart(2, "0")} ${mUnit}`;
  if (minutes > 0) return `${minutes} ${mUnit} ${String(rest).padStart(2, "0")} ${sUnit}`;
  return `${rest} ${sUnit}`;
}
