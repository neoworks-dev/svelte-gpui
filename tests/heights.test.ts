import { describe, expect, test } from "bun:test";
import { Heights } from "../runtime/ui/heights";

const reference = (known: Map<number, number>, length: number, guess: (index: number) => number) => {
  const tops: number[] = [0];
  for (let index = 0; index < length; index++) {
    tops.push(tops[index] + (known.get(index) ?? guess(index)));
  }
  return tops;
};

describe("Heights", () => {
  test("counts every row as the estimate until one is measured", () => {
    const heights = new Heights(10, 40);
    expect(heights.total).toBe(400);
    expect(heights.top(3)).toBe(120);
    expect(heights.indexAt(119)).toBe(2);
    expect(heights.indexAt(120)).toBe(3);
  });

  test("measured rows count as measured, the rest as their estimate", () => {
    const heights = new Heights(6, 40);
    heights.set(0, 20);
    heights.set(2, 300);
    // tops are 0, 20, 60, 360, 400, 440, 480
    expect([0, 1, 2, 3, 4, 5, 6].map((index) => heights.top(index))).toEqual([0, 20, 60, 360, 400, 440, 480]);
    heights.set(0, 30);
    expect(heights.top(1)).toBe(30);
    expect(heights.top(3)).toBe(30 + 40 + 300);
  });

  test("measuring a row moves only the rows under it, by what it differed from its estimate", () => {
    const heights = new Heights(1000, 40);
    const before = heights.total;
    heights.set(500, 2000);
    expect(heights.top(500)).toBe(500 * 40);
    expect(heights.total).toBe(before + 2000 - 40);
    expect(heights.height(900)).toBe(40);
  });

  test("each row counts as its own guess until it is measured", () => {
    const guess = (index: number) => (index % 2 === 0 ? 85 : 30);
    const heights = new Heights(4, 40, guess);
    expect(heights.total).toBe(85 + 30 + 85 + 30);
    heights.set(1, 100);
    expect(heights.top(2)).toBe(85 + 100);
    heights.resize(6, (index) => 10 * index);
    expect(heights.height(4)).toBe(40);
    expect(heights.height(5)).toBe(50);
    expect(heights.height(0)).toBe(85);
    // A guess that is no height counts as the list's estimate.
    heights.resize(8, () => Number.NaN);
    expect(heights.height(7)).toBe(40);
  });

  test("a forgotten row counts as its guess again", () => {
    const heights = new Heights(3, 40, () => 70);
    heights.set(0, 10);
    heights.set(2, 10);
    heights.forgetOutside(2, 3);
    expect(heights.height(0)).toBe(70);
    expect(heights.total).toBe(70 + 70 + 10);
  });

  test("agrees with a plain sum on random heights", () => {
    const length = 1000;
    const guess = (index: number) => 20 + (index % 5) * 9;
    const heights = new Heights(length, 33, guess);
    const known = new Map<number, number>();
    let seed = 7;
    const next = () => (seed = (seed * 1103515245 + 12345) & 0x7fffffff) / 0x7fffffff;
    for (let round = 0; round < 400; round++) {
      const index = Math.floor(next() * length);
      const height = 10 + Math.floor(next() * 200);
      heights.set(index, height);
      known.set(index, height);
    }
    const tops = reference(known, length, guess);
    for (let index = 0; index <= length; index += 37) {
      expect(heights.top(index)).toBeCloseTo(tops[index], 6);
    }
    for (const probe of [0, 1, 500, 12345.5, tops[length] - 1]) {
      const found = heights.indexAt(probe);
      expect(tops[found]).toBeLessThanOrEqual(probe + 1e-9);
      expect(tops[found + 1] ?? Infinity).toBeGreaterThan(probe);
    }
  });

  test("keeps what it knows when it grows, shrinks and is replaced", () => {
    const heights = new Heights(3, 10);
    heights.set(1, 50);
    heights.resize(8);
    expect(heights.isMeasured(1)).toBe(true);
    heights.set(7, 30);
    expect(heights.top(8)).toBe(10 + 50 + 5 * 10 + 30);
    heights.resize(2);
    expect(heights.length).toBe(2);
    expect(heights.total).toBe(10 + 50);
    heights.replace(4, [5, 0, 0, 15]);
    expect(heights.total).toBe(5 + 10 + 10 + 15);
    expect(heights.snapshot()).toEqual(new Float64Array([5, 0, 0, 15]));
  });

  test("forgets the rows outside a range and keeps the ones inside", () => {
    const heights = new Heights(10, 40);
    for (let index = 0; index < 10; index++) {
      heights.set(index, 20 + index);
    }
    heights.forgetOutside(3, 6);
    expect(heights.snapshot()).toEqual(new Float64Array([0, 0, 0, 23, 24, 25, 0, 0, 0, 0]));
    expect(heights.total).toBe(7 * 40 + 23 + 24 + 25);
    // A range that reaches past either end is cut to the list.
    heights.forgetOutside(-5, 4);
    expect(heights.snapshot()).toEqual(new Float64Array([0, 0, 0, 23, 0, 0, 0, 0, 0, 0]));
    heights.forgetOutside(0, 99);
    expect(heights.snapshot()).toEqual(new Float64Array([0, 0, 0, 23, 0, 0, 0, 0, 0, 0]));
  });

  test("handles a hundred thousand rows", () => {
    const heights = new Heights(100_000, 40);
    const started = performance.now();
    for (let index = 0; index < 100_000; index += 3) {
      heights.set(index, 20 + (index % 7) * 10);
    }
    for (let probe = 0; probe < 2000; probe++) {
      heights.indexAt(probe * 1500);
    }
    expect(performance.now() - started).toBeLessThan(2000);
    expect(heights.top(heights.length)).toBeGreaterThan(1_000_000);
  });

  test("an empty list is fine", () => {
    const heights = new Heights(0, 40);
    expect(heights.total).toBe(0);
    expect(heights.indexAt(100)).toBe(0);
    heights.resize(2);
    expect(heights.total).toBe(80);
  });
});
