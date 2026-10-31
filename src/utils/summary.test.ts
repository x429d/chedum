import { describe, expect, it } from "vitest";

import type { Game, GameAnalysis, MoveEvaluation } from "../types/models";
import { buildGameSummary } from "./summary";

const game: Game = {
  id: 1,
  externalId: "g1",
  source: "chesscom",
  accountId: null,
  pgn: "",
  initialFen: null,
  whiteUsername: "Alice",
  blackUsername: "Bob",
  whiteElo: 1800,
  blackElo: 1750,
  userColor: "white",
  result: "1-0",
  termination: null,
  timeControl: "600+0",
  timeClass: "rapid",
  eco: "C25",
  openingName: "Vienna Game",
  playedAt: 1,
  totalPlies: 4,
  notes: "",
  isFavorite: false,
};

const analysis: GameAnalysis = {
  gameId: 1,
  engineName: "Stockfish 19",
  engineVersion: "19",
  classifierVersion: "classifier-v1",
  accuracyMethodVersion: "lichess-v1",
  depth: 14,
  multipv: 2,
  analyzedAt: "",
  whiteAccuracy: 63.42,
  blackAccuracy: 59.24,
  whiteAcpl: 101.6,
  blackAcpl: 87.4,
  whitePerfElo: null,
  blackPerfElo: null,
  whiteOpeningAcc: 80,
  whiteMiddlegameAcc: 68.4,
  whiteEndgameAcc: 72.5,
  blackOpeningAcc: 70,
  blackMiddlegameAcc: 66,
  blackEndgameAcc: 70,
  counts: {
    white: {
      brilliant: 0,
      great: 0,
      best: 0,
      excellent: 0,
      good: 0,
      book: 0,
      inaccuracy: 0,
      mistake: 2,
      miss: 0,
      blunder: 1,
      forced: 0,
    },
    black: {
      brilliant: 0,
      great: 0,
      best: 0,
      excellent: 0,
      good: 0,
      book: 0,
      inaccuracy: 0,
      mistake: 0,
      miss: 0,
      blunder: 0,
      forced: 0,
    },
  },
  summaryText: "",
  keyMomentsCount: 5,
};

const moves: MoveEvaluation[] = [
  {
    gameId: 1,
    ply: 1,
    playerColor: "white",
    phase: "opening",
    fenBefore: "",
    fenAfter: "",
    playedUci: "e2e4",
    playedSan: "e4",
    bestUci: "e2e4",
    bestSan: "e4",
    evalCp: 20,
    evalMate: null,
    bestEvalCp: 25,
    bestEvalMate: null,
    winProbWhite: 55,
    winProbLoss: 0,
    moveAccuracy: 100,
    classification: "best",
    isCritical: false,
    isKeyMoment: false,
    materialDiffWhite: 0,
    clockRemainingSec: 500,
    timeSpentSec: 5,
    timeFlag: "time_trouble",
    explanationTag: "hung_piece",
    explanationParams: { playedSan: "e4", bestSan: "d4", line: "d4 d5", pawns: "3.0" },
    explanationText: null,
    pvLines: [],
    userComment: "",
  },
];

describe("buildGameSummary", () => {
  it("uses structured metrics instead of the stored summary text", () => {
    const text = buildGameSummary(analysis, game, moves);
    expect(text).toContain("C25");
    expect(text).toContain("Vienna Game");
    expect(text).toContain("63.4%");
    expect(text).toContain("101.6");
    expect(text).toContain("Mistake: 2");
    expect(text).toContain("Blunder: 1");
    expect(text).toContain("Time-trouble moves: 1");
    expect(text).toContain("Key moments: 5");
  });

  it("renders the black perspective when the user played black", () => {
    const text = buildGameSummary({ ...analysis, counts: analysis.counts }, { ...game, userColor: "black" }, []);
    expect(text).toContain("You played Black");
    expect(text).toContain("59.2%");
  });
});
