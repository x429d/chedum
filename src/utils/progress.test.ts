import { describe, expect, it } from "vitest";

import type { AnalysisProgress, SyncProgress } from "../types/models";
import { analysisProgressLabel, syncProgressLabel } from "./progress";

const sync: SyncProgress = {
  phase: "archive",
  message: "ignored",
  archivesDone: 2,
  archivesTotal: 5,
  gamesImported: 12,
  percent: 40,
};

describe("syncProgressLabel", () => {
  it("localizes phase and counters", () => {
    expect(syncProgressLabel(sync)).toContain("Archive 2 of 5");
    expect(syncProgressLabel({ ...sync, phase: "done", gamesImported: 12 })).toContain("+12");
    expect(syncProgressLabel({ ...sync, phase: "done", gamesImported: 0 })).toBe("No new games");
  });
});

describe("analysisProgressLabel", () => {
  it("localizes engine progress", () => {
    const progress: AnalysisProgress = {
      stage: "engine",
      ply: 10,
      total: 40,
      percent: 25,
      message: "ignored",
    };
    expect(analysisProgressLabel(progress)).toBe("Stockfish: position 10 of 40");
    expect(analysisProgressLabel({ ...progress, stage: "done" })).toBe("Analysis complete");
  });
});
