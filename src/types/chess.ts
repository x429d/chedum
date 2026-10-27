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
  color: string;
}

export const CATEGORY_META: Record<MoveCategory, CategoryMeta> = {
  brilliant: { label: "Бриллиантовый", color: "#14B8A6" },
  great: { label: "Отличный", color: "#3B82F6" },
  best: { label: "Лучший", color: "#22C55E" },
  excellent: { label: "Прекрасный", color: "#4ADE80" },
  good: { label: "Хороший", color: "#86EFAC" },
  book: { label: "Теория", color: "#D97706" },
  inaccuracy: { label: "Неточность", color: "#FACC15" },
  mistake: { label: "Ошибка", color: "#F97316" },
  miss: { label: "Упущенный шанс", color: "#EC4899" },
  blunder: { label: "Зевок", color: "#EF4444" },
  forced: { label: "Единственный", color: "#6B7280" },
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
