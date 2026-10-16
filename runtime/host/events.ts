// Delivers an event the host reports to the element it names. A primitive's events
// go to that element only: nothing bubbles, so a handler sees exactly what its own
// element did. The exception is `key`, which goes up through the focused element's
// ancestors, as a shortcut would, and then reaches `window` as `keydown`.
//
// Event names are chosen to stay clear of the ones Svelte delegates to the root
// (`click`, `input`, `keydown`, ...), so Svelte attaches each handler directly.

import { fakeDocument, windowTarget } from "../dom/globals";
import { type EventHandler, SElement, SNode } from "../dom/nodes";
import { focusMoved } from "../dom/focus";
import { nodeFor } from "./bridge";

export interface Outcome {
  /** A handler called `preventDefault`, so the host must not do its default action. */
  prevented: boolean;
}

export interface HostEvent {
  type: string;
  target: SNode | null;
  defaultPrevented: boolean;
  preventDefault(): void;
  stopPropagation(): void;
  [field: string]: unknown;
}

function makeEvent(type: string, detail: Record<string, unknown>): HostEvent & { stopped: boolean } {
  const event = {
    ...detail,
    type,
    target: null as SNode | null,
    defaultPrevented: false,
    stopped: false,
    preventDefault() {
      event.defaultPrevented = true;
    },
    stopPropagation() {
      event.stopped = true;
    },
  };
  return event;
}

export function dispatch(id: number, type: string, detail: Record<string, unknown>): Outcome {
  const event = makeEvent(type, detail);
  if (id === 0) {
    toWindow(windowName(type), event);
    return { prevented: event.defaultPrevented };
  }
  const target = nodeFor(id);
  if (!(target instanceof SElement)) {
    return { prevented: false };
  }
  remember(target, type, detail);
  event.target = target;
  if (type !== "key") {
    run(target, type, event);
    return { prevented: event.defaultPrevented };
  }
  for (let node: SNode | null = target; node !== null && !event.stopped; node = node.parentNode) {
    if (node instanceof SElement) {
      run(node, type, event);
    }
  }
  if (!event.stopped) {
    toWindow("keydown", event);
  }
  return { prevented: event.defaultPrevented };
}

function run(element: SElement, type: string, event: HostEvent): void {
  for (const handler of [...(element.listeners.get(type) ?? [])]) {
    handler.call(element, event);
  }
}

/** `<svelte:window onkeydown>` expects a DOM keyboard event, so it gets the DOM's names too. */
function toWindow(type: string, event: HostEvent): void {
  const handlers: EventHandler[] = [...(windowTarget.listeners.get(type) ?? [])];
  if (handlers.length === 0) {
    return;
  }
  Object.assign(event, {
    ctrlKey: event.ctrl,
    shiftKey: event.shift,
    altKey: event.alt,
    metaKey: event.meta,
  });
  for (const handler of handlers) {
    handler.call(globalThis, event);
  }
}

function windowName(type: string): string {
  return type === "key" ? "keydown" : type;
}

/** What an event says about the element, written down before any handler reads it. */
function remember(target: SElement, type: string, detail: Record<string, unknown>): void {
  switch (type) {
    case "edit":
      // The host already shows this text; writing the attribute directly does not echo it back.
      target.attributes.set("value", detail.value);
      return;
    case "scroll":
      target.scrollLeftValue = Number(detail.x ?? 0);
      target.scrollTopValue = Number(detail.y ?? 0);
      return;
    case "layout":
      target.boxLeft = Number(detail.x ?? 0);
      target.boxTop = Number(detail.y ?? 0);
      target.offsetWidth = target.clientWidth = Number(detail.width ?? 0);
      target.offsetHeight = target.clientHeight = Number(detail.height ?? 0);
      return;
    case "focus":
      focusMoved(target, typeof detail.from === "number" ? nodeFor(detail.from) : undefined);
      return;
    case "blur":
      if (fakeDocument.activeElement === target) {
        fakeDocument.activeElement = null;
      }
  }
}
