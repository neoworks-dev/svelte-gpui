// Keeps the host's tree in step with the node tree Svelte mutates, by sending
// only what changed. Every element and text node has an id; a comment has none,
// because the host draws nothing for it. Operations are collected as the
// runtime works and sent as one batch when it has finished.

import protocol from "../../protocol.json";
import { setMirror } from "../dom/hooks";
import { fakeDocument, windowTarget } from "../dom/globals";
import { handBackFocus } from "../dom/focus";
import {
  type EventHandler,
  COMMENT_NODE,
  SElement,
  SNode,
  SText,
  setBackend,
} from "../dom/nodes";

declare const console: { error(...args: unknown[]): void };
declare const host: { apply(batch: string): void; fieldState(id: number): number[] };

// Operation codes: an operation's index in protocol.json, which the host's build reads too.
const code = (name: string): number => protocol.operations.indexOf(name);
const CREATE_ELEMENT = code("createElement");
const CREATE_TEXT = code("createText");
const SET_TEXT = code("setText");
const SET_ATTRIBUTE = code("setAttribute");
const INSERT = code("insert");
const REMOVE = code("remove");
const LISTEN = code("listen");
const UNLISTEN = code("unlisten");
const SET_STYLE = code("setStyle");
const DROP = code("drop");
const FOCUS = code("focus");
const BLUR = code("blur");
const SCROLL_TO = code("scrollTo");
const SELECT = code("select");
const WRITE_CLIPBOARD = code("writeClipboard");
const DRAW = code("draw");

export const ROOT_ID = 1;

const nodes = new Map<number, SNode>();
let nextId = ROOT_ID + 1;
let pending: unknown[][] = [];
/** Nodes taken out of the tree during this batch; the ones still out at the end are gone. */
let detached = new Set<SNode>();

/** Sends a drawing command to a `g-canvas`, which the host paints into its pixels. */
export function draw(node: SNode, command: unknown[]): void {
  if (node.hostId === 0) {
    return;
  }
  pending.push([DRAW, node.hostId, command]);
}

export function nodeFor(id: number): SNode | undefined {
  return nodes.get(id);
}

/** Makes `root` the host's root node, id 1. */
export function adoptRoot(root: SElement): void {
  root.hostId = ROOT_ID;
  nodes.set(ROOT_ID, root);
}

function register(node: SNode): number {
  node.hostId = nextId++;
  nodes.set(node.hostId, node);
  return node.hostId;
}

/** Sends everything collected so far, and forgets the nodes that were removed for good. */
export function flush(): void {
  for (const node of detached) {
    if (node.parentNode === null) {
      pending.push([DROP, node.hostId]);
      forget(node);
    }
  }
  detached = new Set();
  if (pending.length === 0) {
    return;
  }
  const batch = pending;
  pending = [];
  host.apply(JSON.stringify(batch));
}

function forget(node: SNode): void {
  nodes.delete(node.hostId);
  let child = node.firstChildNode;
  while (child !== null) {
    if (child.hostId !== 0) {
      forget(child);
    }
    child = child.nextSiblingNode;
  }
}

/**
 * Announces a node that was built without being announced, with everything under
 * it. Markup parsed for `{@html}` and `innerHTML` is built that way, since the
 * parser also builds the moulds Svelte clones, and a mould is never drawn.
 */
function announce(node: SNode): void {
  if (node.hostId !== 0 || node.nodeType === COMMENT_NODE) {
    return;
  }
  if (node instanceof SText) {
    const id = register(node);
    pending.push([CREATE_TEXT, id, node.data]);
    return;
  }
  const element = node as SElement;
  if (element.localName === "template") {
    return;
  }
  const id = register(element);
  pending.push([CREATE_ELEMENT, id, element.localName]);
  for (const [name, value] of element.attributes) {
    pending.push([SET_ATTRIBUTE, id, name, normalize(value)]);
  }
  for (const [name, value] of element.style.declarations) {
    pending.push([SET_STYLE, id, name, value]);
  }
  for (const type of element.listeners.keys()) {
    pending.push([LISTEN, id, type]);
  }
  for (let child = element.firstChildNode; child !== null; child = child.nextSiblingNode) {
    announce(child);
    if (child.hostId !== 0) {
      pending.push([INSERT, id, child.hostId, 0]);
    }
  }
}

