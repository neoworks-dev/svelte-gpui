import { describe, expect, test } from "bun:test";
import { withAlpha } from "../packages/svelte-gpui-ui/src/components/colorMix";
import { neighbourValue } from "../packages/svelte-gpui-ui/src/components/radioGroupContext";
import { nextFocusIndex } from "../packages/svelte-gpui-ui/src/components/rovingFocus";
import { toggleValue } from "../packages/svelte-gpui-ui/src/components/segmentedSelection";
import {
  fractionOf,
  moveRangeThumb,
  nearestThumbIndex,
  snap,
  tickFractions,
  valueAfterKey,
  valueAtOffset,
  type SliderScale,
} from "../packages/svelte-gpui-ui/src/components/sliderMath";

const percentScale: SliderScale = { minimum: 0, maximum: 100, step: 1 };

describe("slider values", () => {
  test("snap rounds to the step counted from the minimum and stays inside the scale", () => {
    const scale: SliderScale = { minimum: 5, maximum: 25, step: 10 };
    expect(snap(scale, 9)).toBe(5);
    expect(snap(scale, 11)).toBe(15);
    expect(snap(scale, 100)).toBe(25);
    expect(snap(scale, -40)).toBe(5);
  });

  test("snap does not leave floating point noise for fractional steps", () => {
    const scale: SliderScale = { minimum: 0, maximum: 1, step: 0.1 };
    expect(snap(scale, 0.30000000000000004)).toBe(0.3);
    expect(snap(scale, 0.74)).toBe(0.7);
  });

  test("fractionOf clamps and survives an empty scale", () => {
    expect(fractionOf(percentScale, 25)).toBe(0.25);
    expect(fractionOf(percentScale, 150)).toBe(1);
    expect(fractionOf(percentScale, -5)).toBe(0);
    expect(fractionOf({ minimum: 3, maximum: 3, step: 1 }, 3)).toBe(0);
  });

  test("a pointer offset maps onto the track and snaps", () => {
    expect(valueAtOffset(percentScale, 200, 400)).toBe(50);
    expect(valueAtOffset(percentScale, -30, 400)).toBe(0);
    expect(valueAtOffset(percentScale, 999, 400)).toBe(100);
    expect(valueAtOffset({ minimum: 0, maximum: 10, step: 5 }, 110, 400)).toBe(5);
    expect(valueAtOffset(percentScale, 10, 0)).toBe(0);
  });

  test("keys move one step, ten with shift or page keys, and jump to the ends", () => {
    expect(valueAfterKey(percentScale, "ArrowRight", 10, false)).toBe(11);
    expect(valueAfterKey(percentScale, "ArrowLeft", 10, true)).toBe(0);
    expect(valueAfterKey(percentScale, "PageUp", 10, false)).toBe(20);
    expect(valueAfterKey(percentScale, "PageDown", 5, false)).toBe(0);
    expect(valueAfterKey(percentScale, "Home", 40, false)).toBe(0);
    expect(valueAfterKey(percentScale, "End", 40, false)).toBe(100);
    expect(valueAfterKey(percentScale, "ArrowRight", 100, false)).toBe(100);
    expect(valueAfterKey(percentScale, "a", 40, false)).toBeUndefined();
  });

  test("a press picks the nearer thumb, and a tie goes to the thumb it lies beyond", () => {
    expect(nearestThumbIndex(10, true, [20, 80])).toBe(0);
    expect(nearestThumbIndex(70, true, [20, 80])).toBe(1);
    expect(nearestThumbIndex(50, true, [20, 80])).toBe(1);
    expect(nearestThumbIndex(10, true, [30, 30])).toBe(0);
    expect(nearestThumbIndex(90, false, [0, 0])).toBe(0);
  });

  test("a range thumb cannot cross the other one", () => {
    expect(moveRangeThumb([20, 70], 0, 90)).toEqual([70, 70]);
    expect(moveRangeThumb([20, 70], 1, 5)).toEqual([20, 20]);
    expect(moveRangeThumb([20, 70], 0, 10)).toEqual([10, 70]);
  });

  test("ticks mark every step, and none when there would be too many", () => {
    expect(tickFractions({ minimum: 0, maximum: 4, step: 1 })).toEqual([0, 0.25, 0.5, 0.75, 1]);
    expect(tickFractions(percentScale)).toEqual([]);
    expect(tickFractions({ minimum: 0, maximum: 4, step: 0 })).toEqual([]);
  });
});

describe("focus movement", () => {
  const states = [false, true, false];

  test("arrow keys skip disabled entries and wrap", () => {
    expect(nextFocusIndex("ArrowRight", 0, states, "horizontal")).toBe(2);
    expect(nextFocusIndex("ArrowRight", 2, states, "horizontal")).toBe(0);
    expect(nextFocusIndex("ArrowLeft", 0, states, "horizontal")).toBe(2);
  });

  test("only the keys of the orientation navigate", () => {
    expect(nextFocusIndex("ArrowDown", 0, states, "horizontal")).toBeUndefined();
    expect(nextFocusIndex("ArrowDown", 0, states, "vertical")).toBe(2);
    expect(nextFocusIndex("ArrowUp", 0, states, "both")).toBe(2);
  });

  test("Home and End find the first and last enabled entry", () => {
    expect(nextFocusIndex("Home", 2, [true, false, false], "vertical")).toBe(1);
    expect(nextFocusIndex("End", 0, [false, false, true], "vertical")).toBe(1);
  });

  test("everything disabled leaves focus where it is", () => {
    expect(nextFocusIndex("ArrowRight", 0, [true, true], "horizontal")).toBeUndefined();
  });
});

describe("radio group neighbours", () => {
  const options = [
    { value: "free", isDisabled: () => false },
    { value: "pro", isDisabled: () => false },
    { value: "team", isDisabled: () => true },
  ];

  test("arrow keys choose the next enabled option and wrap", () => {
    expect(neighbourValue(options, "free", "ArrowDown")).toBe("pro");
    expect(neighbourValue(options, "pro", "ArrowRight")).toBe("free");
    expect(neighbourValue(options, "free", "ArrowUp")).toBe("pro");
    expect(neighbourValue(options, "pro", "ArrowLeft")).toBe("free");
  });

  test("other keys leave the choice alone", () => {
    expect(neighbourValue(options, "free", "Tab")).toBeUndefined();
  });
});

describe("segmented selection", () => {
  test("toggleValue adds a missing value at the end and removes a present one", () => {
    expect(toggleValue(["mon", "wed"], "tue")).toEqual(["mon", "wed", "tue"]);
    expect(toggleValue(["mon", "wed"], "mon")).toEqual(["wed"]);
    expect(toggleValue([], "mon")).toEqual(["mon"]);
  });
});

describe("withAlpha", () => {
  test("appends the opacity as two hex digits and clamps it", () => {
    expect(withAlpha("#f2f1ee", 0.8)).toBe("#f2f1eecc");
    expect(withAlpha("#171715", 0.08)).toBe("#17171514");
    expect(withAlpha("#171715", 2)).toBe("#171715ff");
    expect(withAlpha("#171715", -1)).toBe("#17171500");
  });
});
