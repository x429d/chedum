import { describe, expect, it } from "vitest";

import { formatClock, formatEval, toErrorCode, toErrorMessage } from "./format";

describe("format helpers", () => {
  it("formats clocks and evaluations", () => {
    expect(formatClock(65)).toBe("1:05");
    expect(formatClock(3661)).toBe("1:01:01");
    expect(formatEval(35, null)).toBe("+0.35");
    expect(formatEval(null, -3)).toBe("-M3");
  });
});

describe("error helpers", () => {
  it("keeps the stable code and localizes known errors", () => {
    const error = { code: "validation", message: "depth out of range" };
    expect(toErrorCode(error)).toBe("validation");
    expect(toErrorMessage(error)).toContain("Invalid input");
    expect(toErrorMessage(error)).toContain("depth out of range");
  });

  it("hides technical details for internal errors", () => {
    expect(toErrorMessage({ code: "internal", message: "secret stack" })).toBe("Internal error");
  });
});
