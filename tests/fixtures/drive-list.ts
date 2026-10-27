// Runs the app in list-app.ts the way the host does, with a host that lays rows out at heights of its own, and
// prints what the lists built and what the app asked of the host as JSON, for tests/list.test.ts. It runs in a
// process of its own: the bundle brings its own copy of the runtime, which must not meet the one other tests install
// their globals from.

import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { build } from "../../build.mjs";
import protocol from "../../protocol.json";

const code = (name: string): number => protocol.operations.indexOf(name);
const [CREATE_ELEMENT, CREATE_TEXT, SET_TEXT, SET_ATTRIBUTE, INSERT, REMOVE, LISTEN, DROP, SCROLL_TO] = [
  "createElement", "createText", "setText", "setAttribute", "insert", "remove", "listen", "drop", "scrollTo",
].map(code);

interface Node {
  id: number;
  tag: string;
  text: string;
  attributes: Record<string, unknown>;
  listeners: Set<string>;
  children: Node[];
}

/** The host's root is node 1 before the app says anything. */
const nodes = new Map<number, Node>([[1, { id: 1, tag: "#root", text: "", attributes: {}, listeners: new Set(), children: [] }]]);
let scrollRequests = 0;
/** The offset the app asked for last. */
let lastScrollOffset = 0;

/** Applies a batch of the app's operations to a copy of the tree, as the host does. */
function apply(batch: string): void {
  for (const operation of JSON.parse(batch) as any[][]) {
    const [kind, id] = operation;
    if (kind === CREATE_ELEMENT || kind === CREATE_TEXT) {
      nodes.set(id, { id, tag: kind === CREATE_TEXT ? "#text" : operation[2], text: kind === CREATE_TEXT ? operation[2] : "", attributes: {}, listeners: new Set(), children: [] });
    } else if (kind === SET_TEXT) {
      nodes.get(id)!.text = operation[2];
    } else if (kind === SET_ATTRIBUTE) {
      nodes.get(id)!.attributes[operation[2]] = operation[3];
    } else if (kind === LISTEN) {
      nodes.get(id)?.listeners.add(operation[2]);
    } else if (kind === INSERT) {
      const [, parent, child, before] = operation;
      for (const holder of nodes.values()) {
        holder.children = holder.children.filter((node) => node.id !== child);
      }
      const children = nodes.get(parent)!.children;
      const at = before === 0 ? children.length : children.findIndex((node) => node.id === before);
      children.splice(at < 0 ? children.length : at, 0, nodes.get(child)!);
    } else if (kind === REMOVE) {
      for (const holder of nodes.values()) {
        holder.children = holder.children.filter((node) => node.id !== id);
      }
    } else if (kind === DROP) {
      nodes.delete(id);
    } else if (kind === SCROLL_TO) {
      scrollRequests++;
      lastScrollOffset = operation[3];
    }
  }
}

const globals = globalThis as any;
globals.host = {
  apply,
  io: () => {},
  fieldState: () => [],
  frameStats: () => [0, 0],
  clipboardText: () => "",
};

const outfile = join(mkdtempSync(join(tmpdir(), "svelte-gpui-list-")), "app.js");
await build({ entry: new URL("./list-app.ts", import.meta.url).pathname, outfile, sourcemap: false });
await import(outfile);

/** The text under a node. */
function textOf(node: Node): string {
  return node.tag === "#text" ? node.text : node.children.map(textOf).join("");
}

/** The lists' scrollers, in the order the page draws them. */
function scrollers(): Node[] {
  return [...nodes.values()].filter((node) => node.tag === "g-scroll").sort((a, b) => a.id - b.id);
}

/** What a list has built: the rows' texts in order, and the heights of the two spacers around them. */
function built(scroller: Node): { rows: string[]; above: number; below: number; anchor: unknown } {
  const height = (node: Node): number => JSON.parse(String(node.attributes.look ?? "{}")).height ?? 0;
  const children = scroller.children.filter((node) => node.tag === "g-view");
  const rows = children.slice(1, -1);
  return {
    rows: rows.map(textOf),
    above: height(children[0]),
    below: height(children[children.length - 1]),
    anchor: scroller.attributes.anchor ?? null,
  };
}

/** Lays out the rows a list has built at heights of their own, as the host would, and tells the app. */
function layOut(scroller: Node, heightOf: (text: string) => number): void {
  for (const row of scroller.children.slice(1, -1)) {
    if (row.listeners.has("resize")) {
      globals.__event(row.id, "resize", JSON.stringify({ x: 0, y: 0, width: 400, height: heightOf(textOf(row)) }));
    }
  }
}

const log: Record<string, unknown> = {};
const [chat, plain, fixed, tall] = scrollers();
globals.__event(chat.id, "layout", JSON.stringify({ x: 0, y: 0, width: 400, height: 600 }));

log.opened = { chat: built(chat), plain: built(plain), fixed: built(fixed), scrollRequests };

const heightOf = (text: string): number => 80 + (Number(text.split(" ")[1]) % 3) * 40;
layOut(chat, heightOf);
log.measured = { chat: built(chat), scrollRequests };

globals.__page.append();
globals.__settle();
layOut(chat, heightOf);
log.appended = { chat: built(chat), scrollRequests };

// The user scrolls up: the host reports where the window is, short of the end.
globals.__event(chat.id, "scroll", JSON.stringify({ x: 0, y: 200000, maxX: 0, maxY: 400000 }));
layOut(chat, heightOf);
log.scrolledUp = { chat: built(chat), scrollRequests };

globals.__page.append();
globals.__settle();
log.appendedWhileUp = { chat: built(chat), scrollRequests };

// And back to the end, which the host pins again.
globals.__event(chat.id, "scroll", JSON.stringify({ x: 0, y: 400000, maxX: 0, maxY: 400000 }));
layOut(chat, heightOf);
log.backAtEnd = { chat: built(chat), scrollRequests };

// Scrolling up again, then asking for the end: the app asks the host for the end itself, however far that turns out to be.
globals.__event(chat.id, "scroll", JSON.stringify({ x: 0, y: 100000, maxX: 0, maxY: 400000 }));
layOut(chat, heightOf);
const requestsBefore = scrollRequests;
globals.__page.toEnd();
globals.__settle();
layOut(chat, heightOf);
log.toEnd = { chat: built(chat), scrollRequests: scrollRequests - requestsBefore, offset: lastScrollOffset };

// Rows taller than the window, each measured as it is built, until the list builds no new ones.
globals.__event(tall.id, "layout", JSON.stringify({ x: 0, y: 0, width: 400, height: 600 }));
for (let pass = 0; pass < 4; pass++) {
  layOut(tall, () => 700);
}
log.tall = built(tall);

process.stdout.write(`\n@@${JSON.stringify(log)}\n`);
