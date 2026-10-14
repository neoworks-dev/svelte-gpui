// Turns an event the host reports into the DOM event Svelte expects, and walks
// it up the tree. Svelte delegates the usual events (`click`, `input`, ...) to a
// listener on the mount root, which reads `composedPath()`; anything else sits
// on its own element. After the elements come `document` and `window`.

import { documentTarget, fakeDocument, windowTarget } from "../dom/globals";
import { type EventHandler, SElement, SNode } from "../dom/nodes";
import { nodeFor } from "./bridge";

interface HostEvent {
  type: string;
  target: unknown;
  currentTarget: unknown;
  bubbles: boolean;
  cancelBubble: boolean;
  defaultPrevented: boolean;
  composedPath(): unknown[];
  stopPropagation(): void;
  stopImmediatePropagation(): void;
  preventDefault(): void;
  [key: string]: unknown;
}

/**
 * Delivers an event of `type` to the node with `id`, or to the document when
 * `id` is 0, and lets it bubble through the root, `document` and `window`.
 */
export interface Outcome {
  /** A handler called `preventDefault`, so the host must not do its default action. */
  prevented: boolean;
  /** Text a clipboard handler put on the clipboard, for the host to write. */
  clipboard: string | null;
}

export function dispatch(id: number, type: string, detail: Record<string, unknown>): Outcome {
  const target: unknown = id === 0 ? fakeDocument : nodeFor(id);
  if (target === undefined) {
    return { prevented: false, clipboard: null };
  }
  if (target instanceof SElement) {
    rememberState(target, type, detail);
  }

  const path: unknown[] = [];
  for (let node = target as SNode | null; node instanceof SNode; node = node.parentNode) {
    path.push(node);
  }
  if (!(target instanceof SNode)) {
    path.push(target);
  }
  path.push(fakeDocument, globalThis);

  let written: string | null = null;
  const bubbles = detail.bubbles !== false;
  const event: HostEvent = {
    ...detail,
    clipboardData: {
      types: ["text/plain"],
      files: [],
      items: [],
      getData: () => String(detail.clipboardText ?? ""),
      setData: (_format: string, text: string) => {
        written = String(text);
      },
    },
    type,
    target,
    currentTarget: null,
    bubbles,
    cancelBubble: false,
    defaultPrevented: false,
    composedPath: () => path,
    stopPropagation() {
      event.cancelBubble = true;
    },
    stopImmediatePropagation() {
      event.cancelBubble = true;
    },
    preventDefault() {
      event.defaultPrevented = true;
    },
  };

  for (const stop of path) {
    if (stop !== target && !bubbles) {
      break;
    }
    const handlers = listenersOf(stop, type);
    event.currentTarget = stop;
    for (const handler of handlers) {
      handler.call(stop, event);
    }
    if (event.cancelBubble) {
      break;
    }
  }
  return { prevented: event.defaultPrevented, clipboard: written };
}

function listenersOf(stop: unknown, type: string): EventHandler[] {
  if (stop instanceof SElement) {
    return [...(stop.listeners.get(type) ?? [])];
  }
  if (stop === fakeDocument) {
    return [...(documentTarget.listeners.get(type) ?? [])];
  }
  if (stop === globalThis) {
    return [...(windowTarget.listeners.get(type) ?? [])];
  }
  return [];
}

/** What an event says about the element, before any handler reads it. */
function rememberState(target: SElement, type: string, detail: Record<string, unknown>): void {
  if (type === "scroll") {
    target.scrollLeftValue = Number(detail.scrollLeft ?? 0);
    target.scrollTopValue = Number(detail.scrollTop ?? 0);
    return;
  }
  if ((type === "input" || type === "change") && target.localName === "select") {
    const chosen = Number(detail.selectedIndex ?? -1);
    target.options.forEach((option, index) => {
      option.attributes.set("selected", index === chosen);
    });
    return;
  }
  if (type === "input" || type === "change") {
    if (typeof detail.value === "string") {
      target.attributes.set("value", detail.value);
    }
    if (typeof detail.checked === "boolean") {
      target.attributes.set("checked", detail.checked);
    }
    if (typeof detail.selectionStart === "number") {
      target.selectionStartValue = detail.selectionStart;
      target.selectionEndValue = Number(detail.selectionEnd ?? detail.selectionStart);
    }
    return;
  }
  if (type === "focus" || type === "focusin") {
    fakeDocument.activeElement = target;
    return;
  }
  if (type === "blur" || type === "focusout") {
    if (fakeDocument.activeElement === target) {
      fakeDocument.activeElement = null;
    }
  }
}
