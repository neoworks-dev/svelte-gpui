declare const console: { error(...args: unknown[]): void };
// Svelte's client runtime reads `document`, `window` and the node constructors
// at module-evaluation time, so this must be the first import in any entry
// point — before anything that pulls in `svelte/internal/client`.
//
// QuickJS is a bare engine: no timers, no animation frames, no `performance`.
// They are built here from a single clock the host pumps with `__pump(now)`.

import "./parse";
import {
  type EventHandler,
  SComment,
  SElement,
  SFragment,
  SNode,
  SText,
} from "./nodes";

/** `document` and `window` take listeners too; the host reports events to them. */
export class EventTargetLite {
  readonly listeners = new Map<string, Set<EventHandler>>();

  addEventListener(type: string, handler: EventHandler): void {
    let handlers = this.listeners.get(type);
    if (handlers === undefined) {
      handlers = new Set();
      this.listeners.set(type, handlers);
    }
    handlers.add(handler);
  }

  removeEventListener(type: string, handler: EventHandler): void {
    this.listeners.get(type)?.delete(handler);
  }

  dispatchEvent(): boolean {
    return true;
  }
}

export const documentTarget = new EventTargetLite();
export const windowTarget = new EventTargetLite();

const globals = globalThis as any;

class SMediaElement extends SElement {}

const documentElement = new SElement("html");
const head = new SElement("head");
const body = new SElement("body");
documentElement.appendChild(head);
documentElement.appendChild(body);

const fakeDocument = {
  // `IS_XHTML` is derived from this once, at import time; anything with "xml"
  // in it would switch the runtime to XHTML-compliant tag casing.
  contentType: "text/html",
  baseURI: "file:///",
  documentElement,
  head,
  body,
  activeElement: null as SNode | null,
  nodeType: 9,
  createElement: (tag: string) => new SElement(tag),
  createElementNS: (_ns: string, tag: string) => new SElement(tag),
  createTextNode: (data = "") => new SText(data),
  createComment: (data = "") => new SComment(data),
  createDocumentFragment: () => new SFragment(),
  importNode: (node: SNode, deep?: boolean) => node.cloneNode(deep),
  addEventListener: (type: string, handler: EventHandler) =>
    documentTarget.addEventListener(type, handler),
  removeEventListener: (type: string, handler: EventHandler) =>
    documentTarget.removeEventListener(type, handler),
  dispatchEvent: () => true,
};

export { fakeDocument };

interface Timer {
  due: number;
  callback: () => void;
  repeat: number | null;
}

const timers = new Map<number, Timer>();
let nextTimer = 1;
let now = 0;

/** Runs every timer and animation frame that is due, as of the host's clock. */
export function pump(clock: number): boolean {
  now = clock;
  let ran = false;
  for (const [handle, timer] of [...timers]) {
    if (timer.due > clock) {
      continue;
    }
    ran = true;
    if (timer.repeat === null) {
      timers.delete(handle);
    } else {
      timer.due = clock + timer.repeat;
    }
    timer.callback();
  }
  return ran;
}

function install(): void {
  if (globals.document !== undefined) {
    return;
  }

  globals.document = fakeDocument;
  globals.navigator = { userAgent: "svelte-gpui" };
  globals.Node = SNode;
  globals.Element = SElement;
  globals.HTMLElement = SElement;
  globals.HTMLMediaElement = SMediaElement;
  globals.Text = SText;
  globals.Comment = SComment;
  globals.DocumentFragment = SFragment;
  // `set_custom_element_data` branches on this being present; returning
  // undefined keeps it on the plain setAttribute path.
  globals.customElements = { get: () => undefined, define: () => {} };
  globals.MutationObserver = class {
    observe(): void {}
    disconnect(): void {}
    takeRecords(): unknown[] {
      return [];
    }
  };
  globals.ResizeObserver = class {
    observe(): void {}
    unobserve(): void {}
    disconnect(): void {}
  };

  globals.queueMicrotask = (fn: () => void) => {
    void Promise.resolve().then(fn).catch(reportError);
  };
  globals.setTimeout = (fn: () => void, delay = 0) => {
    const handle = nextTimer++;
    timers.set(handle, { due: now + delay, callback: fn, repeat: null });
    return handle;
  };
  globals.setInterval = (fn: () => void, delay = 0) => {
    const handle = nextTimer++;
    timers.set(handle, { due: now + delay, callback: fn, repeat: Math.max(delay, 1) });
    return handle;
  };
  globals.clearTimeout = (handle: number) => {
    timers.delete(handle);
  };
  globals.clearInterval = globals.clearTimeout;
  globals.requestAnimationFrame = (fn: (time: number) => void) => {
    const handle = nextTimer++;
    timers.set(handle, { due: now, callback: () => fn(now), repeat: null });
    return handle;
  };
  globals.cancelAnimationFrame = globals.clearTimeout;
  globals.performance = { now: () => now };

  globals.window = globals;
  Object.assign(globals, {
    trustedTypes: undefined,
    addEventListener: (type: string, handler: EventHandler) =>
      windowTarget.addEventListener(type, handler),
    removeEventListener: (type: string, handler: EventHandler) =>
      windowTarget.removeEventListener(type, handler),
    dispatchEvent: () => true,
    getComputedStyle: () => ({ getPropertyValue: () => "" }),
    matchMedia: () => ({
      matches: false,
      addEventListener: () => {},
      removeEventListener: () => {},
    }),
    getSelection: () => null,
    scrollTo: () => {},
    innerWidth: 0,
    innerHeight: 0,
    devicePixelRatio: 1,
  });
}

function reportError(error: unknown): void {
  console.error("svelte-gpui: microtask failed:", error);
}

install();