/** The next sibling the host knows about, skipping comment anchors. */
function hostSiblingAfter(ref: SNode | null): number {
  let sibling = ref;
  while (sibling !== null) {
    if (sibling.hostId !== 0) {
      return sibling.hostId;
    }
    sibling = sibling.nextSiblingNode;
  }
  return 0;
}

/** Attribute values cross as JSON: strings, numbers and booleans, or null for removed. */
function normalize(value: unknown): unknown {
  if (value === null || value === undefined) {
    return null;
  }
  if (typeof value === "string" || typeof value === "number" || typeof value === "boolean") {
    return value;
  }
  return String(value);
}

export function install(): void {
  setBackend({
    created(node: SNode): void {
      const id = register(node);
      if (node instanceof SText) {
        pending.push([CREATE_TEXT, id, node.data]);
        return;
      }
      pending.push([CREATE_ELEMENT, id, (node as SElement).localName]);
    },

    attributeChanged(node: SElement, name: string, value: unknown): void {
      if (node.hostId === 0) {
        return;
      }
      pending.push([SET_ATTRIBUTE, node.hostId, name, normalize(value)]);
    },

    styleChanged(node: SElement, property: string, value: string | null): void {
      if (node.hostId === 0) {
        return;
      }
      pending.push([SET_STYLE, node.hostId, property, value]);
    },

    listenerAdded(node: SElement, type: string, _handler: EventHandler): void {
      if (node.hostId === 0) {
        return;
      }
      pending.push([LISTEN, node.hostId, type]);
    },

    listenerRemoved(node: SElement, type: string): void {
      if (node.hostId === 0 || (node.listeners.get(type)?.size ?? 0) > 0) {
        return;
      }
      pending.push([UNLISTEN, node.hostId, type]);
    },

    focus(node: SElement): void {
      if (node.hostId === 0) {
        return;
      }
      pending.push([FOCUS, node.hostId]);
    },

    blur(node: SElement): void {
      if (node.hostId === 0) {
        return;
      }
      const back = handBackFocus(node);
      if (back !== null) {
        back.focus();
      } else if (fakeDocument.activeElement === node) {
        pending.push([BLUR, node.hostId]);
      }
    },

    fieldState(node: SElement): number[] {
      return node.hostId === 0 ? [] : host.fieldState(node.hostId);
    },

    scrollTo(node: SElement, left: number, top: number): void {
      if (node.hostId === 0) {
        return;
      }
      pending.push([SCROLL_TO, node.hostId, left, top]);
    },

    select(node: SElement, start: number, end: number): void {
      if (node.hostId === 0) {
        return;
      }
      pending.push([SELECT, node.hostId, start, end]);
    },
  });

  // The host reports keys to `window` while it has a listener, under id 0.
  const add = windowTarget.addEventListener.bind(windowTarget);
  const remove = windowTarget.removeEventListener.bind(windowTarget);
  windowTarget.addEventListener = (type: string, handler: EventHandler): void => {
    add(type, handler);
    pending.push([LISTEN, 0, type]);
  };
  windowTarget.removeEventListener = (type: string, handler: EventHandler): void => {
    remove(type, handler);
    if ((windowTarget.listeners.get(type)?.size ?? 0) === 0) {
      pending.push([UNLISTEN, 0, type]);
    }
  };

  // `navigator.clipboard` reads what the host last saw and writes through it.
  (globalThis as any).navigator.clipboard = {
    writeText(text: string): Promise<void> {
      pending.push([WRITE_CLIPBOARD, 0, String(text)]);
      return Promise.resolve();
    },
    readText(): Promise<string> {
      return Promise.resolve((globalThis as any).host.clipboardText());
    },
  };

  setMirror({
    inserted(child: SNode, before: SNode | null): void {
      detached.delete(child);
      if (child.nodeType === COMMENT_NODE) {
        return;
      }
      const parent = child.parentNode;
      if (parent === null || parent.hostId === 0) {
        return;
      }
      announce(child);
      if (child.hostId === 0) {
        return;
      }
      pending.push([INSERT, parent.hostId, child.hostId, hostSiblingAfter(before)]);
    },

    removed(child: SNode, formerParent: SNode): void {
      if (child.hostId === 0 || formerParent.hostId === 0) {
        return;
      }
      pending.push([REMOVE, child.hostId]);
      detached.add(child);
    },

    textChanged(node: SText): void {
      if (node.hostId === 0) {
        return;
      }
      pending.push([SET_TEXT, node.hostId, node.data]);
    },
  });
}
