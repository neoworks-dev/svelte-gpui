import { describe, expect, test } from "bun:test";
import { place, type Rect, type Wanted } from "../../runtime/web/ui/placement";
import { stacking } from "../../runtime/web/ui/overlays";

// The same cases as the host's, in host/src/render/overlay.rs: a popover opens on the same side and
// flips and shifts at the same points on both targets.

const WINDOW = { width: 1000, height: 800 };
const SIZE = { width: 150, height: 100 };

const wanted = (side: Wanted["side"], alignment: Wanted["alignment"] = "start"): Wanted => ({
  side,
  alignment,
  offset: 0,
  crossOffset: 0,
  flip: true,
  shift: true,
  margin: 4,
  atPoint: false,
});

const button = (x: number, y: number): Rect => ({ x, y, width: 80, height: 30 });

describe("placing an overlay", () => {
  test("it sits under the anchor, lined up with its start, when there is room", () => {
    expect(place(button(100, 100), SIZE, WINDOW, wanted("bottom"))).toEqual({ x: 100, y: 130, side: "bottom" });
  });

  test("alignment lines it up by centre or end", () => {
    expect(place(button(100, 100), SIZE, WINDOW, wanted("bottom", "center")).x).toBe(100 + (80 - 150) / 2);
    expect(place(button(300, 100), SIZE, WINDOW, wanted("bottom", "end")).x).toBe(300 + 80 - 150);
  });

  test("the offset is the gap along the side, the cross offset slides it along the edge", () => {
    const placed = place(button(100, 100), SIZE, WINDOW, { ...wanted("bottom"), offset: 6, crossOffset: 10 });
    expect(placed).toEqual({ x: 110, y: 136, side: "bottom" });
    expect(place(button(100, 100), SIZE, WINDOW, { ...wanted("right"), offset: 6 })).toMatchObject({ x: 186, y: 100 });
  });

  test("it flips above the anchor when there is no room below and more above", () => {
    const placed = place(button(100, 740), SIZE, WINDOW, wanted("bottom"));
    expect(placed.side).toBe("top");
    expect(placed.y).toBe(740 - 100);
  });

  test("it does not flip to a side with even less room", () => {
    const placed = place(button(100, 10), { width: 150, height: 900 }, WINDOW, wanted("bottom"));
    expect(placed.side).toBe("bottom");
  });

  test("without `flip` it stays on its side", () => {
    expect(place(button(100, 740), SIZE, WINDOW, { ...wanted("bottom"), flip: false, shift: false }).side).toBe("bottom");
  });

  test("it shifts back inside the window, keeping the margin", () => {
    expect(place(button(950, 100), SIZE, WINDOW, wanted("bottom")).x).toBe(1000 - 4 - 150);
    expect(place(button(-20, 100), SIZE, WINDOW, wanted("bottom")).x).toBe(4);
    expect(place(button(950, 100), SIZE, WINDOW, { ...wanted("bottom"), shift: false }).x).toBe(950);
  });

  test("it flips to the left of the anchor at the window's right edge", () => {
    const placed = place(button(900, 100), SIZE, WINDOW, wanted("right"));
    expect(placed.side).toBe("left");
    expect(placed.x).toBe(900 - 150);
  });

  test("at a point, the alignment flips when it would stick out", () => {
    const point: Rect = { x: 950, y: 100, width: 0, height: 0 };
    const placed = place(point, SIZE, WINDOW, { ...wanted("bottom"), atPoint: true });
    expect(placed.x).toBe(950 - 150);
    // With room, it opens at the point, toward the end.
    const roomy = place({ x: 100, y: 100, width: 0, height: 0 }, SIZE, WINDOW, { ...wanted("bottom"), atPoint: true });
    expect(roomy).toEqual({ x: 100, y: 100, side: "bottom" });
  });

  test("something too big for the room starts at the margin", () => {
    expect(place(button(100, 100), { width: 2000, height: 100 }, WINDOW, wanted("bottom")).x).toBe(4);
  });
});

describe("stacking", () => {
  test("a higher layer is above a lower one, and a later one above an earlier one in its layer", () => {
    expect(stacking(1, 1)).toBeGreaterThan(stacking(0, 99_999));
    expect(stacking(0, 5)).toBeGreaterThan(stacking(0, 4));
  });
});
