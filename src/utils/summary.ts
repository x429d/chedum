import { t, tCategory, tPhase } from "../i18n";
import type { Game, GameAnalysis, MoveEvaluation, Phase, UserColor } from "../types/models";
import { formatNumber } from "./format";

const ERROR_CATEGORIES = ["blunder", "mistake", "miss", "inaccuracy"] as const;

function openingPart(game: Game): string | null {
  const eco = game.eco;
  const name = game.openingName;
  if (eco && name) return t("summary.opening", { eco, name });
  if (eco) return t("summary.openingEco", { eco });
  if (name) return t("summary.openingName", { name });
  return null;
}

function resultPart(game: Game, userColor: UserColor): string {
  const white = game.whiteUsername;
  const black = game.blackUsername;
  const opponent = userColor === "black" ? white : black;
  if (game.result === "1/2-1/2") return t("summary.draw", { white, black });
  if (userColor === "white") {
    return game.result === "1-0"
      ? t("summary.wonAsWhite", { opponent })
      : t("summary.lostAsWhite", { opponent });
  }
  if (userColor === "black") {
    return game.result === "0-1"
      ? t("summary.wonAsBlack", { opponent })
      : t("summary.lostAsBlack", { opponent });
  }
  return t("summary.result", { white, black, result: game.result });
}

function accuracyPart(analysis: GameAnalysis, userColor: UserColor): string | null {
  const accuracy = userColor === "black" ? analysis.blackAccuracy : analysis.whiteAccuracy;
  if (accuracy === null) return null;
  const acpl = userColor === "black" ? analysis.blackAcpl : analysis.whiteAcpl;
  const value = formatNumber(accuracy, 1);
  if (acpl === null) return t("summary.accuracyNoAcpl", { accuracy: value });
  return t("summary.accuracy", { accuracy: value, acpl: formatNumber(acpl, 1) });
}

function errorsPart(analysis: GameAnalysis, userColor: UserColor): string {
  const counts = userColor === "black" ? analysis.counts.black : analysis.counts.white;
  const parts = ERROR_CATEGORIES.filter((category) => (counts[category] ?? 0) > 0).map(
    (category) => `${tCategory(category)}: ${counts[category]}`,
  );
  return parts.length > 0 ? t("summary.weakness", { errors: parts.join(", ") }) : t("summary.clean");
}

function phaseValues(analysis: GameAnalysis, userColor: UserColor): [Phase, number | null][] {
  if (userColor === "black") {
    return [
      ["opening", analysis.blackOpeningAcc],
      ["middlegame", analysis.blackMiddlegameAcc],
      ["endgame", analysis.blackEndgameAcc],
    ];
  }
  return [
    ["opening", analysis.whiteOpeningAcc],
    ["middlegame", analysis.whiteMiddlegameAcc],
    ["endgame", analysis.whiteEndgameAcc],
  ];
}

function weakestPhasePart(analysis: GameAnalysis, userColor: UserColor): string | null {
  const values = phaseValues(analysis, userColor).filter(
    (entry): entry is [Phase, number] => entry[1] !== null,
  );
  if (values.length === 0) return null;
  const weakest = values.reduce((left, right) => (right[1] < left[1] ? right : left));
  return t("summary.weakestPhase", {
    phase: tPhase(weakest[0]),
    accuracy: formatNumber(weakest[1], 1),
  });
}

function timeParts(moves: MoveEvaluation[], userColor: UserColor): { trouble: number; impulsive: number } {
  let trouble = 0;
  let impulsive = 0;
  for (const move of moves) {
    if (move.playerColor !== userColor) continue;
    if (move.timeFlag === "time_trouble") trouble += 1;
    if (move.timeFlag === "impulsive_error") impulsive += 1;
  }
  return { trouble, impulsive };
}

/**
 * Builds the game summary from structured metrics in the selected language.
 * The backend stores the same fields; `summaryText` is only a legacy fallback.
 */
export function buildGameSummary(
  analysis: GameAnalysis,
  game: Game,
  moves: MoveEvaluation[],
): string {
  const userColor: UserColor =
    game.userColor === "black" ? "black" : game.userColor === "white" ? "white" : "none";
  const parts: string[] = [];
  const opening = openingPart(game);
  if (opening) parts.push(opening);
  parts.push(resultPart(game, userColor));
  if (userColor !== "none") {
    const accuracy = accuracyPart(analysis, userColor);
    if (accuracy) parts.push(accuracy);
    parts.push(errorsPart(analysis, userColor));
    const phase = weakestPhasePart(analysis, userColor);
    if (phase) parts.push(phase);
  }
  const time = userColor === "none" ? { trouble: 0, impulsive: 0 } : timeParts(moves, userColor);
  parts.push(
    analysis.keyMomentsCount > 0
      ? t("summary.keyMoments", {
          count: analysis.keyMomentsCount,
          timeTrouble: time.trouble,
          impulsive: time.impulsive,
        })
      : t("summary.noKeyMoments"),
  );
  return parts.join(" ");
}
