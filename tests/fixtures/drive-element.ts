// Runs the app in element-app.ts the way the host does, and prints the operations it sent after each step as
// JSON, for tests/element.test.ts. It runs in a process of its own: the bundle brings its own copy of the
// runtime, which must not meet the one other tests install their globals from.

import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { build } from "../../build.mjs";

const globals = globalThis as any;
const operations: unknown[][] = [];
globals.host = {
  apply: (batch: string) => operations.push(...JSON.parse(batch)),
  io: () => {},
  fieldState: () => [],
  frameStats: () => [0, 0],
  clipboardText: () => "",
};

/** The operations sent since the last time this was asked. */
function sent(): unknown[][] {
  return operations.splice(0, operations.length);
}

const outfile = join(mkdtempSync(join(tmpdir(), "svelte-gpui-element-")), "app.js");
await build({ entry: new URL("./element-app.ts", import.meta.url).pathname, outfile, sourcemap: false });
// The build reports on the console; the output of this script is one JSON document.
const log: Record<string, unknown> = {};
await import(outfile);
const mounted = sent();
log.mounted = mounted;
const thing = mounted.find((op) => op[0] === 0 && op[2] === "g-element")![1] as number;

globals.__page.setPath("/b");
globals.__settle();
log.afterProp = sent();

globals.__event(thing, "app:save", JSON.stringify({ detail: { count: 7 } }));
log.afterEvent = sent();

const answered = globals.__page.insert("hello");
globals.__settle();
const asked = sent();
log.call = asked;
const callNumber = asked.find((op) => op[1] === thing && op[3] === "insert")![2];
globals.__io(callNumber, "resolve", '"inserted"', undefined);
await answered;
globals.__settle();
log.afterAnswer = sent();

const refused = globals.__page.insert("oops");
globals.__settle();
const secondCall = sent().find((op) => op[1] === thing && op[3] === "insert")![2];
globals.__io(secondCall, "reject", JSON.stringify({ message: "no such method", code: "UnknownMethod" }), undefined);
await refused;
globals.__settle();
log.afterRefusal = sent();

process.stdout.write(`\n@@${JSON.stringify(log)}\n`);
