import { beforeAll, describe, expect, test } from "bun:test";
import protocol from "../protocol.json";

const code = (name: string): number => protocol.operations.indexOf(name);
const CREATE_TEXT = code("createText");
const SET_TEXT = code("setText");
const SET_ATTRIBUTE = code("setAttribute");
const LISTEN = code("listen");
const CALL = code("call");

type Operation = unknown[];
interface Run {
  mounted: Operation[];
  afterProp: Operation[];
  afterEvent: Operation[];
  call: Operation[];
  afterAnswer: Operation[];
  afterRefusal: Operation[];
}

let run: Run;
let thing: number;

/** Whether some text node was told to show text that starts with this. */
function shows(operations: Operation[], text: string): boolean {
  return operations.some((op) => (op[0] === SET_TEXT || op[0] === CREATE_TEXT) && String(op[2]).startsWith(text));
}

beforeAll(async () => {
  const driver = new URL("./fixtures/drive-element.ts", import.meta.url).pathname;
  const child = Bun.spawn(["bun", driver], { stdout: "pipe", stderr: "pipe" });
  const [output, errors] = await Promise.all([new Response(child.stdout).text(), new Response(child.stderr).text()]);
  expect(await child.exited, errors).toBe(0);
  run = JSON.parse(output.slice(output.lastIndexOf("@@") + 2));
  thing = run.mounted.find((op) => op[0] === code("createElement") && op[2] === "g-element")![1] as number;
});

describe("element()", () => {
  test("draws one host tag, named for the element, with its key and props as JSON", () => {
    expect(run.mounted).toContainEqual([SET_ATTRIBUTE, thing, "name", "thing"]);
    expect(run.mounted).toContainEqual([SET_ATTRIBUTE, thing, "key", "main"]);
    expect(run.mounted).toContainEqual([SET_ATTRIBUTE, thing, "prop:path", '"/a"']);
  });

  test("listens for an event the view emits under app: and its name", () => {
    expect(run.mounted).toContainEqual([LISTEN, thing, "app:save"]);
  });

  test("sends a prop again when the app changes it", () => {
    expect(run.afterProp).toContainEqual([SET_ATTRIBUTE, thing, "prop:path", '"/b"']);
  });

  test("hands the listener what the view emitted", () => {
    expect(shows(run.afterEvent, "saved 7")).toBe(true);
  });

  test("calls a method of the view with its arguments as JSON", () => {
    const call = run.call.find((op) => op[0] === CALL)!;
    expect(call[1]).toBe(thing);
    expect(call[3]).toBe("insert");
    expect(JSON.parse(call[4] as string)).toEqual(["hello"]);
  });

  test("resolves the call with the view's answer", () => {
    expect(shows(run.afterAnswer, "inserted")).toBe(true);
  });

  test("rejects the call with the view's message", () => {
    expect(shows(run.afterRefusal, "failed: no such method")).toBe(true);
  });
});
