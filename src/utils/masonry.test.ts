import { describe, expect, it } from "vitest";

import { packMasonry } from "./masonry";

describe("packMasonry", () => {
  it("puts every next card into the shortest column", () => {
    const slots = packMasonry([100, 50, 30], 2);
    expect(slots.map((slot) => slot.column)).toEqual([1, 2, 2]);
  });

  it("keeps cards in a column sequential, with the requested gap", () => {
    const heights = [120, 80, 40, 200, 60, 90];
    const slots = packMasonry(heights, 2);
    const columns = new Map<number, number[]>();
    slots.forEach((slot, index) => {
      columns.set(slot.column, [...(columns.get(slot.column) ?? []), index]);
    });
    for (const indices of columns.values()) {
      for (let index = 1; index < indices.length; index += 1) {
        const previousIndex = indices[index - 1]!;
        const currentIndex = indices[index]!;
        const previous = slots[previousIndex]!;
        const current = slots[currentIndex]!;
        expect(current.start).toBe(previous.start + previous.span);
        const gap = current.start - (previous.start + (heights[previousIndex] ?? 0));
        expect(gap).toBeGreaterThanOrEqual(16);
        expect(gap).toBeLessThan(17);
      }
    }
  });

  it("covers the measured height and never overlaps", () => {
    const heights = [137, 42.5, 0, 311, 88];
    const slots = packMasonry(heights, 2);
    slots.forEach((slot, index) => {
      expect(slot.span).toBeGreaterThanOrEqual((heights[index] ?? 0) + 16);
      expect(slot.span).toBeLessThan((heights[index] ?? 0) + 17);
    });
  });

  it("falls back to a single column", () => {
    const slots = packMasonry([10, 20, 30], 1);
    expect(slots.map((slot) => slot.column)).toEqual([1, 1, 1]);
    expect(slots.map((slot) => slot.start)).toEqual([1, 27, 63]);
  });

  it("handles no cards and invalid heights", () => {
    expect(packMasonry([], 2)).toEqual([]);
    const slots = packMasonry([Number.NaN], 2);
    expect(slots[0]).toEqual({ column: 1, start: 1, span: 16 });
  });
});
