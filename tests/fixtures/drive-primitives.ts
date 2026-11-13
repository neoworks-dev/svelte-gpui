// Runs the app in primitives-app.ts the way the host does, and prints the operations it sent after each step as
// JSON, for tests/primitives.test.ts. It runs in a process of its own: the bundle brings its own copy of the
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

const outfile = join(mkdtempSync(join(tmpdir(), "svelte-gpui-primitives-")), "app.js");
await build({ entry: new URL("./primitives-app.ts", import.meta.url).pathname, outfile, sourcemap: false });
const log: Record<string, unknown> = {};
await import(outfile);
const page = globals.__page;
const mounted = sent();
log.mounted = mounted;

const ofTag = (tag: string): number => mounted.find((op) => op[0] === 0 && op[2] === tag)![1] as number;
const view = ofTag("g-view");
const button = ofTag("g-button");
const image = ofTag("g-image");
log.ids = { view, button, image };

page.setWidth(120);
globals.__settle();
log.afterWidth = sent();

page.setPress(false);
globals.__settle();
log.afterPressOff = sent();
log.pressOffReply = JSON.parse(globals.__event(view, "press", "{}"));
log.pressedAfterOff = page.pressed();

page.setPress(true);
globals.__settle();
log.afterPressOn = sent();
globals.__event(view, "press", "{}");
log.pressedAfterOn = page.pressed();

globals.__event(button, "press", "{}");
log.pressedAfterButton = page.pressed();

page.setDisabled(true);
globals.__settle();
log.afterDisabled = sent();

page.setDisabled(false);
globals.__settle();
log.afterEnabled = sent();

globals.__event(button, "hover", JSON.stringify({ hovered: true }));
globals.__event(image, "load", "{}");
log.loaded = page.loaded();

page.setSize(24);
globals.__settle();
log.afterSize = sent();

page.focusView();
globals.__settle();
log.afterFocus = sent();

page.hideButton();
globals.__settle();
log.afterHide = sent();

process.stdout.write(`\n@@${JSON.stringify(log)}\n`);
