import { describe, expect, test } from "bun:test";
import { clampSelection, cursorRows, numericOnly } from "../../runtime/web/ui/input-text";

describe("a numeric field", () => {
  test("keeps digits, signs, a point and exponents", () => {
    expect(numericOnly("-12.5e+3", 8)).toEqual({ text: "-12.5e+3", caret: 8 });
  });

  test("drops anything else, and moves the cursor back by what was dropped before it", () => {
    expect(numericOnly("12ab34", 4)).toEqual({ text: "1234", caret: 2 });
    expect(numericOnly("abc", 3)).toEqual({ text: "", caret: 0 });
    expect(numericOnly("1x2", 1)).toEqual({ text: "12", caret: 1 });
    expect(numericOnly("1x2", 3)).toEqual({ text: "12", caret: 2 });
  });
});

describe("a selection", () => {
  test("is put in order and kept inside the text", () => {
    expect(clampSelection(2, 5, 10)).toEqual({ start: 2, end: 5 });
    expect(clampSelection(5, 2, 10)).toEqual({ start: 2, end: 5 });
    expect(clampSelection(-3, 99, 10)).toEqual({ start: 0, end: 10 });
    expect(clampSelection(2.7, 4.2, 10)).toEqual({ start: 2, end: 4 });
  });
});

describe("the rows the cursor is on", () => {
  // Each character's distance from the top, for a text that wraps as three rows of four characters.
  const top = (offset: number): number => Math.floor(Math.min(offset, 11) / 4) * 20;

  test("on the first row, in the middle and on the last", () => {
    expect(cursorRows(top, 2, 2, 12)).toEqual({ firstRow: true, lastRow: false });
    expect(cursorRows(top, 5, 5, 12)).toEqual({ firstRow: false, lastRow: false });
    expect(cursorRows(top, 9, 12, 12)).toEqual({ firstRow: false, lastRow: true });
  });

  test("a text of one row is both", () => {
    expect(cursorRows(() => 0, 1, 1, 3)).toEqual({ firstRow: true, lastRow: true });
  });
});
