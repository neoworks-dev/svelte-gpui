// Turns an event the host reports into the DOM event Svelte expects, and walks
// it up the tree. Svelte delegates the usual events (`click`, `input`, ...) to a
// listener on the mount root, which reads `composedPath()`; anything else sits
// on its own element.

import { SElement, SNode } from "../dom/nodes";
import { nodeFor } from "./bridge";

interface HostEvent {
  type: string;
  target: SNode;
  currentTarget: SNode | null;
  bubbles: boolean;
  cancelBubble: boolean;
  defaultPrevented: boolean;
  composedPath(): SNode[];
  stopPropagation(): void;
  stopImmediatePropagation(): void;
  preventDefault(): void;
  [key: string]: unknown;
}

/** Delivers an event of `type` to the node with `id`, bubbling to the root. */
export function dispatch(id: number, type: string, detail: Record<string, unknown>): void {
  const target = nodeFor(id);
  if (target === undefined) {
    return;
  }
  const path: SNode[] = [];
  for (let node: SNode | null = target; node !== null; node = node.parentNode) {
    path.push(node);
  }

  const event: HostEvent = {
    ...detail,
    type,
    target,
    currentTarget: null,
    bubbles: true,
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

  for (const node of path) {
    if (!(node instanceof SElement)) {
      continue;
    }
    event.currentTarget = node;
    for (const handler of [...(node.listeners.get(type) ?? [])]) {
      handler.call(node, event);
    }
    if (event.cancelBubble) {
      return;
    }
  }
}
