// Tells an element's owner where it was laid out, as the host does after each frame: in window
// coordinates, and again whenever that changes, whether it was resized or only moved.
//
// A `ResizeObserver` hears size changes but not moves, and a move is what scrolling, a sibling
// growing or a panel opening does. So something that might have moved anything, such as a scroll,
// a change to the page, an animation or the window resizing, asks for one check on the next
// frame, which measures every watched element. Nothing runs while the page is still.

import type { Layout } from "../../ui/events";

interface Watch {
  element: Element;
  /** Only a change of width or height counts. */
  sizeOnly: boolean;
  callback: (layout: Layout) => void;
  last: Layout | null;
}

const watches = new Set<Watch>();
let resizes: ResizeObserver | undefined;
let changes: MutationObserver | undefined;
let pending = 0;

const same = (a: number, b: number): boolean => Math.abs(a - b) < 0.01;

/** The element's box in the window. */
export function measure(element: Element): Layout {
  const box = element.getBoundingClientRect();
  return { x: box.x, y: box.y, width: box.width, height: box.height };
}

function changed(watch: Watch, now: Layout): boolean {
  const before = watch.last;
  if (before === null) {
    return true;
  }
  const resized = !same(before.width, now.width) || !same(before.height, now.height);
  return watch.sizeOnly ? resized : resized || !same(before.x, now.x) || !same(before.y, now.y);
}

function check(): void {
  pending = 0;
  let moved = false;
  for (const watch of [...watches]) {
    if (!watches.has(watch)) {
      continue;
    }
    const now = measure(watch.element);
    if (changed(watch, now)) {
      watch.last = now;
      moved = true;
      watch.callback(now);
    }
  }
  // What a callback did may have moved something else, so look once more.
  if (moved) {
    ask();
  }
}

/** Asks for a check on the next frame. Asking again before it runs changes nothing. */
export function ask(): void {
  if (pending === 0 && watches.size > 0) {
    pending = requestAnimationFrame(check);
  }
}

const EVENTS = ["scroll", "resize", "transitionend", "animationend", "load", "pointerup", "keyup"];

function start(): void {
  resizes = new ResizeObserver(ask);
  changes = new MutationObserver(ask);
  changes.observe(document.documentElement, { subtree: true, childList: true, attributes: true, characterData: true });
  for (const type of EVENTS) {
    window.addEventListener(type, ask, { capture: true, passive: true });
  }
}

function stop(): void {
  resizes?.disconnect();
  changes?.disconnect();
  resizes = changes = undefined;
  for (const type of EVENTS) {
    window.removeEventListener(type, ask, { capture: true });
  }
  if (pending !== 0) {
    cancelAnimationFrame(pending);
    pending = 0;
  }
}

/**
 * Calls `callback` with the element's box once it is laid out, and each time that changes. With
 * `sizeOnly`, only when its width or height changed. Returns the function that stops.
 */
export function watchLayout(element: Element, callback: (layout: Layout) => void, sizeOnly = false): () => void {
  const watch: Watch = { element, sizeOnly, callback, last: null };
  if (watches.size === 0) {
    start();
  }
  watches.add(watch);
  resizes?.observe(element);
  ask();
  return () => {
    if (!watches.delete(watch)) {
      return;
    }
    resizes?.unobserve(element);
    if (watches.size === 0) {
      stop();
    }
  };
}
