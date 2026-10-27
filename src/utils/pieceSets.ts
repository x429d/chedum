import type { Role } from "chessops/types";

import type { Color, PieceSet } from "../types/chess";

export interface PieceSetMeta {
  id: PieceSet;
  label: string;
  author: string;
  license: string;
  source: string;
}

export const PIECE_SETS: PieceSetMeta[] = [
  {
    id: "cburnett",
    label: "Cburnett",
    author: "Colin M.L. Burnett",
    license: "GPLv2+",
    source: "https://github.com/lichess-org/lila/tree/master/public/piece/cburnett",
  },
  {
    id: "merida",
    label: "Merida",
    author: "Armando Hernandez Marroquin",
    license: "GPLv2+",
    source: "https://github.com/lichess-org/lila/tree/master/public/piece/merida",
  },
  {
    id: "chessnut",
    label: "Chessnut",
    author: "Alexis Luengas",
    license: "Apache 2.0",
    source: "https://github.com/lichess-org/lila/tree/master/public/piece/chessnut",
  },
  {
    id: "rhosgfx",
    label: "Rhosgfx",
    author: "RhosGFX",
    license: "CC0 1.0",
    source: "https://github.com/lichess-org/lila/tree/master/public/piece/rhosgfx",
  },
  {
    id: "celtic",
    label: "Celtic",
    author: "Maurizio Monge",
    license: "MIT",
    source: "https://github.com/lichess-org/lila/tree/master/public/piece/celtic",
  },
  {
    id: "kiwen-suwi",
    label: "Kiwen Suwi",
    author: "neverRare",
    license: "CC BY 4.0",
    source: "https://github.com/lichess-org/lila/tree/master/public/piece/kiwen-suwi",
  },
];

const ROLE_LETTER: Record<Role, string> = {
  pawn: "P",
  knight: "N",
  bishop: "B",
  rook: "R",
  queen: "Q",
  king: "K",
};

const ASSETS = import.meta.glob("../assets/pieces/*/*.svg", {
  eager: true,
  query: "?url",
  import: "default",
}) as Record<string, string>;

function assetKey(set: PieceSet, color: Color, role: Role): string {
  const name = `${color === "white" ? "w" : "b"}${ROLE_LETTER[role]}`;
  return `../assets/pieces/${set}/${name}.svg`;
}

export function pieceImage(set: PieceSet, color: Color, role: Role): string {
  return ASSETS[assetKey(set, color, role)] ?? ASSETS[assetKey("cburnett", color, role)] ?? "";
}

export function pieceSetMeta(id: PieceSet): PieceSetMeta {
  return PIECE_SETS.find((set) => set.id === id) ?? PIECE_SETS[0]!;
}
