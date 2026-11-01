export type Color = "white" | "black";
export type UserColor = Color | "none";
export type BoardTheme = "midnight" | "walnut" | "slate";
export type Language = "ru" | "en";
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
  brilliant: { label: "Brilliant", color: "#14B8A6" },
  great: { label: "Great", color: "#3B82F6" },
  best: { label: "Best", color: "#22C55E" },
  excellent: { label: "Excellent", color: "#4ADE80" },
  good: { label: "Good", color: "#86EFAC" },
  book: { label: "Book", color: "#D97706" },
  inaccuracy: { label: "Inaccuracy", color: "#FACC15" },
  mistake: { label: "Mistake", color: "#F97316" },
  miss: { label: "Missed Win", color: "#EC4899" },
  blunder: { label: "Blunder", color: "#EF4444" },
  forced: { label: "Forced", color: "#6B7280" },
};

export const PHASE_LABELS: Record<Phase, string> = {
  opening: "Opening",
  middlegame: "Middlegame",
  endgame: "Endgame",
};

export interface ArrowSpec {
  from: string;
  to: string;
  color: string;
  opacity?: number;
  dashed?: boolean;
  width?: number;
}

/** A square the user marked with the right mouse button. */
export interface SquareHighlight {
  square: string;
  color: string;
}

export interface BoardBadge {
  square: string;
  category: MoveCategory;
}
