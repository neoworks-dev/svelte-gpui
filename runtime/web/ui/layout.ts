// Tells an element's owner where it was laid out, as the host does after each frame: in window
// coordinates, and again whenever that changes.
//
// A size change is what a `ResizeObserver` reports, for exactly the elements that changed, so a
// watch that only wants sizes (`onresize`) costs nothing while its element keeps its size, however
// much else on the page changes.
//
// A move is what it doesn't report: scrolling, a sibling growing or a panel opening moves an
// element without resizing it. So while any watch wants moves too (`onlayout`), something that
// might have moved anything (a scroll, a change to the page, an animation, the window resizing)
// asks for one check on the next frame, which measures those watches, and only those. A page
// whose watches all want sizes never measures on a change it doesn't see.

import type { Layout } from "../../ui/events";

interface Watch {
  element: Element;
  /** Only a change of width or height counts. */
  sizeOnly: boolean;
  callback: (layout: Layout) => void;
  last: Layout | null;
}

/** Watches that want sizes only, by element: driven by the `ResizeObserver` alone. */
const sized = new Map<Element, Set<Watch>>();
/** Watches that want moves too: measured on every check. */
const placed = new Set<Watch>();
/** How many watches each observed element has, so it is observed once and let go with the last. */
const observed = new Map<Element, number>();
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

/** Tells the watch if its element's box changed (as it counts it); whether it did. */
function report(watch: Watch, now: Layout): boolean {
  if (!changed(watch, now)) {
    return false;
  }
  watch.last = now;
  watch.callback(now);
  return true;
}

/** The elements that changed size: their size watches hear it; a move watch may have moved. */
function resized(entries: ResizeObserverEntry[]): void {
  for (const entry of entries) {
    const watches = sized.get(entry.target);
    if (watches === undefined) {
      continue;
    }
    const now = measure(entry.target);
    for (const watch of [...watches]) {
      if (watches.has(watch)) {
        report(watch, now);
      }
    }
  }
  ask();
}

function check(): void {
  pending = 0;
  let moved = false;
  for (const watch of [...placed]) {
    if (placed.has(watch) && report(watch, measure(watch.element))) {
      moved = true;
    }
  }
  // What a callback did may have moved something else, so look once more.
  if (moved) {
    ask();
  }
}

/**
 * Asks for a check of the watches that want moves on the next frame; nothing when there are none.
 * Asking again before it runs changes nothing.
 */
export function ask(): void {
  if (pending === 0 && placed.size > 0) {
    pending = requestAnimationFrame(check);
  }
}

const EVENTS = ["scroll", "resize", "transitionend", "animationend", "load", "pointerup", "keyup"];

/** Hears what may move things: only while a watch wants moves. */
function listen(): void {
  changes = new MutationObserver(ask);
  changes.observe(document.documentElement, { subtree: true, childList: true, attributes: true, characterData: true });
  for (const type of EVENTS) {
    window.addEventListener(type, ask, { capture: true, passive: true });
  }
}

function unlisten(): void {
  changes?.disconnect();
  changes = undefined;
  for (const type of EVENTS) {
    window.removeEventListener(type, ask, { capture: true });
  }
  if (pending !== 0) {
    cancelAnimationFrame(pending);
    pending = 0;
  }
}

function observe(element: Element): void {
  resizes ??= new ResizeObserver(resized);
  const count = observed.get(element) ?? 0;
  observed.set(element, count + 1);
  if (count === 0) {
    resizes.observe(element);
  }
}

function unobserve(element: Element): void {
  const count = (observed.get(element) ?? 1) - 1;
  if (count > 0) {
    observed.set(element, count);
    return;
  }
  observed.delete(element);
  resizes?.unobserve(element);
  if (observed.size === 0) {
    resizes?.disconnect();
    resizes = undefined;
  }
}

/**
 * Calls `callback` with the element's box once it is laid out, and each time that changes. With
 * `sizeOnly`, only when its width or height changed. Returns the function that stops.
 */
export function watchLayout(element: Element, callback: (layout: Layout) => void, sizeOnly = false): () => void {
  const watch: Watch = { element, sizeOnly, callback, last: null };
  // A new observation is reported once laid out, so a size watch hears its first box from it.
  observe(element);
  if (sizeOnly) {
    let watches = sized.get(element);
    if (watches === undefined) {
      watches = new Set();
      sized.set(element, watches);
    }
    watches.add(watch);
  } else {
    if (placed.size === 0) {
      listen();
    }
    placed.add(watch);
    ask();
  }
  let stopped = false;
  return () => {
    if (stopped) {
      return;
    }
    stopped = true;
    unobserve(element);
    if (sizeOnly) {
      const watches = sized.get(element);
      watches?.delete(watch);
      if (watches?.size === 0) {
        sized.delete(element);
      }
    } else {
      placed.delete(watch);
      if (placed.size === 0) {
        unlisten();
      }
    }
  };
}
