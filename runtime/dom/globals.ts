declare const console: { error(...args: unknown[]): void };
// Svelte's client runtime reads `document`, `window` and the node constructors
// at module-evaluation time, so this must be the first import in any entry
// point — before anything that pulls in `svelte/internal/client`.
//
// QuickJS is a bare engine: no timers, no animation frames, no `performance`.
// They are built here from a single clock the host pumps with `__pump(now)`.

import "./parse";
import "./web";
import { animating, installAnimate, stepAnimations, useClock } from "./animate";
import { type EventHandler, SComment, SElement, SFragment, SNode, SText } from "./nodes";

/** `window` takes listeners too: the host reports keys and the window's size to it. */
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

export const windowTarget = new EventTargetLite();

const globals = globalThis as any;

const documentElement = new SElement("html");
const body = new SElement("body");
documentElement.appendChild(body);

/** The document, as far as Svelte's runtime reads it. */
const fakeDocument = {
  // `IS_XHTML` is derived from this once, at import time; anything with "xml"
  // in it would switch the runtime to XHTML-compliant tag casing.
  contentType: "text/html",
  get defaultView() {
    return globalThis;
  },
  documentElement,
  body,
  activeElement: null as SNode | null,
  nodeType: 9,
  createElement: (tag: string) => new SElement(tag),
  createElementNS: (_ns: string, tag: string) => new SElement(tag),
  createTextNode: (data = "") => new SText(data),
  createComment: (data = "") => new SComment(data),
  createDocumentFragment: () => new SFragment(),
  importNode: (node: SNode, deep?: boolean) => node.cloneNode(deep),
  addEventListener: () => {},
  removeEventListener: () => {},
};

// Every node belongs to the one document.
Object.defineProperty(SNode.prototype, "ownerDocument", { get: () => fakeDocument, configurable: true });

export { fakeDocument };

interface Timer {
  due: number;
  callback: () => void;
  repeat: number | null;
}

const timers = new Map<number, Timer>();
const frames = new Map<number, (time: number) => void>();
let nextTimer = 1;
let now = 0;

/** The host's clock when it offers one, so events between ticks see the real time. */
export function currentTime(): number {
  const host = (globalThis as any).host;
  if (host !== undefined && typeof host.now === "function") {
    return host.now();
  }
  return now;
}

/**
 * Runs every timer that is due and, on a display frame, the animation frames, as of
 * the host's clock.
 */
export function pump(clock: number, frame: boolean): boolean {
  now = clock;
  let ran = false;
  if (frame) {
    ran = stepAnimations();
    const due = [...frames.values()];
    frames.clear();
    for (const callback of due) {
      ran = true;
      callback(clock);
    }
  }
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

/**
 * When the clock next has work: `0` for the display's next frame (an animation frame
 * or a running animation), the milliseconds until the first timer, or `-1` for never.
 */
export function wake(): number {
  if (frames.size > 0 || animating()) {
    return 0;
  }
  let first = Infinity;
  for (const timer of timers.values()) {
    first = Math.min(first, timer.due);
  }
  return first === Infinity ? -1 : Math.max(first - currentTime(), 0.5);
}

function install(): void {
  if (globals.document !== undefined) {
    return;
  }
  installAnimate();
  useClock(currentTime);

  globals.document = fakeDocument;
  globals.navigator = { userAgent: "svelte-gpui" };
  globals.Node = SNode;
  globals.Element = SElement;
  globals.HTMLElement = SElement;
  globals.Text = SText;
  globals.Comment = SComment;
  globals.DocumentFragment = SFragment;
  // `set_custom_element_data` branches on this being present; returning
  // undefined keeps it on the plain setAttribute path.
  globals.customElements = { get: () => undefined, define: () => {} };
  globals.Event = HostEvent;
  globals.CustomEvent = HostEvent;
  globals.queueMicrotask = (fn: () => void) => {
    void Promise.resolve().then(fn).catch(reportError);
  };
  globals.setTimeout = (fn: () => void, delay = 0) => {
    const handle = nextTimer++;
    timers.set(handle, { due: currentTime() + delay, callback: fn, repeat: null });
    return handle;
  };
  globals.setInterval = (fn: () => void, delay = 0) => {
    const handle = nextTimer++;
    timers.set(handle, { due: currentTime() + delay, callback: fn, repeat: Math.max(delay, 1) });
    return handle;
  };
  globals.clearTimeout = (handle: number) => {
    timers.delete(handle);
  };
  globals.clearInterval = globals.clearTimeout;
  globals.requestAnimationFrame = (fn: (time: number) => void) => {
    const handle = nextTimer++;
    frames.set(handle, fn);
    return handle;
  };
  globals.cancelAnimationFrame = (handle: number) => {
    frames.delete(handle);
  };
  globals.performance = { now: () => currentTime() };

  globals.window = globals;
  Object.assign(globals, {
    trustedTypes: undefined,
    addEventListener: (type: string, handler: EventHandler) =>
      windowTarget.addEventListener(type, handler),
    removeEventListener: (type: string, handler: EventHandler) =>
      windowTarget.removeEventListener(type, handler),
    dispatchEvent: () => true,
    getComputedStyle: (element: SElement) => computedStyle(element),
    // `prefersReducedMotion` in svelte/motion asks this; nothing here ever matches.
    matchMedia: () => ({ matches: false, addEventListener: () => {}, removeEventListener: () => {} }),
    innerWidth: 0,
    innerHeight: 0,
    devicePixelRatio: 1,
  });
}

/** `Event` and `CustomEvent`, which Svelte makes for `introstart` and its kin. */
class HostEvent {
  target: unknown = null;
  currentTarget: unknown = null;
  defaultPrevented = false;
  cancelBubble = false;
  readonly bubbles: boolean;
  readonly cancelable: boolean;
  readonly detail: unknown;

  constructor(
    readonly type: string,
    init: { bubbles?: boolean; cancelable?: boolean; detail?: unknown } = {},
  ) {
    this.bubbles = init.bubbles === true;
    this.cancelable = init.cancelable === true;
    this.detail = init.detail ?? null;
  }

  preventDefault(): void {
    this.defaultPrevented = true;
  }

  stopPropagation(): void {
    this.cancelBubble = true;
  }

  stopImmediatePropagation(): void {
    this.cancelBubble = true;
  }
}

/** `getComputedStyle`: the inline declarations, and the sizes the host last measured. */
function computedStyle(element: SElement): Record<string, unknown> {
  const read = (name: string): string => {
    const inline = element.style.getPropertyValue(name);
    if (inline !== "") {
      return inline;
    }
    if (name === "height") {
      return `${element.offsetHeight}px`;
    }
    if (name === "width") {
      return `${element.offsetWidth}px`;
    }
    return "";
  };
  const camel = (name: string): string => read(name.replace(/[A-Z]/g, (letter) => `-${letter.toLowerCase()}`));
  return new Proxy(
    { getPropertyValue: (name: string) => read(name) },
    {
      get(target, property) {
        if (property in target) {
          return (target as any)[property];
        }
        if (typeof property !== "string") {
          return undefined;
        }
        const value = camel(property);
        if (value === "" && /^(padding|margin|border)/.test(property)) {
          return "0px";
        }
        if (value === "" && property === "opacity") {
          return "1";
        }
        return value;
      },
    },
  );
}

function reportError(error: unknown): void {
  const stack = error instanceof Error ? error.stack : undefined;
  console.error("svelte-gpui: microtask failed:", error, stack === undefined ? "" : `\n${stack}`);
}

install();
