export type Color = "white" | "black";
export type UserColor = Color | "none";
export type BoardTheme = "midnight" | "walnut" | "slate";
export type PieceSet =
  | "cburnett"
  | "merida"
  | "chessnut"
  | "rhosgfx"
  | "celtic"
  | "kiwen-suwi";

export type MoveCategory =
  | "book"
  | "forced"
  | "brilliant"
  | "great"
  | "best"
  | "excellent"
  | "good"
  | "miss"
  | "inaccuracy"
  | "mistake"
  | "blunder";

export type Phase = "opening" | "middlegame" | "endgame";
export type TimeFlag = "normal" | "time_trouble" | "impulsive_error" | "overthink";

export const CATEGORY_ORDER: MoveCategory[] = [
  "brilliant",
  "great",
  "best",
  "excellent",
  "good",
  "book",
  "inaccuracy",
  "mistake",
  "miss",
  "blunder",
  "forced",
];

export interface CategoryMeta {
  label: string;
  symbol: string;
  color: string;
}

export const CATEGORY_META: Record<MoveCategory, CategoryMeta> = {
  brilliant: { label: "Бриллиантовый", symbol: "!!", color: "#14B8A6" },
  great: { label: "Отличный", symbol: "!", color: "#3B82F6" },
  best: { label: "Лучший", symbol: "★", color: "#22C55E" },
  excellent: { label: "Прекрасный", symbol: "👍", color: "#4ADE80" },
  good: { label: "Хороший", symbol: "✓", color: "#86EFAC" },
  book: { label: "Теория", symbol: "📖", color: "#D97706" },
  inaccuracy: { label: "Неточность", symbol: "?!", color: "#FACC15" },
  mistake: { label: "Ошибка", symbol: "?", color: "#F97316" },
  miss: { label: "Упущенный шанс", symbol: "✕", color: "#EC4899" },
  blunder: { label: "Зевок", symbol: "??", color: "#EF4444" },
  forced: { label: "Единственный", symbol: "□", color: "#6B7280" },
};

export const PHASE_LABELS: Record<Phase, string> = {
  opening: "Дебют",
  middlegame: "Миттельшпиль",
  endgame: "Эндшпиль",
};

export interface ArrowSpec {
  from: string;
  to: string;
  color: string;
  opacity?: number;
  dashed?: boolean;
  width?: number;
}

export interface BoardBadge {
  square: string;
  category: MoveCategory;
}
