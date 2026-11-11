// The parts of a text field that are about text, not the page.

/** What a `numeric` field takes: digits, signs, a point and exponents. */
const NOT_NUMERIC = /[^0-9+\-.eE]/g;

/** Drops what a `numeric` field does not take, and moves the cursor back by the characters dropped before it. */
export function numericOnly(text: string, caret: number): { text: string; caret: number } {
  const kept = text.replace(NOT_NUMERIC, "");
  if (kept === text) {
    return { text, caret };
  }
  return { text: kept, caret: text.slice(0, caret).replace(NOT_NUMERIC, "").length };
}

/** A span's two ends in order, kept inside the text, as `setSelectionRange` takes them. */
export function clampSelection(start: number, end: number, length: number): { start: number; end: number } {
  const inside = (offset: number): number => Math.min(Math.max(Math.trunc(offset), 0), length);
  const a = inside(start);
  const b = inside(end);
  return { start: Math.min(a, b), end: Math.max(a, b) };
}

/**
 * Whether the cursor is on the first and on the last row of a text drawn `rows` high, where `top(offset)` is the
 * height above the row a character is on, as the page lays it out. A long line that wraps counts as several rows.
 */
export function cursorRows(
  top: (offset: number) => number,
  start: number,
  end: number,
  length: number,
): { firstRow: boolean; lastRow: boolean } {
  return { firstRow: top(start) === top(0), lastRow: top(end) === top(length) };
}
