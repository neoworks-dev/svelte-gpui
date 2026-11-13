import { beforeAll, describe, expect, test } from "bun:test";
import protocol from "../protocol.json";

const code = (name: string): number => protocol.operations.indexOf(name);
const CREATE_ELEMENT = code("createElement");
const SET_ATTRIBUTE = code("setAttribute");
const INSERT = code("insert");
const LISTEN = code("listen");
const UNLISTEN = code("unlisten");
const FOCUS = code("focus");
const REMOVE = code("remove");
const DROP = code("drop");

type Operation = unknown[];

interface Run {
  mounted: Operation[];
  ids: { view: number; button: number; image: number };
  afterWidth: Operation[];
  afterPressOff: Operation[];
  pressedAfterOff: number;
  afterPressOn: Operation[];
  pressedAfterOn: number;
  pressedAfterButton: number;
  afterDisabled: Operation[];
  afterEnabled: Operation[];
  loaded: number;
  afterSize: Operation[];
  afterFocus: Operation[];
  afterHide: Operation[];
}

let run: Run;

beforeAll(async () => {
  const driver = new URL("./fixtures/drive-primitives.ts", import.meta.url).pathname;
  const child = Bun.spawn(["bun", driver], { stdout: "pipe", stderr: "pipe" });
  const [output, errors] = await Promise.all([new Response(child.stdout).text(), new Response(child.stderr).text()]);
  expect(await child.exited, errors).toBe(0);
  run = JSON.parse(output.slice(output.lastIndexOf("@@") + 2));
});

describe("a primitive", () => {
  test("is one host element with the attributes its props give, and none for the props it was not given", () => {
    const { view } = run.ids;
    expect(run.mounted).toContainEqual([SET_ATTRIBUTE, view, "look", '{"width":100}']);
    expect(run.mounted).toContainEqual([SET_ATTRIBUTE, view, "hover", '{"background":"#fff"}']);
    expect(run.mounted).toContainEqual([SET_ATTRIBUTE, view, "focusable", ""]);
    const names = run.mounted.filter((op) => op[0] === SET_ATTRIBUTE && op[1] === view).map((op) => op[2]);
    expect(names).toEqual(["look", "hover", "focusable"]);
  });

  test("builds its children inside itself, and sits in its parent after them", () => {
    const { view, button } = run.ids;
    const insertions = run.mounted.filter((op) => op[0] === INSERT);
    expect(insertions).toContainEqual([INSERT, view, button, 0]);
    const created = run.mounted.findIndex((op) => op[0] === CREATE_ELEMENT && op[1] === button);
    const insertedIntoParent = run.mounted.findIndex((op) => op[0] === INSERT && op[2] === view);
    expect(insertedIntoParent).toBeGreaterThan(created);
  });

  test("writes the other primitives' own attributes: an icon's size and colour, an image's source and fit, an overlay's placement", () => {
    const attributes = (tag: string): Record<string, unknown> => {
      const id = run.mounted.find((op) => op[0] === CREATE_ELEMENT && op[2] === tag)![1];
      return Object.fromEntries(run.mounted.filter((op) => op[0] === SET_ATTRIBUTE && op[1] === id).map((op) => [op[2], op[3]]));
    };
    expect(attributes("g-icon")).toEqual({ svg: "<svg/>", look: '{"size":16,"color":"#abc"}' });
    expect(attributes("g-image")).toEqual({ src: "cover.png", fit: "cover" });
    expect(attributes("g-overlay")).toEqual({ side: "top", offset: "4", at: "1,2", "match-width": "" });
    expect(attributes("g-text")).toEqual({ look: '{"fontSize":12}', pre: "" });
  });

  test("listens for the events whose props it was given, and only those", () => {
    const { view, button, image } = run.ids;
    const listened = (id: number) => run.mounted.filter((op) => op[0] === LISTEN && op[1] === id).map((op) => op[2]);
    expect(listened(view)).toEqual(["press"]);
    expect(listened(button)).toEqual(["press", "hover"]);
    expect(listened(image)).toEqual(["load"]);
  });

  test("changes one attribute when one prop changes", () => {
    expect(run.afterWidth).toEqual([[SET_ATTRIBUTE, run.ids.view, "look", '{"width":120}']]);
    const icon = run.mounted.find((op) => op[0] === CREATE_ELEMENT && op[2] === "g-icon")![1];
    expect(run.afterSize).toEqual([[SET_ATTRIBUTE, icon, "look", '{"size":24,"color":"#abc"}']]);
  });

  test("stops listening when the prop goes, and listens again when it comes back", () => {
    const { view } = run.ids;
    expect(run.afterPressOff).toEqual([[UNLISTEN, view, "press"]]);
    expect(run.pressedAfterOff).toBe(0);
    expect(run.afterPressOn).toEqual([[LISTEN, view, "press"]]);
    expect(run.pressedAfterOn).toBe(1);
  });

  test("calls the callback it has now, not the one it had when it was made", () => {
    expect(run.pressedAfterButton).toBe(2);
  });

  test("a disabled button takes its press listener away and lays its disabled style over its own", () => {
    const { button } = run.ids;
    expect(run.afterDisabled).toContainEqual([SET_ATTRIBUTE, button, "disabled", ""]);
    expect(run.afterDisabled).toContainEqual([SET_ATTRIBUTE, button, "look", '{"cursor":"default","opacity":0.5}']);
    expect(run.afterDisabled).toContainEqual([UNLISTEN, button, "press"]);
    expect(run.afterEnabled).toContainEqual([LISTEN, button, "press"]);
    expect(run.afterEnabled).toContainEqual([SET_ATTRIBUTE, button, "disabled", null]);
  });

  test("hands a hover callback whether the pointer is over it, and a load callback nothing", () => {
    expect(run.loaded).toBe(2);
  });

  test("gives bind:this the element's focus", () => {
    expect(run.afterFocus).toEqual([[FOCUS, run.ids.view]]);
  });

  test("takes its element out when it goes, and the host drops it", () => {
    const { button } = run.ids;
    expect(run.afterHide).toContainEqual([REMOVE, button]);
    expect(run.afterHide).toContainEqual([DROP, button]);
  });
});
