// Where focus came from, so `blur()` can hand it back. The host reports each move of focus
// with the element that had it before; the elements are remembered weakly, so one that leaves
// the tree is not kept alive.

import { fakeDocument } from "./globals";
import type { SElement, SNode } from "./nodes";

const cameFrom = new WeakMap<SNode, SNode>();
/** The element `blur()` just gave focus back to, whose arrival must not be recorded as a move. */
let handedBack: SNode | null = null;

/** The host reported that `target` has focus, taken from `from` (nothing, when it had none). */
export function focusMoved(target: SNode, from: SNode | undefined): void {
  fakeDocument.activeElement = target;
  const returning = handedBack === target;
  handedBack = null;
  if (returning) {
    return;
  }
  if (from !== undefined && from !== target) {
    cameFrom.set(target, from);
  } else {
    cameFrom.delete(target);
  }
}

/** The top of the tree a node is in. `isConnected` cannot say whether two nodes share one. */
function treeOf(node: SNode): SNode {
  while (node.parentNode !== null) {
    node = node.parentNode;
  }
  return node;
}

/**
 * What `blur()` does on `element`: nothing unless it has focus. Then the element that had
 * focus before it gets it back if that is still in the tree, and otherwise it is returned
 * as `null` and the host drops focus altogether.
 */
export function handBackFocus(element: SElement): SElement | null {
  if (fakeDocument.activeElement !== element) {
    return null;
  }
  const back = cameFrom.get(element) ?? null;
  cameFrom.delete(element);
  if (back === null || treeOf(back) !== treeOf(element)) {
    return null;
  }
  handedBack = back;
  return back as SElement;
}
