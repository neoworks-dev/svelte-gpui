// What overlays on a page know of each other. An overlay's content is drawn in a layer above the app,
// so it is no longer inside the element it belongs to. Each overlay keeps a marker where it was written,
// and these functions find, from something inside an overlay, the elements it logically sits in.

import { layerElement } from "./base";

/** The marker each overlay element leaves at its place in the tree. */
const markers = new WeakMap<Element, Element>();

/** The order overlays appeared in, so that a later one is drawn over an earlier one. */
let appearances = 0;

export function registerOverlay(overlay: Element, marker: Element): () => void {
  markers.set(overlay, marker);
  forwardKeys();
  return () => markers.delete(overlay);
}

/** The next position in the order overlays are drawn in, within a layer. */
export function nextAppearance(): number {
  return ++appearances;
}

/** The z-index of an overlay on `layer` that appeared as number `appearance`. */
export const stacking = (layer: number, appearance: number): number => layer * 100000 + appearance;

/** The element an overlay is placed against: the one its marker sits in. */
export function parentOf(overlay: Element): Element | null {
  return markers.get(overlay)?.parentElement ?? null;
}

/**
 * Whether `target` is inside `overlay`, or inside an overlay that was opened from inside it. A press in such a
 * child is not outside, though in the page it is neither in `overlay` nor under it.
 */
export function insideOverlay(overlay: Element, target: Node | null): boolean {
  let node: Node | null = target;
  while (node !== null) {
    if (node === overlay) {
      return true;
    }
    const holder: Element | null = node instanceof Element ? node.closest(".g-overlay") : node.parentElement?.closest(".g-overlay") ?? null;
    if (holder === null) {
      return false;
    }
    if (holder === overlay) {
      return true;
    }
    // Continue from where that overlay was opened.
    node = markers.get(holder) ?? null;
  }
  return false;
}

let forwarding = false;

/**
 * A key that nothing in an overlay stopped goes on to the elements the overlay sits in, and then to the window,
 * as it does in the host's tree, where the overlay is a child of its parent.
 */
function forwardKeys(): void {
  if (forwarding) {
    return;
  }
  forwarding = true;
  const layer = layerElement();
  for (const type of ["keydown", "keyup"] as const) {
    layer.addEventListener(type, (event) => {
      const original = event as KeyboardEvent;
      const overlay = (original.target as Element | null)?.closest?.(".g-overlay");
      const parent = overlay ? parentOf(overlay) : null;
      if (!parent || original.isComposing) {
        return;
      }
      original.stopPropagation();
      const copy = new KeyboardEvent(type, {
        key: original.key,
        code: original.code,
        repeat: original.repeat,
        ctrlKey: original.ctrlKey,
        shiftKey: original.shiftKey,
        altKey: original.altKey,
        metaKey: original.metaKey,
        bubbles: true,
        cancelable: true,
        composed: true,
      });
      parent.dispatchEvent(copy);
      if (copy.defaultPrevented) {
        original.preventDefault();
      }
    });
  }
}
