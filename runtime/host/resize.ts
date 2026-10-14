// `ResizeObserver`, and the sizes Svelte reads through `bind:clientWidth`. The
// host measures an element after layout and reports it; this turns that into
// the entries observers expect.

import { SElement } from "../dom/nodes";
import { nodeFor, observeSize } from "./bridge";

type Callback = (entries: unknown[]) => void;

class HostResizeObserver {
  private readonly elements = new Set<SElement>();

  constructor(private readonly callback: Callback) {
    observers.add(this);
  }

  observe(element: SElement): void {
    this.elements.add(element);
    observeSize(element);
  }

  unobserve(element: SElement): void {
    this.elements.delete(element);
  }

  disconnect(): void {
    this.elements.clear();
  }

  /** Calls back with an entry for the element, if this observer watches it. */
  notify(element: SElement, width: number, height: number): void {
    if (!this.elements.has(element)) {
      return;
    }
    const box = [{ inlineSize: width, blockSize: height }];
    this.callback([
      {
        target: element,
        contentRect: { x: 0, y: 0, top: 0, left: 0, width, height, right: width, bottom: height },
        borderBoxSize: box,
        contentBoxSize: box,
        devicePixelContentBoxSize: box,
      },
    ]);
  }
}

const observers = new Set<HostResizeObserver>();

export function installResizeObserver(): void {
  (globalThis as any).ResizeObserver = HostResizeObserver;
}

/** Records a measured size on the element and tells whoever is watching. */
export function reportSize(id: number, left: number, top: number, width: number, height: number): void {
  const node = nodeFor(id);
  if (!(node instanceof SElement)) {
    return;
  }
  node.boxLeft = left;
  node.boxTop = top;
  node.clientWidth = width;
  node.offsetWidth = width;
  node.clientHeight = height;
  node.offsetHeight = height;
  for (const observer of observers) {
    observer.notify(node, width, height);
  }
}
