// Where focus came from, so `blur()` can hand it back, as the host does: the element that had
// focus before this one took it gets it again, if it is still on the page. The elements are kept
// weakly, so one that leaves the page is not kept alive.

const cameFrom = new WeakMap<Element, Element>();
/** The element `blur()` just gave focus back to, whose arrival must not be recorded as a move. */
let handedBack: Element | null = null;
let tracking = false;

/** Starts remembering moves of focus. Safe to call again. */
export function trackFocus(): void {
  if (tracking || typeof document === "undefined") {
    return;
  }
  tracking = true;
  document.addEventListener(
    "focusin",
    (event) => {
      const target = event.target as Element;
      const from = (event as FocusEvent).relatedTarget as Element | null;
      const returning = handedBack === target;
      handedBack = null;
      if (returning) {
        return;
      }
      if (from !== null && from !== target) {
        cameFrom.set(target, from);
      } else {
        cameFrom.delete(target);
      }
    },
    true,
  );
}

/**
 * What `blur()` does on `element`: nothing unless it has focus. Then the element that had focus before
 * it gets it back if that is still on the page, and otherwise focus is dropped altogether.
 */
export function giveUpFocus(element: HTMLElement): void {
  if (document.activeElement !== element) {
    return;
  }
  const back = cameFrom.get(element) as HTMLElement | undefined;
  cameFrom.delete(element);
  if (back !== undefined && back.isConnected) {
    handedBack = back;
    back.focus();
    return;
  }
  element.blur();
}
