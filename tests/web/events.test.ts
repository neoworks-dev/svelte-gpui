import { describe, expect, test } from "bun:test";
import {
  ClickCounter,
  fileDragEventOf,
  keyEventOf,
  modifiersOf,
  pointerEventOf,
  takesButton,
  wheelEventOf,
} from "../../runtime/web/ui/events";

const none = { ctrlKey: false, shiftKey: false, altKey: false, metaKey: false };

describe("pointer events", () => {
  test("positions come in the element's coordinates and the window's", () => {
    const event = pointerEventOf({ clientX: 150, clientY: 90, button: 0, ...none }, { left: 100, top: 40 }, 2);
    expect(event).toEqual({
      x: 50,
      y: 50,
      windowX: 150,
      windowY: 90,
      button: 0,
      clicks: 2,
      ctrl: false,
      shift: false,
      alt: false,
      meta: false,
    });
  });

  test("a pointer outside the element, as captured moves are, has negative coordinates", () => {
    const event = pointerEventOf({ clientX: 10, clientY: 10, button: -1, ...none }, { left: 100, top: 40 });
    expect(event.x).toBe(-90);
    expect(event.y).toBe(-30);
    expect(event.button).toBe(-1);
    expect(event.clicks).toBe(1);
  });

  test("modifiers have the host's names", () => {
    expect(modifiersOf({ ctrlKey: true, shiftKey: false, altKey: true, metaKey: false })).toEqual({
      ctrl: true,
      shift: false,
      alt: true,
      meta: false,
    });
  });

  test("a view takes the buttons it names, and all three without a list", () => {
    expect(takesButton(0, undefined)).toBe(true);
    expect(takesButton(2, undefined)).toBe(true);
    expect(takesButton(0, ["left"])).toBe(true);
    expect(takesButton(2, ["left"])).toBe(false);
    expect(takesButton(1, ["left", "middle"])).toBe(true);
    expect(takesButton(2, ["right"])).toBe(true);
  });

  test("presses in a row on one place count up, and start again when late or far", () => {
    const counter = new ClickCounter();
    expect(counter.press(1000, 10, 10)).toBe(1);
    expect(counter.press(1200, 11, 10)).toBe(2);
    expect(counter.press(1400, 10, 11)).toBe(3);
    expect(counter.press(2500, 10, 11)).toBe(1);
    expect(counter.press(2600, 90, 90)).toBe(1);
  });
});

describe("the wheel", () => {
  test("a pixel wheel passes its deltas, a line wheel is worth sixteen pixels a line", () => {
    expect(wheelEventOf({ deltaX: 3, deltaY: -40, deltaMode: 0, ...none })).toMatchObject({ dx: 3, dy: -40 });
    expect(wheelEventOf({ deltaX: 0, deltaY: 3, deltaMode: 1, ...none })).toMatchObject({ dx: 0, dy: 48 });
    expect(wheelEventOf({ deltaX: 0, deltaY: 1, deltaMode: 2, ...none }).dy).toBeGreaterThan(100);
  });
});

describe("keys", () => {
  function press(key: string, target: unknown = { tagName: "DIV" }) {
    const log = { prevented: 0, stopped: 0 };
    const event = keyEventOf({
      key,
      code: `Key${key}`,
      repeat: false,
      target,
      ...none,
      preventDefault: () => log.prevented++,
      stopPropagation: () => log.stopped++,
    });
    return { event, log };
  }

  test("a key carries the page's key, code and modifiers", () => {
    const { event } = press("k");
    expect(event).toMatchObject({ key: "k", code: "Keyk", repeat: false, ctrl: false });
  });

  test("preventing a key prevents the page's own handling of it, and nothing else", () => {
    const { event, log } = press("Enter");
    event.preventDefault();
    expect(log).toEqual({ prevented: 1, stopped: 0 });
  });

  test("stopping a key stops it going up, and a scrolling key does not also scroll the page", () => {
    const { event, log } = press("ArrowDown");
    event.stopPropagation();
    expect(log).toEqual({ prevented: 1, stopped: 1 });
  });

  test("stopping a letter leaves its default alone", () => {
    const { event, log } = press("a");
    event.stopPropagation();
    expect(log).toEqual({ prevented: 0, stopped: 1 });
  });

  test("in a field a stopped arrow still moves the cursor, unless the handler prevents it", () => {
    const { event, log } = press("ArrowLeft", { tagName: "TEXTAREA" });
    event.stopPropagation();
    expect(log).toEqual({ prevented: 0, stopped: 1 });
  });
});

describe("files dragged in", () => {
  test("a browser tells the page how many, and no paths; a drop carries the files", () => {
    const over = fileDragEventOf({ clientX: 30, clientY: 50 }, { left: 10, top: 10 }, 2);
    expect(over).toEqual({ x: 20, y: 40, windowX: 30, windowY: 50, paths: [], count: 2 });
    expect("files" in over).toBe(false);

    const file = {
      name: "a.txt",
      size: 5,
      type: "text/plain",
      lastModified: 0,
      text: async () => "hello",
      arrayBuffer: async () => new ArrayBuffer(5),
    };
    const dropped = fileDragEventOf({ clientX: 30, clientY: 50 }, { left: 10, top: 10 }, 1, [file]);
    expect(dropped.paths).toEqual([]);
    expect(dropped.files).toEqual([file]);
    expect(dropped.count).toBe(1);
  });
});
