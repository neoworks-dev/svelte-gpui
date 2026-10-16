import { describe, expect, test } from "bun:test";
import { offsetAfter } from "../runtime/ui/scroll";

const position = { x: 30, y: 200, maxX: 100, maxY: 1000 };

describe("offsetAfter", () => {
  test("moves along the vertical axis by default", () => {
    expect(offsetAfter(position, false, 50)).toBe(250);
    expect(offsetAfter(position, false, -50)).toBe(150);
  });

  test("moves along the horizontal axis for a horizontal scroller", () => {
    expect(offsetAfter(position, true, 50)).toBe(80);
  });

  test("stops at both ends", () => {
    expect(offsetAfter(position, false, 5000)).toBe(1000);
    expect(offsetAfter(position, false, -5000)).toBe(0);
    expect(offsetAfter(position, true, 5000)).toBe(100);
  });

  test("does not move a scroller whose content fits", () => {
    expect(offsetAfter({ x: 0, y: 0, maxX: 0, maxY: 0 }, false, 40)).toBe(0);
  });
});
