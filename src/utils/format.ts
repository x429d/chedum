import {
  formatDateLocale,
  formatDurationLocale,
  tOutcome,
  tSide,
  tTimeClass,
} from "../i18n";
import type { Color, TimeClass, UserColor } from "../types/models";

export function formatDate(timestamp: number): string {
  return formatDateLocale(timestamp);
}

export function formatDateTime(timestamp: number): string {
  const date = new Date(timestamp * 1000);
  return date.toLocaleString();
}

export function formatDuration(totalSeconds: number | null): string {
  return formatDurationLocale(totalSeconds);
}

export function formatClock(seconds: number | null): string {
  if (seconds === null || !Number.isFinite(seconds)) return "—";
  const total = Math.max(0, Math.round(seconds));
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const secs = total % 60;
  if (hours > 0) {
    return `${hours}:${String(minutes).padStart(2, "0")}:${String(secs).padStart(2, "0")}`;
  }
  return `${minutes}:${String(secs).padStart(2, "0")}`;
}

export function formatNumber(value: number | null, digits = 1): string {
  if (value === null || !Number.isFinite(value)) return "—";
  return value.toFixed(digits);
}

export function formatPercent(value: number | null, digits = 1): string {
  if (value === null || !Number.isFinite(value)) return "—";
  return `${value.toFixed(digits)}%`;
}

export function formatEval(cp: number | null, mate: number | null): string {
  if (mate !== null && mate !== undefined) {
    return mate > 0 ? `M${mate}` : `-M${Math.abs(mate)}`;
  }
  if (cp === null || cp === undefined) return "0.00";
  const pawns = cp / 100;
  return `${pawns >= 0 ? "+" : ""}${pawns.toFixed(2)}`;
}

export function formatEvalPawns(value: number | null): string {
  if (value === null || !Number.isFinite(value)) return "—";
  return `${value >= 0 ? "+" : ""}${value.toFixed(2)}`;
}

export function userOutcome(result: string, color: UserColor): "win" | "loss" | "draw" | "unknown" {
  if (result === "1/2-1/2") return "draw";
  if (color === "white") return result === "1-0" ? "win" : result === "0-1" ? "loss" : "unknown";
  if (color === "black") return result === "0-1" ? "win" : result === "1-0" ? "loss" : "unknown";
  return "unknown";
}

export function outcomeLabel(result: string, color: UserColor): string {
  return tOutcome(result, color);
}

export function outcomeClass(result: string, color: UserColor): string {
  switch (userOutcome(result, color)) {
    case "win":
      return "outcome-win";
    case "loss":
      return "outcome-loss";
    case "draw":
      return "outcome-draw";
    default:
      return "outcome-unknown";
  }
}

export function timeClassLabel(value: TimeClass | null): string {
  return tTimeClass(value);
}

export function sideLabel(color: Color | UserColor): string {
  return tSide(color);
}

export function pluralRu(count: number, one: string, few: string, many: string): string {
  const mod10 = count % 10;
  const mod100 = count % 100;
  if (mod10 === 1 && mod100 !== 11) return one;
  if (mod10 >= 2 && mod10 <= 4 && (mod100 < 10 || mod100 >= 20)) return few;
  return many;
}

export function toErrorMessage(error: unknown): string {
  if (typeof error === "string") return error;
  if (error && typeof error === "object" && "message" in error) {
    const message = (error as { message?: unknown }).message;
    if (typeof message === "string" && message.length > 0) return message;
  }
  return "Unknown error";
}

/** Машинный код ошибки из `AppError` (`{ code, message }`). */
export function toErrorCode(error: unknown): string | null {
  if (error && typeof error === "object" && "code" in error) {
    const code = (error as { code?: unknown }).code;
    if (typeof code === "string" && code.length > 0) return code;
  }
  return null;
}
